//! Integer-only, lossless recurring-record templates with exact exceptions.
//!
//! Marine-inspired **MDL-style experimental coding**, not an import of
//! marine-core, production MEM8 retrieval, or a probabilistic reconstruction.
//! The bounded Space-Saving pass supplies approximate candidate frequencies;
//! a second pass measures exact surviving-candidate counts and raw-byte
//! promotion heuristic scores. Scores charge literal definitions, field-stream
//! wrappers and length metadata, but the eight-byte wrapper charge does NOT
//! bound zlib expansion. Scores are neither measured entropy savings nor bounds
//! on final compressed savings. Overlapping candidates can make unused
//! dictionaries costly; no improvement is promised.
//! All bytes (including invalid UTF-8) are preserved without normalization.
//!
//! # STP1 wire format
//!
//! Header integers are little endian. The fixed header is 96 bytes:
//! - 0..4: `STP1` (magic and version).
//! - 4: flags, bit 0 context prediction, bit 1 lattice literal backend.
//! - 5: reserved zero; 6..8: max_record (16..=4096).
//! - 8..10: configured max_templates (0..=256); 10..12: actual count T.
//! - 12..16: min_gain in estimated BYTES.
//! - u64 at 16: original length; 24: original FNV-1a-64 (NOT authentication).
//! - u64 at 32,40,48: raw dictionary, program, literal lengths.
//! - u64 at 56,64,72: stored dictionary, program, literal lengths.
//! - u64 at 80: total raw field payload length; 88: reserved zero.
//!
//! Next are T pairs of u64 (raw field length, stored field length). Then come
//! dictionary, program, literal, and T field streams in that order. Dictionary,
//! program and every field stream (including empty/exact-template streams) are
//! independent complete zlib level-9 members. Literals are either another zlib
//! member or a COMPLETE M8L1 container, including its 64-byte header, produced
//! with context_bytes=4, min_match=16, dc10=true. No trailing bytes/members.
//!
//! Dictionary: for each template in ID order, canonical ULEB128 field count
//! (0..=8), followed by count+1 (ULEB128 length, exact literal bytes) segments.
//! Total fixed bytes per template are 12..=max_record. Then T+1 canonical
//! ULEB128 next-table entries: zero means absent, otherwise template ID+1.
//! Context zero is start/literal; context ID+1 follows that template. With
//! prediction disabled every entry must be zero. No other dictionary bytes.
//!
//! Program tokens are canonical ULEB128: 0 followed by a positive literal byte
//! length; 1 means the current context's preferred next template; ID+2 is an
//! explicit template. A template token is followed by its field lengths, in
//! slot order. Field bytes are consumed from that template's independent
//! payload stream in occurrence/slot order. Reconstruction interleaves fixed
//! segments and exact fields. A template record may not exceed max_record.
//! Literals reset context to zero, and consecutive literal records coalesce.
//! Every raw stream must be consumed exactly, and output length/hash must match.
//!
//! Input records split at LF INCLUDING LF; the final unterminated record is
//! retained. Long records are literal exceptions. Candidates are whole records,
//! ASCII-digit-run masks (1..=8 runs), and a single XML-like middle field
//! (optional leading ASCII whitespace, '<', first '>', final '<'). Pattern
//! identity includes ALL segment boundaries, including empty segments.
//!
//! Original length is at most u32::MAX and isize::MAX. Dictionary, program and
//! payload lengths are checked before inflation/allocation. max_output limits
//! final output, NOT total working memory. Candidate capacity is 4096; at most
//! three distinct patterns are visited per eligible record, without stale heaps.

use crate::mem8_lattice;
use flate2::write::ZlibEncoder;
use flate2::{Compression, Decompress, FlushDecompress, Status};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

const MAGIC: &[u8; 4] = b"STP1";
const HEADER: usize = 96;
const CANDIDATE_CAP: usize = 4096;
const MAX_TEMPLATES: usize = 256;
const MIN_FIXED: usize = 12;

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub max_templates: u16,
    pub max_record: u16,
    /// Minimum raw-byte promotion heuristic in BYTES, not guaranteed final savings.
    pub min_gain: u32,
    pub context: bool,
    pub lattice_literals: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_templates: 256,
            max_record: 512,
            min_gain: 64,
            context: true,
            lattice_literals: false,
        }
    }
}

/// Raw stream sizes, except `bytes`, which includes the ENTIRE container.
/// Estimated gain is eight times the positive raw-byte promotion scores, not
/// observed or guaranteed final compressed savings. All promoted dictionaries are counted,
/// even if another overlapping template wins every occurrence.
#[derive(Debug)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub templates: usize,
    pub template_records: u64,
    pub predicted_records: u64,
    /// Exact exception bytes BEFORE their zlib/M8L1 encoding.
    pub literal_bytes: usize,
    /// Total per-template field payload bytes BEFORE independent zlib encoding.
    pub field_bytes: usize,
    /// Dictionary bytes BEFORE zlib, including definitions and preferred-next
    /// entries. The compressed dictionary and all framing are in `bytes`.
    pub dictionary_bytes: usize,
    pub estimated_gain_bits: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Pattern {
    segments: Vec<Vec<u8>>,
}

impl Pattern {
    fn fields(&self) -> usize {
        self.segments.len() - 1
    }

    fn fixed(&self) -> usize {
        self.segments.iter().map(Vec::len).sum()
    }

    fn definition_size(&self) -> usize {
        varint_len(self.fields() as u64)
            + self
                .segments
                .iter()
                .map(|s| varint_len(s.len() as u64) + s.len())
                .sum::<usize>()
    }
}

struct Candidate<'a> {
    pattern: Pattern,
    fields: Vec<&'a [u8]>,
}

fn options_valid(options: Options) -> Result<(), String> {
    if options.max_templates as usize > MAX_TEMPLATES {
        return Err("max_templates must be 0..=256".into());
    }
    if !(16..=4096).contains(&options.max_record) {
        return Err("max_record must be 16..=4096".into());
    }
    Ok(())
}

fn records(input: &[u8]) -> impl Iterator<Item = &[u8]> {
    input.split_inclusive(|&b| b == b'\n')
}

/// At most three distinct patterns; fields borrow the exact original bytes.
fn candidates(record: &[u8], max_record: usize) -> Vec<Candidate<'_>> {
    if record.len() < MIN_FIXED || record.len() > max_record {
        return Vec::new();
    }
    let mut out = vec![Candidate {
        pattern: Pattern {
            segments: vec![record.to_vec()],
        },
        fields: Vec::new(),
    }];
    let mut segments = Vec::new();
    let mut fields = Vec::new();
    let mut pos = 0;
    let mut start = 0;
    while pos < record.len() {
        if record[pos].is_ascii_digit() {
            segments.push(record[start..pos].to_vec());
            let begin = pos;
            while pos < record.len() && record[pos].is_ascii_digit() {
                pos += 1;
            }
            fields.push(&record[begin..pos]);
            start = pos;
            if fields.len() > 8 {
                break;
            }
        } else {
            pos += 1;
        }
    }
    if !fields.is_empty() && fields.len() <= 8 {
        segments.push(record[start..].to_vec());
        let pattern = Pattern { segments };
        if pattern.fixed() >= MIN_FIXED {
            out.push(Candidate { pattern, fields });
        }
    }
    let first = record.iter().position(|b| !b.is_ascii_whitespace());
    if let Some(first) = first.filter(|&p| record[p] == b'<') {
        if let (Some(close), Some(last)) = (
            record[first..]
                .iter()
                .position(|&b| b == b'>')
                .map(|p| p + first),
            record.iter().rposition(|&b| b == b'<'),
        ) {
            if last > close {
                let pattern = Pattern {
                    segments: vec![record[..close + 1].to_vec(), record[last..].to_vec()],
                };
                if pattern.fixed() >= MIN_FIXED && !out.iter().any(|c| c.pattern == pattern) {
                    out.push(Candidate {
                        pattern,
                        fields: vec![&record[close + 1..last]],
                    });
                }
            }
        }
    }
    out
}

#[derive(Default)]
struct ExactStats {
    count: u64,
    saving: i64,
}

/// Space-Saving estimates are deliberately discarded before scoring survivors.
fn learn(input: &[u8], options: Options) -> Result<Vec<(Pattern, u64)>, String> {
    if options.max_templates == 0 {
        return Ok(Vec::new());
    }
    let mut pool = BTreeMap::<Pattern, u64>::new();
    let mut order = BTreeSet::<(u64, Pattern)>::new();
    for record in records(input) {
        for candidate in candidates(record, options.max_record as usize) {
            let pattern = candidate.pattern;
            let count = if let Some(&old) = pool.get(&pattern) {
                order.remove(&(old, pattern.clone()));
                old.checked_add(1).ok_or("candidate estimate overflow")?
            } else if pool.len() == CANDIDATE_CAP {
                let (old, evicted) = order.pop_first().ok_or("empty candidate ordering")?;
                pool.remove(&evicted);
                old.checked_add(1).ok_or("candidate estimate overflow")?
            } else {
                1
            };
            pool.insert(pattern.clone(), count);
            order.insert((count, pattern));
        }
    }
    let mut exact: BTreeMap<Pattern, ExactStats> = pool
        .into_keys()
        .map(|p| (p, ExactStats::default()))
        .collect();
    for record in records(input) {
        for candidate in candidates(record, options.max_record as usize) {
            if let Some(stats) = exact.get_mut(&candidate.pattern) {
                stats.count = stats.count.checked_add(1).ok_or("exact count overflow")?;
                let overhead: usize = candidate
                    .fields
                    .iter()
                    .map(|f| varint_len(f.len() as u64))
                    .sum();
                let saving = candidate.pattern.fixed() as i64 - 3 - overhead as i64;
                stats.saving = stats
                    .saving
                    .checked_add(saving)
                    .ok_or("candidate score overflow")?;
            }
        }
    }
    let mut ranked = Vec::new();
    for (pattern, stats) in exact {
        // Definition, one field zlib wrapper, pair of u64 lengths, next-table
        // entry (<=2 bytes), plus a fixed dictionary-wrapper charge. These
        // raw-byte heuristic charges do not bound final zlib expansion/savings.
        let charge = pattern.definition_size() + 8 + 16 + 2 + 8;
        let gain = stats.saving - charge as i64;
        if stats.count != 0 && gain > 0 && gain as u64 >= u64::from(options.min_gain) {
            ranked.push((pattern, gain as u64));
        }
    }
    ranked.sort_by(|(pa, ga), (pb, gb)| gb.cmp(ga).then_with(|| pa.cmp(pb)));
    ranked.truncate(options.max_templates as usize);
    Ok(ranked)
}

/// Selection charges the actual explicit token width, before the next table is
/// learned. Prediction can only shorten that token. Lower IDs break score ties.
fn choose<'a>(
    record: &'a [u8],
    options: Options,
    ids: &BTreeMap<Pattern, usize>,
) -> Option<(usize, Vec<&'a [u8]>)> {
    let mut best: Option<(i64, usize, Vec<&'a [u8]>)> = None;
    for candidate in candidates(record, options.max_record as usize) {
        if let Some(&id) = ids.get(&candidate.pattern) {
            let overhead = varint_len((id + 2) as u64)
                + candidate
                    .fields
                    .iter()
                    .map(|f| varint_len(f.len() as u64))
                    .sum::<usize>();
            let score = candidate.pattern.fixed() as i64 - overhead as i64;
            if best
                .as_ref()
                .map_or(true, |(s, old, _)| score > *s || (score == *s && id < *old))
            {
                best = Some((score, id, candidate.fields));
            }
        }
    }
    best.map(|(_, id, fields)| (id, fields))
}

fn preferred_next(
    input: &[u8],
    options: Options,
    ids: &BTreeMap<Pattern, usize>,
) -> Vec<Option<usize>> {
    let n = ids.len();
    let mut next = vec![None; n + 1];
    if !options.context || n == 0 {
        return next;
    }
    // Fixed bound: at most 257 * 256 counters, exact chosen transitions only.
    let mut counts = vec![0u64; (n + 1) * n];
    let mut state = 0;
    for record in records(input) {
        if let Some((id, _)) = choose(record, options, ids) {
            // At most u32::MAX records, so this u64 increment cannot overflow.
            counts[state * n + id] += 1;
            state = id + 1;
        } else {
            state = 0;
        }
    }
    for state in 0..=n {
        let mut best = 0;
        for id in 0..n {
            let count = counts[state * n + id];
            if count > best {
                best = count;
                next[state] = Some(id);
            }
        }
    }
    next
}

fn dictionary_encode(patterns: &[Pattern], next: &[Option<usize>]) -> Vec<u8> {
    let mut out = Vec::new();
    for pattern in patterns {
        write_varint(pattern.fields() as u64, &mut out);
        for segment in &pattern.segments {
            write_varint(segment.len() as u64, &mut out);
            out.extend_from_slice(segment);
        }
    }
    for id in next {
        write_varint(id.map_or(0, |id| id as u64 + 1), &mut out);
    }
    out
}

/// Lossless bounded template learning plus exact literal exceptions.
pub fn compress(input: &[u8], options: Options) -> Result<Encoded, String> {
    options_valid(options)?;
    source_length(input.len() as u64)?;
    let learned = learn(input, options)?;
    let estimated_gain_bits = learned.iter().try_fold(0u64, |sum, (_, gain)| {
        gain.checked_mul(8)
            .and_then(|bits| sum.checked_add(bits))
            .ok_or("estimated score overflow")
    })?;
    let patterns: Vec<Pattern> = learned.into_iter().map(|(p, _)| p).collect();
    let ids: BTreeMap<Pattern, usize> = patterns
        .iter()
        .cloned()
        .enumerate()
        .map(|(id, p)| (p, id))
        .collect();
    let next = preferred_next(input, options, &ids);
    let dictionary = dictionary_encode(&patterns, &next);
    let mut program = Vec::new();
    let mut literals = Vec::new();
    let mut fields = vec![Vec::new(); patterns.len()];
    let mut run = 0usize;
    let mut state = 0;
    let mut template_records = 0u64;
    let mut predicted_records = 0u64;
    for record in records(input) {
        let selected = if ids.is_empty() {
            None
        } else {
            choose(record, options, &ids)
        };
        if let Some((id, values)) = selected {
            if run != 0 {
                write_varint(0, &mut program);
                write_varint(run as u64, &mut program);
                run = 0;
            }
            if options.context && next[state] == Some(id) {
                write_varint(1, &mut program);
                predicted_records += 1;
            } else {
                write_varint(id as u64 + 2, &mut program);
            }
            for value in values {
                write_varint(value.len() as u64, &mut program);
                fields[id].extend_from_slice(value);
            }
            template_records += 1;
            state = id + 1;
        } else {
            literals.extend_from_slice(record);
            run += record.len(); // Bounded by validated input length.
            state = 0;
        }
    }
    if run != 0 {
        write_varint(0, &mut program);
        write_varint(run as u64, &mut program);
    }
    let field_bytes = fields.iter().map(Vec::len).sum();
    let dictionary_z = zlib_encode(&dictionary)?;
    let program_z = zlib_encode(&program)?;
    let literal_z = if options.lattice_literals {
        mem8_lattice::compress(
            &literals,
            mem8_lattice::Options {
                context_bytes: 4,
                min_match: 16,
                dc10: true,
            },
        )?
        .bytes
    } else {
        zlib_encode(&literals)?
    };
    let field_z: Vec<Vec<u8>> = fields
        .iter()
        .map(|f| zlib_encode(f))
        .collect::<Result<_, _>>()?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.push(u8::from(options.context) | (u8::from(options.lattice_literals) << 1));
    bytes.push(0);
    bytes.extend_from_slice(&options.max_record.to_le_bytes());
    bytes.extend_from_slice(&options.max_templates.to_le_bytes());
    bytes.extend_from_slice(&(patterns.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&options.min_gain.to_le_bytes());
    for value in [
        input.len() as u64,
        fnv(input),
        dictionary.len() as u64,
        program.len() as u64,
        literals.len() as u64,
        dictionary_z.len() as u64,
        program_z.len() as u64,
        literal_z.len() as u64,
        field_bytes as u64,
        0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for (raw, stored) in fields.iter().zip(&field_z) {
        bytes.extend_from_slice(&(raw.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&(stored.len() as u64).to_le_bytes());
    }
    bytes.extend_from_slice(&dictionary_z);
    bytes.extend_from_slice(&program_z);
    bytes.extend_from_slice(&literal_z);
    for stream in field_z {
        bytes.extend_from_slice(&stream);
    }
    Ok(Encoded {
        bytes,
        templates: patterns.len(),
        template_records,
        predicted_records,
        literal_bytes: literals.len(),
        field_bytes,
        dictionary_bytes: dictionary.len(),
        estimated_gain_bits,
    })
}

fn source_length(value: u64) -> Result<usize, String> {
    if value > u32::MAX as u64 || value > isize::MAX as u64 {
        return Err("original length exceeds u32/address limit".into());
    }
    Ok(value as usize)
}

fn address_length(value: u64) -> Result<usize, String> {
    if value > isize::MAX as u64 {
        return Err("stream length exceeds address limit".into());
    }
    Ok(value as usize)
}

fn header_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn dictionary_decode(
    bytes: &[u8],
    count: usize,
    options: Options,
) -> Result<(Vec<Pattern>, Vec<Option<usize>>), String> {
    let mut cursor = 0;
    let mut patterns = Vec::new();
    let mut seen = BTreeSet::new();
    for _ in 0..count {
        let fields = read_varint(bytes, &mut cursor)?;
        if fields > 8 {
            return Err("template has too many fields".into());
        }
        let mut segments = Vec::new();
        let mut fixed = 0usize;
        for _ in 0..=fields {
            let len = address_length(read_varint(bytes, &mut cursor)?)?;
            if len > options.max_record as usize - fixed || len > bytes.len() - cursor {
                return Err("invalid template literal length".into());
            }
            fixed += len;
            segments.push(bytes[cursor..cursor + len].to_vec());
            cursor += len;
        }
        if fixed < MIN_FIXED {
            return Err("template has insufficient fixed bytes".into());
        }
        let pattern = Pattern { segments };
        if !seen.insert(pattern.clone()) {
            return Err("duplicate template pattern".into());
        }
        patterns.push(pattern);
    }
    let mut next = Vec::new();
    for _ in 0..=count {
        let id = read_varint(bytes, &mut cursor)?;
        if id > count as u64 || (!options.context && id != 0) {
            return Err("invalid preferred-next entry".into());
        }
        next.push(if id == 0 { None } else { Some(id as usize - 1) });
    }
    if cursor != bytes.len() {
        return Err("trailing dictionary bytes".into());
    }
    Ok((patterns, next))
}

/// Decode with a final-output limit, not a whole-memory budget. All header
/// lengths are validated before raw-stream allocations, and all independently
/// compressed members and raw streams must end exactly.
pub fn decompress(container: &[u8], max_output: usize) -> Result<Vec<u8>, String> {
    if container.len() < HEADER || &container[..4] != MAGIC {
        return Err("invalid or truncated STP1 header".into());
    }
    if container[4] & !3 != 0 || container[5] != 0 || header_u64(container, 88) != 0 {
        return Err("unknown STP1 flags/reserved bytes".into());
    }
    let options = Options {
        context: container[4] & 1 != 0,
        lattice_literals: container[4] & 2 != 0,
        max_record: u16::from_le_bytes(container[6..8].try_into().unwrap()),
        max_templates: u16::from_le_bytes(container[8..10].try_into().unwrap()),
        min_gain: u32::from_le_bytes(container[12..16].try_into().unwrap()),
    };
    options_valid(options)?;
    let count = u16::from_le_bytes(container[10..12].try_into().unwrap()) as usize;
    if count > options.max_templates as usize {
        return Err("template count exceeds configured limit".into());
    }
    let original = source_length(header_u64(container, 16))?;
    if original > max_output {
        return Err("original exceeds output limit".into());
    }
    let dictionary_len = address_length(header_u64(container, 32))?;
    let program_len = address_length(header_u64(container, 40))?;
    let literal_len = address_length(header_u64(container, 48))?;
    let dictionary_stored = address_length(header_u64(container, 56))?;
    let program_stored = address_length(header_u64(container, 64))?;
    let literal_stored = address_length(header_u64(container, 72))?;
    let total_fields = address_length(header_u64(container, 80))?;
    // 9 segment length varints (<=2 bytes each), a field count, next entry,
    // and generous fixed slack. Count and max_record have already been bounded.
    let dictionary_bound = count * (options.max_record as usize + 32) + 2;
    // Bounds are arithmetic limits, not allocation sizes. Keep them wide even
    // when a valid, addressable 32-bit source has a bound above usize::MAX.
    let program_bound = original as u64 * 3; // original <= u32::MAX.
    if dictionary_len > dictionary_bound
        || program_len as u64 > program_bound
        || literal_len > original
        || total_fields > original - literal_len
    {
        return Err("raw stream length exceeds source-derived bound".into());
    }
    stored_bound(dictionary_len, dictionary_stored, false)?;
    stored_bound(program_len, program_stored, false)?;
    stored_bound(literal_len, literal_stored, options.lattice_literals)?;
    let table_end = HEADER
        .checked_add(count * 16)
        .ok_or("field table overflow")?;
    if table_end > container.len() {
        return Err("truncated field table".into());
    }
    let mut field_sizes = Vec::new();
    let mut sum_fields = 0usize;
    let mut end = table_end;
    for stored in [dictionary_stored, program_stored, literal_stored] {
        end = end.checked_add(stored).ok_or("stored length overflow")?;
    }
    for id in 0..count {
        let raw = address_length(header_u64(container, HEADER + id * 16))?;
        let stored = address_length(header_u64(container, HEADER + id * 16 + 8))?;
        if raw > total_fields - sum_fields {
            return Err("field length exceeds total payload".into());
        }
        sum_fields += raw;
        stored_bound(raw, stored, false)?;
        end = end
            .checked_add(stored)
            .ok_or("stored field length overflow")?;
        field_sizes.push((raw, stored));
    }
    if sum_fields != total_fields || end != container.len() {
        return Err("container length/payload total mismatch or trailing bytes".into());
    }
    let mut cursor = table_end;
    let dictionary = zlib_decode(
        take(container, &mut cursor, dictionary_stored)?,
        dictionary_len,
    )?;
    let (patterns, next) = dictionary_decode(&dictionary, count, options)?;
    for (pattern, &(raw, _)) in patterns.iter().zip(&field_sizes) {
        if pattern.fields() == 0 && raw != 0 {
            return Err("exact template has field payload".into());
        }
    }
    let program = zlib_decode(take(container, &mut cursor, program_stored)?, program_len)?;
    let stored_literals = take(container, &mut cursor, literal_stored)?;
    let literals = if options.lattice_literals {
        if stored_literals.len() < 64
            || &stored_literals[..4] != b"M8L1"
            || stored_literals[4..8] != [1, 4, 16, 0]
        {
            return Err("literal backend is not the specified complete M8L1 container".into());
        }
        let decoded = mem8_lattice::decompress(stored_literals, literal_len)?;
        if decoded.len() != literal_len {
            return Err("lattice literal length mismatch".into());
        }
        decoded
    } else {
        zlib_decode(stored_literals, literal_len)?
    };
    let mut fields = Vec::new();
    for &(raw, stored) in &field_sizes {
        fields.push(zlib_decode(take(container, &mut cursor, stored)?, raw)?);
    }
    let mut field_cursors = vec![0usize; count];
    let mut literal_cursor = 0;
    let mut pc = 0;
    let mut state = 0;
    let mut out = Vec::new();
    while pc < program.len() {
        let token = read_varint(&program, &mut pc)?;
        if token == 0 {
            let len = address_length(read_varint(&program, &mut pc)?)?;
            if len == 0 || len > literals.len() - literal_cursor || len > original - out.len() {
                return Err("invalid literal run".into());
            }
            append(&mut out, &literals[literal_cursor..literal_cursor + len])?;
            literal_cursor += len;
            state = 0;
            continue;
        }
        let id = if token == 1 {
            if !options.context {
                return Err("predicted token with context disabled".into());
            }
            next[state].ok_or("predicted token has no preferred template")?
        } else {
            if token - 2 >= count as u64 {
                return Err("invalid explicit template ID".into());
            }
            (token - 2) as usize
        };
        let pattern = &patterns[id];
        let mut lengths = Vec::new();
        let mut record_len = pattern.fixed();
        let mut field_len = 0usize;
        for _ in 0..pattern.fields() {
            let len = address_length(read_varint(&program, &mut pc)?)?;
            if len > options.max_record as usize - record_len {
                return Err("template record exceeds max_record".into());
            }
            record_len += len;
            field_len += len;
            lengths.push(len);
        }
        if record_len > original - out.len() || field_len > fields[id].len() - field_cursors[id] {
            return Err("template expansion exceeds output or payload".into());
        }
        for (slot, segment) in pattern.segments.iter().enumerate() {
            append(&mut out, segment)?;
            if let Some(&len) = lengths.get(slot) {
                let begin = field_cursors[id];
                append(&mut out, &fields[id][begin..begin + len])?;
                field_cursors[id] += len;
            }
        }
        state = id + 1;
    }
    if cursor != container.len()
        || literal_cursor != literals.len()
        || field_cursors
            .iter()
            .zip(&fields)
            .any(|(&used, stream)| used != stream.len())
        || out.len() != original
    {
        return Err("unconsumed streams or original length mismatch".into());
    }
    if fnv(&out) != header_u64(container, 24) {
        return Err("original checksum mismatch (FNV is not authentication)".into());
    }
    Ok(out)
}

fn take<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8], String> {
    if len > bytes.len() - *cursor {
        return Err("truncated stream".into());
    }
    let begin = *cursor;
    *cursor += len;
    Ok(&bytes[begin..*cursor])
}

fn append(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), String> {
    out.try_reserve(bytes.len())
        .map_err(|e| format!("output allocation: {e}"))?;
    out.extend_from_slice(bytes);
    Ok(())
}

fn stored_bound(raw: usize, stored: usize, lattice: bool) -> Result<(), String> {
    // Deliberately generous bounds, valid for our own zlib encoder. The lattice
    // branch includes the full 64-byte M8L1 header and its two zlib wrappers,
    // DC10's <=2x source expansion and source-derived M8L1 control lengths.
    let (factor, slack) = if lattice { (16u64, 512u64) } else { (2, 64) };
    // Actual stream sizes still pass address_length; a theoretical upper bound
    // need not fit usize on a 32-bit decoder.
    let bound = (raw as u64)
        .checked_mul(factor)
        .and_then(|n| n.checked_add(slack))
        .ok_or("stored stream bound overflow")?;
    if stored as u64 > bound {
        return Err("stored stream exceeds source-derived bound".into());
    }
    Ok(())
}

fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x100000001b3)
    })
}

fn varint_len(mut value: u64) -> usize {
    let mut len = 1;
    while value >= 128 {
        len += 1;
        value >>= 7;
    }
    len
}

/// G8T discovery: hutter_uleb128_encode_u64, equivalence class
/// (u64)->u64::398a2f2d8912, author aye::Hue+Gemini(0+1)::x86_64.
/// Metadata only (150837/50000 cycles/op); NO capsule source reuse. The local
/// joint is a conventional canonical ULEB128 scalar-to-byte-stream writer.
fn write_varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        out.push(byte | if value != 0 { 0x80 } else { 0 });
        if value == 0 {
            return;
        }
    }
}

fn read_varint(bytes: &[u8], cursor: &mut usize) -> Result<u64, String> {
    let mut value = 0u64;
    for index in 0..10 {
        let byte = *bytes.get(*cursor).ok_or("truncated ULEB128")?;
        *cursor += 1;
        if index == 9 && byte > 1 {
            return Err("overflowing ULEB128".into());
        }
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            if index != 0 && byte == 0 {
                return Err("noncanonical ULEB128".into());
            }
            return Ok(value);
        }
    }
    Err("overlong ULEB128".into())
}

fn zlib_encode(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    encoder
        .write_all(bytes)
        .map_err(|e| format!("zlib encode: {e}"))?;
    encoder.finish().map_err(|e| format!("zlib finish: {e}"))
}

/// Adapted locally from mem8_lattice::zlib_decode, which is private; it is not
/// modified. Low-level inflate proves StreamEnd and exact member consumption,
/// unlike Read wrappers that may mask a missing trailer. Growth is bounded by
/// declared+1, never a header-sized initial reservation.
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
        append(&mut out, &chunk[..produced])?;
        if out.len() > declared {
            return Err("zlib output exceeds declared length".into());
        }
        if status == Status::StreamEnd {
            if decoder.total_in() != bytes.len() as u64 {
                return Err("trailing bytes/member in zlib stream".into());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> Options {
        Options {
            min_gain: 0,
            ..Options::default()
        }
    }

    fn roundtrip(input: &[u8], options: Options) -> Encoded {
        let encoded = compress(input, options).unwrap();
        assert_eq!(decompress(&encoded.bytes, input.len()).unwrap(), input);
        assert_eq!(compress(input, options).unwrap().bytes, encoded.bytes);
        if !input.is_empty() {
            assert!(decompress(&encoded.bytes, input.len() - 1).is_err());
        }
        encoded
    }

    #[test]
    fn arbitrary_bytes_and_option_matrix() {
        let all: Vec<u8> = (0..=255).collect();
        let mut mixed = Vec::new();
        for i in 0..80 {
            mixed.extend_from_slice(
                format!("  <long_element>日本語 & < > value-{i}</long_element>\n").as_bytes(),
            );
            mixed.extend_from_slice(
                format!("integer-counter={i}; second=000123; stable suffix\n").as_bytes(),
            );
            mixed.extend_from_slice(b"fixed repeated whole record line\n");
            mixed.extend_from_slice(b"  <binary_item>\xff\x00<>\xfe</binary_item>\n");
        }
        mixed.extend_from_slice(&all);
        mixed.extend_from_slice(b"last record without newline");
        let mut long = vec![b'X'; 20_000];
        long.push(b'\n');
        long.extend_from_slice(b"unterminated");
        for context in [false, true] {
            for lattice_literals in [false, true] {
                let options = Options {
                    context,
                    lattice_literals,
                    ..opts()
                };
                for input in [&[][..], &all, &mixed, &long, b"a\nb\n\n", b"no newline"] {
                    roundtrip(input, options);
                }
                let control = roundtrip(
                    &mixed,
                    Options {
                        max_templates: 0,
                        ..options
                    },
                );
                assert_eq!(control.templates, 0);
                assert_eq!(control.literal_bytes, mixed.len());
                assert_eq!(control.field_bytes, 0);
                assert_eq!(control.template_records, 0);
            }
        }
    }

    #[test]
    fn selected_patterns_predictions_and_exact_fields() {
        let mut input = Vec::new();
        for i in 0..200 {
            input.extend_from_slice(
                format!("  <varying_element>\u{03bb} {i} & text</varying_element>\n").as_bytes(),
            );
            input.extend_from_slice(
                format!("counter-number={i:05}; constant literal trailer\n").as_bytes(),
            );
            input.extend_from_slice(b"repeat whole unchanged record\n");
        }
        let on = roundtrip(&input, opts());
        assert!(on.templates >= 3);
        assert!(on.template_records > 500);
        assert!(on.predicted_records > 400);
        assert!(on.field_bytes > 0);
        assert!(on.estimated_gain_bits > 0);
        let off = roundtrip(
            &input,
            Options {
                context: false,
                ..opts()
            },
        );
        assert_eq!(off.predicted_records, 0);
        assert_eq!(on.template_records, off.template_records);
        let repeat = roundtrip(&b"repeat whole unchanged record\n".repeat(100), opts());
        assert!(repeat.templates > 0);
        assert_eq!(repeat.field_bytes, 0);
        assert_eq!(repeat.literal_bytes, 0);
        assert_eq!(repeat.template_records, 100);
        assert_eq!(repeat.predicted_records, 100);
    }

    #[test]
    fn candidate_identity_and_binary_middle() {
        let p = Pattern {
            segments: vec![b"abcdefghijkl".to_vec()],
        };
        let q = Pattern {
            segments: vec![b"abcdef".to_vec(), b"ghijkl".to_vec()],
        };
        assert_ne!(p, q); // Same concatenated fixed bytes, different slot identity.
        let candidates = candidates(b"  <long_name>\xff\x00<>\xfe</long_name>\n", 512);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[1].fields, vec![&b"\xff\x00<>\xfe"[..]]);
        assert_eq!(candidates[1].pattern.fields(), 1);
        let digits = super::candidates(b"prefix-fixed=123; trailing-fixed\n", 512);
        assert_eq!(digits.len(), 2);
        assert_eq!(digits[1].fields, vec![&b"123"[..]]);
        let all_fixed = super::candidates(b"fixed whole record bytes\n", 512);
        assert_eq!(all_fixed.len(), 1);
        assert_eq!(
            all_fixed[0].pattern.segments,
            vec![b"fixed whole record bytes\n".to_vec()]
        );
        // Literal LF splitting never swallows, invents or normalizes a delimiter.
        roundtrip(
            &b"  <long_name>\xff\n\x00<>\xfe</long_name>\n".repeat(30),
            opts(),
        );
        assert!(super::candidates(b"12345678901234567890\n", 512).len() == 1);
        assert!(
            super::candidates(b"fixed-prefix 1 2 3 4 5 6 7 8 9 fixed-suffix\n", 512).len() == 1
        );
    }

    fn set64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    /// Build a small, independently specified frame to test the grammar rather
    /// than relying solely on the encoder to produce decoder inputs.
    fn frame(
        original: &[u8],
        patterns: &[Pattern],
        next: &[Option<usize>],
        program: &[u8],
        literals: &[u8],
        fields: &[Vec<u8>],
        context: bool,
    ) -> Vec<u8> {
        let dictionary = dictionary_encode(patterns, next);
        let streams: Vec<Vec<u8>> = [&dictionary[..], program, literals]
            .into_iter()
            .chain(fields.iter().map(Vec::as_slice))
            .map(|s| zlib_encode(s).unwrap())
            .collect();
        let mut bytes = vec![0; HEADER];
        bytes[..4].copy_from_slice(MAGIC);
        bytes[4] = u8::from(context);
        bytes[6..8].copy_from_slice(&512u16.to_le_bytes());
        bytes[8..10].copy_from_slice(&256u16.to_le_bytes());
        bytes[10..12].copy_from_slice(&(patterns.len() as u16).to_le_bytes());
        for (offset, value) in [
            (16, original.len()),
            (32, dictionary.len()),
            (40, program.len()),
            (48, literals.len()),
            (56, streams[0].len()),
            (64, streams[1].len()),
            (72, streams[2].len()),
            (80, fields.iter().map(Vec::len).sum()),
        ] {
            set64(&mut bytes, offset, value as u64);
        }
        set64(&mut bytes, 24, fnv(original));
        for (raw, compressed) in fields.iter().zip(&streams[3..]) {
            bytes.extend_from_slice(&(raw.len() as u64).to_le_bytes());
            bytes.extend_from_slice(&(compressed.len() as u64).to_le_bytes());
        }
        for stream in streams {
            bytes.extend_from_slice(&stream);
        }
        bytes
    }

    #[test]
    fn malformed_tokens_dictionary_and_consumption() {
        let p = Pattern {
            segments: vec![b"abcdefghijkl".to_vec()],
        };
        let valid = frame(
            b"abcdefghijkl",
            &[p.clone()],
            &[Some(0), None],
            &[1],
            b"",
            &[vec![]],
            true,
        );
        assert_eq!(decompress(&valid, 12).unwrap(), b"abcdefghijkl");
        for (program, next, context) in [
            (vec![1], vec![None, None], true),
            (vec![3], vec![None, None], true),
            (vec![0, 0], vec![None, None], true),
            (vec![0x82, 0], vec![None, None], true),
            (vec![1], vec![None, None], false),
            (vec![2, 2], vec![None, None], true),
            (vec![0x80], vec![None, None], true),
            (vec![0xff; 10], vec![None, None], true),
        ] {
            let bytes = frame(
                b"abcdefghijkl",
                &[p.clone()],
                &next,
                &program,
                b"",
                &[vec![]],
                context,
            );
            assert!(decompress(&bytes, 12).is_err(), "program {program:?}");
        }
        let bad_next = frame(
            b"abcdefghijkl",
            &[p.clone()],
            &[Some(1), None],
            &[2],
            b"",
            &[vec![]],
            true,
        );
        assert!(decompress(&bad_next, 12).is_err());
        let short = Pattern {
            segments: vec![b"tiny".to_vec()],
        };
        assert!(decompress(
            &frame(b"tiny", &[short], &[None, None], &[2], b"", &[vec![]], true),
            4
        )
        .is_err());
        let unused = frame(
            b"abcdefghijkl",
            &[p.clone()],
            &[None, None],
            &[2],
            b"!",
            &[vec![]],
            true,
        );
        assert!(decompress(&unused, 12).is_err());
        let unused_field = frame(
            b"abcdefghijkl",
            &[p.clone()],
            &[None, None],
            &[2],
            b"",
            &[vec![1]],
            true,
        );
        assert!(decompress(&unused_field, 12).is_err());
        let multi = Pattern {
            segments: vec![Vec::new(); 10],
        };
        assert!(decompress(
            &frame(b"x", &[multi], &[None, None], &[2], b"x", &[vec![]], true),
            1
        )
        .is_err());
        let duplicate = frame(
            b"abcdefghijkl",
            &[p.clone(), p],
            &[None; 3],
            &[2],
            b"",
            &[vec![], vec![]],
            true,
        );
        assert!(decompress(&duplicate, 12).is_err());
        // An exact binary field can contain LF, '<', NUL and invalid UTF-8;
        // reconstruction is purely length-delimited, never split by payload.
        let p = Pattern {
            segments: vec![b"<long_name>".to_vec(), b"</long_name>\n".to_vec()],
        };
        let field = b"\xff\x00\n<>\xfe";
        let mut original = b"<long_name>".to_vec();
        original.extend_from_slice(field);
        original.extend_from_slice(b"</long_name>\n");
        let bytes = frame(
            &original,
            &[p.clone()],
            &[None; 2],
            &[2, field.len() as u8],
            b"",
            &[field.to_vec()],
            false,
        );
        assert_eq!(decompress(&bytes, original.len()).unwrap(), original);
        let oversized = frame(
            &original,
            &[p],
            &[None; 2],
            &[2, 0xff, 0x7f],
            b"",
            &[field.to_vec()],
            false,
        );
        assert!(decompress(&oversized, original.len()).is_err());
    }

    #[test]
    fn header_limits_truncation_checksum_and_trailing_bytes() {
        let encoded = compress(
            b"arbitrary binary \xff\x00\n",
            Options {
                max_templates: 0,
                ..opts()
            },
        )
        .unwrap();
        for len in 0..encoded.bytes.len() {
            assert!(
                decompress(&encoded.bytes[..len], 1000).is_err(),
                "accepted prefix length {len}"
            );
        }
        for (offset, value) in [
            (16, u64::MAX),
            (32, 100_000),
            (40, 100_000),
            (48, 100_000),
            (56, u64::MAX),
            (64, u64::MAX),
            (72, u64::MAX),
            (80, 100_000),
            (88, 1),
        ] {
            let mut bytes = encoded.bytes.clone();
            set64(&mut bytes, offset, value);
            assert!(
                decompress(&bytes, 1000).is_err(),
                "accepted header offset {offset}"
            );
        }
        for (offset, value) in [(6, 15u16), (6, 4097), (8, 257), (10, 257)] {
            let mut bytes = encoded.bytes.clone();
            bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
            assert!(decompress(&bytes, 1000).is_err());
        }
        for offset in [0, 4, 5, 10, 24] {
            let mut bytes = encoded.bytes.clone();
            bytes[offset] ^= 0xff;
            assert!(
                decompress(&bytes, 1000).is_err(),
                "accepted corrupt byte {offset}"
            );
        }
        let mut trailing = encoded.bytes.clone();
        trailing.push(0);
        assert!(decompress(&trailing, 1000).is_err());
        assert!(compress(
            b"",
            Options {
                max_templates: 257,
                ..opts()
            }
        )
        .is_err());
        assert!(compress(
            b"",
            Options {
                max_record: 15,
                ..opts()
            }
        )
        .is_err());
        assert!(compress(
            b"",
            Options {
                max_record: 4097,
                ..opts()
            }
        )
        .is_err());
        let mut checksum = encoded.bytes.clone();
        checksum[24] ^= 1;
        assert!(decompress(&checksum, 1000)
            .unwrap_err()
            .contains("checksum"));
    }

    #[test]
    fn strict_inflate_member_ends_and_lengths() {
        let raw = b"some zlib member bytes\xff\x00";
        let encoded = zlib_encode(raw).unwrap();
        assert_eq!(zlib_decode(&encoded, raw.len()).unwrap(), raw);
        for cut in 0..encoded.len() {
            assert!(zlib_decode(&encoded[..cut], raw.len()).is_err());
        }
        let mut concatenated = encoded.clone();
        concatenated.extend_from_slice(&zlib_encode(b"").unwrap());
        assert!(zlib_decode(&concatenated, raw.len()).is_err());
        let mut junk = encoded.clone();
        junk.push(0);
        assert!(zlib_decode(&junk, raw.len()).is_err());
        assert!(zlib_decode(&encoded, raw.len() - 1).is_err());
        assert!(zlib_decode(&encoded, raw.len() + 1).is_err());
        assert!(zlib_decode(&zlib_encode(&vec![0; 100_000]).unwrap(), 1).is_err());
        assert_eq!(zlib_decode(&zlib_encode(b"").unwrap(), 0).unwrap(), b"");
    }

    #[test]
    fn saturated_space_saving_uses_exact_survivor_scores() {
        let mut input = Vec::new();
        for i in 0..CANDIDATE_CAP + 200 {
            input.extend_from_slice(b"unique long candidate record ");
            let mut value = i;
            for _ in 0..4 {
                input.push(b'a' + (value % 26) as u8);
                value /= 26;
            }
            input.push(b'\n');
        }
        let hot = b"frequently recurring fixed record without digits\n";
        input.extend_from_slice(&hot.repeat(100));
        let learned = learn(&input, opts()).unwrap();
        // Replacement estimates for once-seen survivors exceed one, but exact
        // recounting must not promote them on those approximate frequencies.
        assert_eq!(learned.len(), 1);
        assert_eq!(learned[0].0.segments, vec![hot.to_vec()]);
        let encoded = roundtrip(&input, opts());
        assert_eq!(encoded.template_records, 100);
        assert_eq!(encoded.templates, 1);
        for max_record in [16, 4096] {
            let record = b"fixed-line-text\n";
            roundtrip(
                &record.repeat(80),
                Options {
                    max_record,
                    ..opts()
                },
            );
        }
    }

    #[test]
    fn xml_fields_are_selected_without_digit_masks_or_utf8_assumptions() {
        let mut input = Vec::new();
        for i in 0..200 {
            input.extend_from_slice(b"  <long_element>");
            input.extend_from_slice(&[0xff, 0, b'<', b'>', 128 + (i % 128) as u8]);
            input.extend_from_slice(b"</long_element>\n");
        }
        let learned = learn(&input, opts()).unwrap();
        assert_eq!(learned.len(), 1);
        assert_eq!(learned[0].0.fields(), 1);
        assert_eq!(
            learned[0].0.segments,
            vec![b"  <long_element>".to_vec(), b"</long_element>\n".to_vec()]
        );
        let encoded = roundtrip(&input, opts());
        assert_eq!(encoded.template_records, 200);
        assert_eq!(encoded.field_bytes, 200 * 5);
        assert_eq!(encoded.literal_bytes, 0);
        assert_eq!(encoded.predicted_records, 200);

        // Equal concatenated fixed literals do not make an exact template and
        // a one-slot template the same dictionary pattern, even for empty fields.
        let exact = Pattern {
            segments: vec![b"abcdefghijkl".to_vec()],
        };
        let slotted = Pattern {
            segments: vec![b"abcdef".to_vec(), b"ghijkl".to_vec()],
        };
        let bytes = frame(
            b"abcdefghijklabcdefghijkl",
            &[exact, slotted],
            &[None; 3],
            &[2, 3, 0],
            b"",
            &[vec![], vec![]],
            false,
        );
        assert_eq!(decompress(&bytes, 24).unwrap(), b"abcdefghijklabcdefghijkl");
    }

    #[test]
    fn malformed_field_frames_and_bounded_mutations_do_not_panic() {
        let input = (0..80)
            .map(|i| format!("number field={i}; long stable literal trailer\n"))
            .collect::<String>()
            .into_bytes();
        let encoded = compress(&input, opts()).unwrap();
        assert!(encoded.field_bytes > 0);
        for cut in 0..encoded.bytes.len() {
            assert!(decompress(&encoded.bytes[..cut], input.len()).is_err());
        }
        // Every byte position, including all stream sizes and field metadata.
        // A mutation may leave a semantically equivalent header; success is OK,
        // but no mutated frame may panic or escape the requested output bound.
        for pos in 0..encoded.bytes.len() {
            let mut bytes = encoded.bytes.clone();
            bytes[pos] ^= 0xa5;
            if let Ok(output) = decompress(&bytes, input.len()) {
                assert!(output.len() <= input.len());
            }
        }
        let mut nested = compress(
            b"binary literal \xff\x00",
            Options {
                max_templates: 0,
                lattice_literals: true,
                ..opts()
            },
        )
        .unwrap()
        .bytes;
        let inner = HEADER + header_u64(&nested, 56) as usize + header_u64(&nested, 64) as usize;
        nested[inner + 5] = 8; // Valid M8L1 option, but not the STP1 backend contract.
        assert!(decompress(&nested, 1000).is_err());
    }

    #[test]
    fn theoretical_bounds_may_exceed_32_bit_address_space() {
        // All actual sizes fit a 32-bit isize. Their theoretical bounds need
        // not fit even a 32-bit usize, and must not cause false rejection.
        assert!(stored_bound(1_000_000_000, 1_500_000_000, true).is_ok());
        assert!(stored_bound(i32::MAX as usize, i32::MAX as usize, false).is_ok());
        assert!(stored_bound(0, 513, true).is_err());
        assert!(stored_bound(0, 65, false).is_err());

        // Exercise program-bound preflight without allocating the advertised
        // output. An empty program ultimately fails output-length validation,
        // not source_bound multiplication, on both 32- and 64-bit targets.
        let mut bytes = compress(
            b"",
            Options {
                max_templates: 0,
                ..opts()
            },
        )
        .unwrap()
        .bytes;
        set64(&mut bytes, 16, 1_500_000_000);
        let error = decompress(&bytes, 1_500_000_000).unwrap_err();
        assert!(
            error.contains("unconsumed streams or original length mismatch"),
            "{error}"
        );
    }

    #[test]
    fn varint_boundaries_are_canonical() {
        for value in [0, 1, 127, 128, 16383, 16384, u32::MAX as u64, u64::MAX] {
            let mut bytes = Vec::new();
            write_varint(value, &mut bytes);
            assert_eq!(bytes.len(), varint_len(value));
            let mut cursor = 0;
            assert_eq!(read_varint(&bytes, &mut cursor).unwrap(), value);
            assert_eq!(cursor, bytes.len());
        }
        for bytes in [vec![0x80, 0], vec![0x81, 0], vec![0xff; 10], vec![0x80; 10]] {
            assert!(read_varint(&bytes, &mut 0).is_err());
        }
    }
}
