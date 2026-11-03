//! Experimental, integer-only, lossless implicit-context-match codec.
//!
//! **AYEOS-inspired, not actual MEM8 production retrieval.** The name describes
//! this experiment, not compatibility with, or an implementation of, MEM8.
//! A deterministic 256 x 256 single-entry lattice indexes previous source
//! positions by their preceding 4 or 8 bytes. Hash collisions lose opportunities,
//! never correctness: the complete preceding context must also match. There is
//! no learned model, floating-point arithmetic, probabilistic reconstruction,
//! production-retrieval claim, or promise of better compression than plain zlib.
//!
//! # M8L1 wire format (all multibyte header integers are little-endian u64)
//!
//! | Bytes | Meaning |
//! |---|---|
//! | 0..4 | `M8L1` magic / format version |
//! | 4 | flags: bit 0 = Hutter Demon transform; all other bits forbidden |
//! | 5 | context bytes: exactly 4 or 8 |
//! | 6 | minimum match length: 4..=255 |
//! | 7 | reserved, must be zero |
//! | 8..16 | original byte length |
//! | 16..24 | source byte length (raw or Demon-transformed) |
//! | 24..32 | FNV-1a-64 of the original bytes (not an authentication tag) |
//! | 32..40 | uncompressed control-stream byte length |
//! | 40..48 | uncompressed literal-stream byte length |
//! | 48..56 | compressed control-stream byte length |
//! | 56..64 | compressed literal-stream byte length |
//! | 64.. | one control zlib member, then one literal zlib member; no trailing bytes |
//!
//! Both streams use independent zlib level 9. Control tokens are canonical
//! unsigned LEB128: even `literal_len << 1`, odd
//! `((match_len - min_match) << 1) | 1`. Literal runs must be nonempty. Tokens
//! select literal runs or matches without storing distances; consecutive matches
//! are allowed. Literal bytes are consumed in order from the literal stream.
//! No terminator is stored: all three uncompressed streams must end exactly.
//!
//! At position p, query the lattice BEFORE inserting p. A match uses the single
//! candidate with identical preceding context, and the encoder greedily takes
//! its maximum exact forward match (including overlap) if long enough. Insert
//! EVERY consumed position, including positions inside a literal run or match.
//! The decoder inserts position j from the context preceding j before appending
//! byte j. Overlap copies append sequentially; the selected candidate remains
//! fixed for that token even if intermediate lattice entries change.
//!
//! Sources and originals are limited to u32::MAX and addressable Vec lengths.
//! Decode limits are checked before stream allocation; inflate grows in bounded
//! chunks, never reserves a header-declared giant capacity, checks zlib end and
//! exact member consumption, and stops at declared length + 1. Demon inversion
//! is length-preflighted because the existing inverse has no output-limit API.

use crate::hutter_demon;
use flate2::write::ZlibEncoder;
use flate2::{Compression, Decompress, FlushDecompress, Status};
use std::io::Write;

const MAGIC: &[u8; 4] = b"M8L1";
const HEADER_LEN: usize = 64;
const SLOTS: usize = 256 * 256;
const EMPTY: usize = usize::MAX;

/// Parameters are recorded in the container; no decoder-side options are needed.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub context_bytes: u8,
    pub min_match: u8,
    /// Transform the entire input with Hutter Demon before lattice matching.
    /// This embeds transformed bytecode, not a nested DC10/zlib container.
    pub dc10: bool,
}

/// Experiment telemetry. Stream byte counts are BEFORE zlib; `bytes.len()` is
/// the actual complete container size, including both zlib members and header.
#[derive(Debug)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub matches: u64,
    pub matched_bytes: u64,
    pub control_bytes: usize,
    pub literal_bytes: usize,
}

fn validate_options(options: Options) -> Result<(), String> {
    if options.context_bytes != 4 && options.context_bytes != 8 {
        return Err("context_bytes must be 4 or 8".into());
    }
    if options.min_match < 4 {
        return Err("min_match must be at least 4".into());
    }
    Ok(())
}

fn source_length(value: u64) -> Result<usize, String> {
    if value > u32::MAX as u64 || value > isize::MAX as u64 {
        return Err("source/original length exceeds address limit or u32::MAX".into());
    }
    Ok(value as usize)
}

fn address_length(value: u64) -> Result<usize, String> {
    if value > isize::MAX as u64 {
        return Err("stream length exceeds address limit".into());
    }
    Ok(value as usize)
}

/// Context is a zero-extended little-endian u64; no context exists before k.
/// Fold the high and low halves, then use the specified AYEOS integer
/// multipliers with explicit u32 wrapping. The upper product bytes give x/y;
/// slot = y * 256 + x. This precise mapping is part of M8L1, not a MEM8 API.
fn context_slot(source: &[u8], pos: usize, k: usize) -> Option<usize> {
    if pos < k {
        return None;
    }
    let mut bytes = [0u8; 8];
    bytes[..k].copy_from_slice(&source[pos - k..pos]);
    let context = u64::from_le_bytes(bytes);
    let folded = context as u32 ^ (context >> 32) as u32;
    let x = (folded.wrapping_mul(2_654_435_761) >> 24) as usize;
    let y = (folded.wrapping_mul(1_597_334_677) >> 24) as usize;
    Some(y * 256 + x)
}

fn candidate(source: &[u8], pos: usize, k: usize, table: &[usize]) -> Option<usize> {
    let old = table[context_slot(source, pos, k)?];
    if old == EMPTY || old < k || old >= pos {
        return None;
    }
    if source[old - k..old] != source[pos - k..pos] {
        return None;
    }
    Some(old)
}

fn insert_position(source: &[u8], pos: usize, k: usize, table: &mut [usize]) {
    if let Some(slot) = context_slot(source, pos, k) {
        table[slot] = pos;
    }
}

/// g8t discovery: hutter_uleb128_encode_u64, equivalence class
/// (u64)->u64::398a2f2d8912. Adapted interface joint: scalar-value result to
/// byte-stream writer. Only capsule metadata was available; this conventional
/// ULEB128 implementation is new code, NOT claimed capsule-source reuse.
fn write_varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        out.push(byte | if value != 0 { 0x80 } else { 0 });
        if value == 0 {
            break;
        }
    }
}

fn read_varint(bytes: &[u8], cursor: &mut usize) -> Result<u64, String> {
    let mut value = 0u64;
    for index in 0..10 {
        let byte = *bytes.get(*cursor).ok_or("truncated control varint")?;
        *cursor += 1;
        // Only bit zero may be used in the tenth byte of a u64 varint.
        if index == 9 && byte > 1 {
            return Err("overflowing control varint".into());
        }
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            if index != 0 && byte == 0 {
                return Err("noncanonical control varint".into());
            }
            return Ok(value);
        }
    }
    Err("overlong control varint".into())
}

fn zlib_encode(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    encoder
        .write_all(bytes)
        .map_err(|e| format!("zlib encode: {e}"))?;
    encoder.finish().map_err(|e| format!("zlib finish: {e}"))
}

/// Low-level inflate is intentional: Read-based wrappers can return EOF for a
/// truncated member without proving StreamEnd. Never accept missing trailers,
/// concatenated members, or unused compressed input. One excess output byte is
/// sufficient to reject a dishonest declared length without inflating a bomb.
fn zlib_decode(bytes: &[u8], declared: usize) -> Result<Vec<u8>, String> {
    let limit = declared.checked_add(1).ok_or("inflate limit overflow")?;
    let mut decoder = Decompress::new(true);
    let mut out = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let consumed = decoder.total_in() as usize;
        let available = chunk.len().min(limit - out.len());
        let before_in = decoder.total_in();
        let before_out = decoder.total_out();
        let status = decoder
            .decompress(
                &bytes[consumed..],
                &mut chunk[..available],
                FlushDecompress::None,
            )
            .map_err(|e| format!("invalid zlib stream: {e}"))?;
        let produced = (decoder.total_out() - before_out) as usize;
        out.try_reserve(produced)
            .map_err(|e| format!("inflate allocation: {e}"))?;
        out.extend_from_slice(&chunk[..produced]);
        if out.len() > declared {
            return Err("zlib output exceeds declared length".into());
        }
        if status == Status::StreamEnd {
            if decoder.total_in() != bytes.len() as u64 {
                return Err("trailing bytes in zlib stream".into());
            }
            if out.len() != declared {
                return Err("zlib output length mismatch".into());
            }
            return Ok(out);
        }
        if decoder.total_in() == before_in && decoder.total_out() == before_out {
            return Err("truncated or stalled zlib stream".into());
        }
    }
}

fn pack_container(
    original: &[u8],
    source_len: usize,
    controls: &[u8],
    literals: &[u8],
    options: Options,
) -> Result<Vec<u8>, String> {
    let control_z = zlib_encode(controls)?;
    let literal_z = zlib_encode(literals)?;
    let mut out = Vec::new();
    let total = HEADER_LEN
        .checked_add(control_z.len())
        .and_then(|n| n.checked_add(literal_z.len()))
        .ok_or("container length overflow")?;
    out.try_reserve(total)
        .map_err(|e| format!("container allocation: {e}"))?;
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&[
        u8::from(options.dc10),
        options.context_bytes,
        options.min_match,
        0,
    ]);
    for value in [
        original.len() as u64,
        source_len as u64,
        hutter_demon::fnv1a64_bytes(original),
        controls.len() as u64,
        literals.len() as u64,
        control_z.len() as u64,
        literal_z.len() as u64,
    ] {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&control_z);
    out.extend_from_slice(&literal_z);
    Ok(out)
}

/// Greedy implicit-context matching followed by independent level-9 zlib.
/// Statistics refer to the raw/transformed source, not necessarily original
/// bytes. This experiment deliberately has no raw fallback or ratio guarantee.
pub fn compress(input: &[u8], options: Options) -> Result<Encoded, String> {
    validate_options(options)?;
    source_length(input.len() as u64)?;
    // The transform may escape every input byte. Check its worst case before
    // calling the existing encoder, which does not expose a size-limit API.
    let transformed;
    let source = if options.dc10 {
        let bound = (input.len() as u64)
            .checked_mul(2)
            .ok_or("transform size overflow")?;
        source_length(bound)?;
        transformed = hutter_demon::demon_transform_encode(input).encoded;
        transformed.as_slice()
    } else {
        input
    };
    source_length(source.len() as u64)?;
    let k = options.context_bytes as usize;
    let mut table = vec![EMPTY; SLOTS];
    let mut controls = Vec::new();
    let mut literals = Vec::new();
    let mut literal_run = 0u64;
    let mut matches = 0u64;
    let mut matched_bytes = 0u64;
    let mut pos = 0usize;
    while pos < source.len() {
        let mut length = 0usize;
        if let Some(old) = candidate(source, pos, k, &table) {
            // old < pos guarantees old + length is addressable whenever
            // pos + length is. Full-source comparison permits overlapping LZ.
            while length < source.len() - pos && source[old + length] == source[pos + length] {
                length += 1;
            }
        }
        if length >= options.min_match as usize {
            if literal_run != 0 {
                write_varint(literal_run << 1, &mut controls);
                literal_run = 0;
            }
            write_varint(
                (((length - options.min_match as usize) as u64) << 1) | 1,
                &mut controls,
            );
            for j in pos..pos + length {
                insert_position(source, j, k, &mut table);
            }
            pos += length;
            matches += 1;
            matched_bytes += length as u64;
        } else {
            insert_position(source, pos, k, &mut table);
            literals.push(source[pos]);
            literal_run += 1;
            pos += 1;
        }
    }
    if literal_run != 0 {
        write_varint(literal_run << 1, &mut controls);
    }
    let bytes = pack_container(input, source.len(), &controls, &literals, options)?;
    Ok(Encoded {
        bytes,
        matches,
        matched_bytes,
        control_bytes: controls.len(),
        literal_bytes: literals.len(),
    })
}

fn header_u64(container: &[u8], offset: usize) -> u64 {
    // Only called after the fixed header has been validated.
    u64::from_le_bytes(container[offset..offset + 8].try_into().unwrap())
}

/// Preflight exactly the grammar accepted by demon_transform_decode, without
/// constructing its output. Static patterns and opcode constants come from the
/// existing module rather than a divergent copied dictionary. Every addition
/// is bounded by the already validated original length. Pipe suffix bytes are
/// literal in that grammar, including bytes >= 0x80.
fn preflight_demon(source: &[u8], expected: usize) -> Result<(), String> {
    let mut lengths = [0usize; 126];
    for spec in hutter_demon::STATIC_DEMONS {
        let index = (spec.opcode - 0x80) as usize;
        if index < lengths.len() {
            lengths[index] = spec.pattern.len();
        }
    }
    let mut pos = 0usize;
    let mut output = 0usize;
    while pos < source.len() {
        let byte = source[pos];
        let added;
        if byte < 0x80 {
            added = 1;
            pos += 1;
        } else if byte == hutter_demon::OP_ESCAPE {
            if source.len() - pos < 2 {
                return Err("truncated Demon escape".into());
            }
            added = 1;
            pos += 2;
        } else if byte == hutter_demon::OP_PIPE_PREFIX {
            if source.len() - pos < 3 {
                return Err("truncated Demon pipe header".into());
            }
            let target_len = usize::from(source[pos + 1] & 0x3f) + 1;
            let target_start = pos + 2;
            if target_len >= source.len() - target_start {
                return Err("truncated Demon pipe target".into());
            }
            let target_end = target_start + target_len;
            let prefix_len = usize::from(source[target_end]);
            if prefix_len == 0 || prefix_len > target_len {
                return Err("invalid Demon pipe prefix length".into());
            }
            let suffix_start = target_end + 1;
            let close = source[suffix_start..]
                .iter()
                .position(|&b| b == hutter_demon::OP_WIKI_CLOSE)
                .ok_or("unterminated Demon pipe suffix")?;
            added = target_len
                .checked_add(prefix_len)
                .and_then(|n| n.checked_add(close))
                .and_then(|n| n.checked_add(5))
                .ok_or("Demon length overflow")?;
            pos = suffix_start + close + 1;
        } else {
            added = lengths[(byte - 0x80) as usize];
            if added == 0 {
                return Err("unknown Demon opcode".into());
            }
            pos += 1;
        }
        if added > expected - output {
            return Err("Demon expansion exceeds original length".into());
        }
        output += added;
    }
    if output != expected {
        return Err("Demon original length mismatch".into());
    }
    // Existing inverse uses this initial capacity; prove addressability too.
    source
        .len()
        .checked_add(source.len() / 8)
        .filter(|&n| n <= isize::MAX as usize)
        .ok_or("Demon inverse capacity exceeds address limit")?;
    Ok(())
}

/// Decode with an explicit original-output limit. The intermediate Demon
/// source may contain up to twice the original length. Controls are bounded by
/// twice source length (each positive token produces at least half its own
/// encoded byte count); literals by source length. These bounds are checked
/// before inflation. max_output bounds output bytes, not total working memory.
/// All complete members, token streams, output lengths and original FNV must
/// verify; a malformed container never intentionally returns partial output.
pub fn decompress(container: &[u8], max_output: usize) -> Result<Vec<u8>, String> {
    if container.len() < HEADER_LEN || &container[..4] != MAGIC {
        return Err("invalid or truncated M8L1 header".into());
    }
    if container[4] & !1 != 0 || container[7] != 0 {
        return Err("unknown flags or reserved header byte".into());
    }
    let options = Options {
        context_bytes: container[5],
        min_match: container[6],
        dc10: container[4] == 1,
    };
    validate_options(options)?;
    let original_len = source_length(header_u64(container, 8))?;
    let source_len = source_length(header_u64(container, 16))?;
    if original_len > max_output {
        return Err("original length exceeds max_output".into());
    }
    if options.dc10 {
        if source_len as u64 > (original_len as u64) * 2 || (source_len == 0) != (original_len == 0)
        {
            return Err("invalid Demon source length".into());
        }
    } else if source_len != original_len {
        return Err("raw source/original length mismatch".into());
    }
    let control_len = address_length(header_u64(container, 32))?;
    let literal_len = address_length(header_u64(container, 40))?;
    if control_len as u64 > (source_len as u64) * 2 || literal_len > source_len {
        return Err("uncompressed stream length exceeds source-derived limit".into());
    }
    let control_z_len = address_length(header_u64(container, 48))?;
    let literal_z_len = address_length(header_u64(container, 56))?;
    let split = HEADER_LEN
        .checked_add(control_z_len)
        .ok_or("compressed length overflow")?;
    let end = split
        .checked_add(literal_z_len)
        .ok_or("compressed length overflow")?;
    if end != container.len() {
        return Err("truncated or trailing compressed streams".into());
    }
    let controls = zlib_decode(&container[HEADER_LEN..split], control_len)?;
    let literals = zlib_decode(&container[split..end], literal_len)?;
    let mut source = Vec::new();
    let mut table = vec![EMPTY; SLOTS];
    let k = options.context_bytes as usize;
    let mut cursor = 0usize;
    let mut literal_cursor = 0usize;
    while cursor < controls.len() {
        let token = read_varint(&controls, &mut cursor)?;
        if token & 1 == 0 {
            let length = address_length(token >> 1)?;
            if length == 0
                || length > source_len - source.len()
                || length > literals.len() - literal_cursor
            {
                return Err("invalid literal run length".into());
            }
            source
                .try_reserve(length)
                .map_err(|e| format!("source allocation: {e}"))?;
            for &byte in &literals[literal_cursor..literal_cursor + length] {
                insert_position(&source, source.len(), k, &mut table);
                source.push(byte);
            }
            literal_cursor += length;
        } else {
            let length = (token >> 1)
                .checked_add(u64::from(options.min_match))
                .ok_or("match length overflow")?;
            if length > (source_len - source.len()) as u64 {
                return Err("match length exceeds source length".into());
            }
            let length = address_length(length)?;
            let old = candidate(&source, source.len(), k, &table)
                .ok_or("invalid implicit match candidate or context collision")?;
            source
                .try_reserve(length)
                .map_err(|e| format!("source allocation: {e}"))?;
            for offset in 0..length {
                // old < token_start, so this index is available even when
                // old + offset enters bytes just appended by this same match.
                let byte = source[old + offset];
                insert_position(&source, source.len(), k, &mut table);
                source.push(byte);
            }
        }
    }
    if source.len() != source_len || literal_cursor != literals.len() {
        return Err("source length mismatch or trailing literal bytes".into());
    }
    let output = if options.dc10 {
        preflight_demon(&source, original_len)?;
        hutter_demon::demon_transform_decode(&source)?
    } else {
        source
    };
    if output.len() != original_len {
        return Err("original length mismatch".into());
    }
    if hutter_demon::fnv1a64_bytes(&output) != header_u64(container, 24) {
        return Err("original FNV-1a checksum mismatch".into());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(k: u8, dc10: bool) -> Options {
        Options {
            context_bytes: k,
            min_match: 4,
            dc10,
        }
    }

    fn roundtrip(data: &[u8], opts: Options) -> Encoded {
        let encoded = compress(data, opts).unwrap();
        assert_eq!(decompress(&encoded.bytes, data.len()).unwrap(), data);
        let again = compress(data, opts).unwrap();
        assert_eq!(encoded.bytes, again.bytes);
        assert_eq!(encoded.matches, again.matches);
        assert_eq!(encoded.matched_bytes, again.matched_bytes);
        assert_eq!(encoded.control_bytes, again.control_bytes);
        assert_eq!(encoded.literal_bytes, again.literal_bytes);
        encoded
    }

    fn set_header(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn literal_frame(original: &[u8], source: &[u8], opts: Options) -> Vec<u8> {
        let mut control = Vec::new();
        if !source.is_empty() {
            write_varint((source.len() as u64) << 1, &mut control);
        }
        pack_container(original, source.len(), &control, source, opts).unwrap()
    }

    #[test]
    fn corpus_options_and_determinism() {
        let all_bytes: Vec<u8> = (0u8..=255).collect();
        let utf8_xml = "<page><title>整数 café Ελληνικά 日本語 🦀</title><text xml:space=\"preserve\">[[United States|United States of America]] &amp;nbsp; {{template}} </text></page>\n".repeat(32).into_bytes();
        let mut state = 0x1234_5678u32;
        let random: Vec<u8> = (0..16_384)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                state as u8
            })
            .collect();
        let repeated = b"abcdabcdabcdabcdefghabcdefgh".repeat(300);
        for k in [4, 8] {
            for dc10 in [false, true] {
                for data in [
                    &[][..],
                    &b"x"[..],
                    &all_bytes,
                    &utf8_xml,
                    &random,
                    &repeated,
                ] {
                    roundtrip(data, options(k, dc10));
                }
                for min_match in [4, 7, 16, 255] {
                    roundtrip(
                        &repeated,
                        Options {
                            min_match,
                            ..options(k, dc10)
                        },
                    );
                }
            }
        }
    }

    #[test]
    fn distance_one_overlap_and_per_position_updates() {
        for k in [4, 8] {
            let data = vec![b'a'; 8192];
            let encoded = roundtrip(&data, options(k, false));
            assert_eq!(encoded.matches, 1);
            assert_eq!(encoded.literal_bytes, k as usize + 1);
            assert_eq!(encoded.matched_bytes as usize, data.len() - k as usize - 1);
            // Several later tokens depend on entries inserted inside earlier
            // matches rather than just token-start insertions.
            let varied = b"aaaaabaaaaabaaaaabaaaaacaaaaabaaaaabaaaaabaaaaac".repeat(60);
            assert!(roundtrip(&varied, options(k, false)).matches > 1);
        }
    }

    #[test]
    fn varint_boundaries_and_malformed_encodings() {
        for value in [0, 1, 127, 128, 16_383, 16_384, u32::MAX as u64, u64::MAX] {
            let mut bytes = Vec::new();
            write_varint(value, &mut bytes);
            let mut cursor = 0;
            assert_eq!(read_varint(&bytes, &mut cursor).unwrap(), value);
            assert_eq!(cursor, bytes.len());
        }
        for bytes in [
            vec![],
            vec![0x80],
            vec![0x80, 0],
            vec![0x81, 0],
            vec![0xff; 10],
            [vec![0x80; 9], vec![2]].concat(),
            vec![0x80; 11],
        ] {
            assert!(read_varint(&bytes, &mut 0).is_err());
        }
    }

    #[test]
    fn invalid_tokens_and_stream_lengths() {
        let original = [b'x'; 32];
        for control in [vec![0], vec![1], vec![0x80], vec![0x80, 0], vec![0xff; 10]] {
            let frame = pack_container(&original, original.len(), &control, &[], options(4, false))
                .unwrap();
            assert!(decompress(&frame, 32).is_err());
        }
        // Literal token overreads its stream, leaves trailing literals, or
        // exceeds the declared output. A huge match is rejected before copy.
        for (control, literal) in [
            (vec![64], vec![b'x'; 31]),
            (vec![2], vec![b'x'; 32]),
            (vec![66], vec![b'x'; 32]),
        ] {
            let frame =
                pack_container(&original, 32, &control, &literal, options(4, false)).unwrap();
            assert!(decompress(&frame, 32).is_err());
        }
        let mut huge = Vec::new();
        write_varint(u64::MAX, &mut huge);
        let frame = pack_container(&original, 32, &huge, &[], options(4, false)).unwrap();
        assert!(decompress(&frame, 32).is_err());
        let empty_controls = pack_container(&original, 32, &[], &[], options(4, false)).unwrap();
        assert!(decompress(&empty_controls, 32).is_err());
    }

    #[test]
    fn truncation_corruption_and_header_limits() {
        let data = b"<page>repeat repeat repeat</page>".repeat(20);
        let valid = compress(&data, options(8, true)).unwrap().bytes;
        for end in 0..valid.len() {
            assert!(
                decompress(&valid[..end], data.len()).is_err(),
                "prefix {end}"
            );
        }
        assert!(decompress(&valid, data.len() - 1).is_err());
        for offset in [0, 4, 5, 6, 7, 24] {
            let mut bad = valid.clone();
            bad[offset] = match offset {
                4 => 2,
                5 => 6,
                6 => 3,
                _ => bad[offset] ^ 1,
            };
            assert!(
                decompress(&bad, data.len()).is_err(),
                "header offset {offset}"
            );
        }
        for offset in [8, 16, 32, 40, 48, 56] {
            let mut bad = valid.clone();
            set_header(&mut bad, offset, u64::MAX);
            assert!(
                decompress(&bad, usize::MAX).is_err(),
                "length offset {offset}"
            );
        }
        let mut trailing = valid.clone();
        trailing.push(0);
        assert!(decompress(&trailing, data.len()).is_err());
        for offset in HEADER_LEN..valid.len() {
            let mut bad = valid.clone();
            bad[offset] ^= 0x80;
            // Deflate padding bits may be ignored: a changed representation
            // need not be invalid, but must never silently change the output.
            if let Ok(decoded) = decompress(&bad, data.len()) {
                assert_eq!(decoded, data, "payload offset {offset}");
            }
        }
        let control_end = HEADER_LEN + header_u64(&valid, 48) as usize;
        for offset in [control_end - 1, valid.len() - 1] {
            let mut bad = valid.clone();
            bad[offset] ^= 1; // each member's Adler-32 trailer
            assert!(decompress(&bad, data.len()).is_err());
        }
        let raw = compress(&data, options(4, false)).unwrap().bytes;
        let mut bad = raw.clone();
        set_header(&mut bad, 16, data.len() as u64 - 1);
        assert!(decompress(&bad, data.len()).is_err());
        for opts in [
            options(0, false),
            options(5, false),
            Options {
                min_match: 3,
                ..options(4, false)
            },
        ] {
            assert!(compress(&data, opts).is_err());
        }
    }

    #[test]
    fn bounded_inflate_requires_exact_complete_single_member() {
        let compressed = zlib_encode(&[42; 32_768]).unwrap();
        assert!(zlib_decode(&compressed, 1).is_err());
        assert!(zlib_decode(&compressed, 32_769).is_err());
        assert_eq!(zlib_decode(&compressed, 32_768).unwrap(), vec![42; 32_768]);
        for end in 0..compressed.len() {
            assert!(zlib_decode(&compressed[..end], 32_768).is_err());
        }
        let mut joined = compressed.clone();
        joined.extend_from_slice(&zlib_encode(b"").unwrap());
        assert!(zlib_decode(&joined, 32_768).is_err());
        let mut trailing = compressed;
        trailing.push(0);
        assert!(zlib_decode(&trailing, 32_768).is_err());
        assert_eq!(zlib_decode(&zlib_encode(b"").unwrap(), 0).unwrap(), b"");
    }

    #[test]
    fn per_member_trailing_bytes_and_false_inflate_lengths() {
        let original = b"literal data";
        let valid = literal_frame(original, original, options(4, false));
        let control_z_len = header_u64(&valid, 48) as usize;
        let mut bad = valid.clone();
        bad.insert(HEADER_LEN + control_z_len, 0);
        set_header(&mut bad, 48, control_z_len as u64 + 1);
        assert!(decompress(&bad, original.len()).is_err());
        let mut bad = valid.clone();
        let literal_z_len = header_u64(&valid, 56);
        bad.push(0);
        set_header(&mut bad, 56, literal_z_len + 1);
        assert!(decompress(&bad, original.len()).is_err());
        for offset in [32, 40] {
            let value = header_u64(&valid, offset);
            for replacement in [value - 1, value + 1] {
                let mut bad = valid.clone();
                set_header(&mut bad, offset, replacement);
                assert!(decompress(&bad, original.len()).is_err());
            }
        }
    }

    #[test]
    fn demon_preflight_bounds_and_malformed_bytecode() {
        let xml = b"<page><title>A</title><text xml:space=\"preserve\">[[United States|United States of America]]</text></page>";
        let source = hutter_demon::demon_transform_encode(xml).encoded;
        assert!(preflight_demon(&source, xml.len()).is_ok());
        assert!(preflight_demon(&source, xml.len() - 1).is_err());
        assert!(preflight_demon(&source, xml.len() + 1).is_err());
        for source in [
            vec![0xff],
            vec![0xfe],
            vec![0xfe, 3, b'a'],
            vec![0xfe, 0, b'a', 0, hutter_demon::OP_WIKI_CLOSE],
            vec![0xfe, 0, b'a', 2, hutter_demon::OP_WIKI_CLOSE],
            vec![0xfe, 0, b'a', 1, b'z'],
        ] {
            let frame = literal_frame(b"xxxxxxxxxxxxxxxx", &source, options(4, true));
            assert!(decompress(&frame, 16).is_err());
        }
        let expansion = hutter_demon::STATIC_DEMONS
            .iter()
            .find(|spec| spec.pattern.len() > 4)
            .unwrap();
        let frame = literal_frame(b"x", &[expansion.opcode], options(4, true));
        assert!(decompress(&frame, 1).is_err());
    }

    #[test]
    fn collisions_cannot_be_used_as_implicit_matches() {
        // Pigeonhole: 65,537 distinct 4-byte contexts into 65,536 slots.
        let mut seen = vec![None; SLOTS];
        let mut collision = None;
        for value in 0u32..=65_536 {
            let bytes = value.to_le_bytes();
            let slot = context_slot(&bytes, 4, 4).unwrap();
            if let Some(previous) = seen[slot] {
                collision = Some((previous, value));
                break;
            }
            seen[slot] = Some(value);
        }
        let (a, b) = collision.expect("pigeonhole collision");
        let mut prefix = a.to_le_bytes().to_vec();
        prefix.push(0x5a);
        prefix.extend_from_slice(&b.to_le_bytes());
        let slot = context_slot(&prefix, prefix.len(), 4).unwrap();
        let mut table = vec![EMPTY; SLOTS];
        table[slot] = 4;
        assert_eq!(candidate(&prefix, prefix.len(), 4, &table), None);
        // Install only real earlier positions as a decoder would. If the last
        // colliding entry is overwritten, it still cannot become an equal
        // context: all length-four windows here are checked explicitly.
        let mut table = vec![EMPTY; SLOTS];
        for pos in 0..prefix.len() {
            insert_position(&prefix, pos, 4, &mut table);
        }
        assert_eq!(candidate(&prefix, prefix.len(), 4, &table), None);
        let mut controls = Vec::new();
        write_varint((prefix.len() as u64) << 1, &mut controls);
        write_varint(1, &mut controls); // invalid min-length implicit match
        let mut original = prefix.clone();
        original.extend_from_slice(b"zzzz");
        let invalid = pack_container(
            &original,
            original.len(),
            &controls,
            &prefix,
            options(4, false),
        )
        .unwrap();
        assert!(decompress(&invalid, original.len()).is_err());
        let collision_corpus =
            [prefix.as_slice(), b"payload", prefix.as_slice(), b"payload"].concat();
        roundtrip(&collision_corpus, options(4, false));
        // For 8-byte contexts, equal low^high folds collide by construction.
        let mut eight = vec![0u8; 8];
        eight.push(0x5a);
        eight.extend_from_slice(&[1, 0, 0, 0, 1, 0, 0, 0]);
        let mut table = vec![EMPTY; SLOTS];
        let slot = context_slot(&eight, eight.len(), 8).unwrap();
        table[slot] = 8;
        assert_eq!(candidate(&eight, eight.len(), 8, &table), None);
        roundtrip(&eight.repeat(100), options(8, false));
    }
}
