//! # Hutter Demon Preprocessor & Level-0 Bit-Perfect Container (`DC10`)
//!
//! 100% integer/rational (`p/q`) lossless preprocessing and entropy compression engine
//! engineered for the Hutter Prize (`enwik8` / `enwik9` MediaWiki XML dumps).
//!
//! ## Protocol (`DC10`)
//! - Literal ASCII bytes `< 0x80` pass through unchanged.
//! - Any literal byte `>= 0x80` in the original input is losslessly escaped as `[0xFF, byte]`.
//! - Opcodes `0x80..=0xFD` encode high-frequency MediaWiki & XML structural Demons:
//!   1. **XML Envelope & Article Boundary Demons** (`<page>`, `<revision>`, `<timestamp>`, `<contributor>`, `<text xml:space="preserve">`, etc.)
//!   2. **MediaWiki Structural & Category Demons** (`[[Category:`, `[[Image:`, interwiki links, headings, `[[`, `]]`, `{{`, `}}`, `'''`, `''`)
//!   3. **XML Double-Escaped Entity Demons** (`&amp;nbsp;`, `&amp;mdash;`, `&amp;ndash;`, `&lt;ref&gt;`, `&quot;`, `&amp;`, `&lt;`, `&gt;`, `http://www.`)
//! - Opcode `0xFE` (`OP_PIPE_PREFIX`) encodes the **Pipe-Link Prefix Deduplication Demon**
//!   for `[[Target|Display]]` links where `Target` (`4..=64` ASCII bytes) and `Display` share
//!   a case-insensitive 1st-byte + exact remaining ASCII prefix of length `p >= 4`
//!   (e.g. `[[United States|United States of America]]`, `[[computer|Computers]]`,
//!   `[[Chicago, Illinois|Chicago]]`).
//!
//! ## Zero-Float Arithmetic Contract
//! All ratios, basis points, millipercents, speeds, and Landauer thermodynamic metrics
//! use exact `u64`/`u128` integer and reduced rational `p/q` (`gcd_u128`) arithmetic.

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// Escape byte for literal input bytes `>= 0x80`.
pub const OP_ESCAPE: u8 = 0xFF;

/// Opcode for the Pipe-Link Prefix Deduplication Demon (`[[Target|Display]]`).
pub const OP_PIPE_PREFIX: u8 = 0xFE;

/// Container magic bytes for the Hutter Demon `DC10` container.
pub const DC10_MAGIC: &[u8; 4] = b"DC10";

/// Container header byte length (`magic[4] + flags[1] + orig_len[8] + trans_len[8] + fnv1a64[8]`).
pub const DC10_HEADER_LEN: usize = 29;

/// Exact reduced rational `p/q` (`num / den`) with `den > 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational128 {
    pub num: u128,
    pub den: u128,
}

impl Rational128 {
    /// Construct a normalized rational `num / den` reduced by `gcd_u128`.
    /// If `den == 0`, returns `0 / 1`.
    pub const fn new(num: u128, den: u128) -> Self {
        if den == 0 {
            return Self { num: 0, den: 1 };
        }
        let g = gcd_u128(num, den);
        Self {
            num: num / g,
            den: den / g,
        }
    }
}

impl std::fmt::Display for Rational128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

/// Greatest common divisor over `u128` using iterative Euclidean algorithm.
pub const fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    if a == 0 {
        1
    } else {
        a
    }
}

/// Exact 64-bit FNV-1a checksum over a byte slice.
pub const fn fnv1a64_bytes(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001B3;
    let mut h = FNV_OFFSET;
    let mut i = 0;
    while i < bytes.len() {
        h ^= bytes[i] as u64;
        h = h.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    h
}

/// Demon structural category for telemetry and scoreboard reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemonCategory {
    XmlEnvelope,
    WikiLinkCategory,
    XmlEntity,
    PipeLinkStem,
}

/// Static Demon entry mapping an opcode in `0x80..=0xFD` to its byte pattern and category.
#[derive(Debug, Clone, Copy)]
pub struct StaticDemonSpec {
    pub opcode: u8,
    pub category: DemonCategory,
    pub pattern: &'static [u8],
}

/// Complete table of static Hutter Demons (`0x80..`).
/// Note: `OP_WIKI_CLOSE` is the opcode assigned to `b"]]"` and is also used as the
/// ASCII-safe terminator for `OP_PIPE_PREFIX` display suffixes.
pub const STATIC_DEMONS: &[StaticDemonSpec] = &[
    // --- Category 1: XML Envelope & Article Boundary Demons ---
    StaticDemonSpec {
        opcode: 0x80,
        category: DemonCategory::XmlEnvelope,
        pattern: b"  <page>\n    <title>",
    },
    StaticDemonSpec {
        opcode: 0x81,
        category: DemonCategory::XmlEnvelope,
        pattern: b"<page>\n    <title>",
    },
    StaticDemonSpec {
        opcode: 0x82,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</title>\n    <id>",
    },
    StaticDemonSpec {
        opcode: 0x83,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n    <revision>\n      <id>",
    },
    StaticDemonSpec {
        opcode: 0x84,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n      <timestamp>2006-",
    },
    StaticDemonSpec {
        opcode: 0x85,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n      <timestamp>2005-",
    },
    StaticDemonSpec {
        opcode: 0x86,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n      <timestamp>200",
    },
    StaticDemonSpec {
        opcode: 0x87,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</timestamp>\n      <contributor>\n        <username>",
    },
    StaticDemonSpec {
        opcode: 0x88,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</timestamp>\n      <contributor>\n        <ip>",
    },
    StaticDemonSpec {
        opcode: 0x89,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</username>\n        <id>",
    },
    StaticDemonSpec {
        opcode: 0x8A,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n      </contributor>\n      <minor />\n      <comment>",
    },
    StaticDemonSpec {
        opcode: 0x8B,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n      </contributor>\n      <comment>",
    },
    StaticDemonSpec {
        opcode: 0x8C,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n      </contributor>\n      <text xml:space=\"preserve\">",
    },
    StaticDemonSpec {
        opcode: 0x8D,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n      </contributor>\n",
    },
    StaticDemonSpec {
        opcode: 0x8E,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</id>\n      </contributor>",
    },
    StaticDemonSpec {
        opcode: 0x8F,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</ip>\n      </contributor>\n",
    },
    StaticDemonSpec {
        opcode: 0x90,
        category: DemonCategory::XmlEnvelope,
        pattern: b"      <minor />\n",
    },
    StaticDemonSpec {
        opcode: 0x91,
        category: DemonCategory::XmlEnvelope,
        pattern: b"<minor />\n",
    },
    StaticDemonSpec {
        opcode: 0x92,
        category: DemonCategory::XmlEnvelope,
        pattern: b"      <comment>",
    },
    StaticDemonSpec {
        opcode: 0x93,
        category: DemonCategory::XmlEnvelope,
        pattern: b"<comment>",
    },
    StaticDemonSpec {
        opcode: 0x94,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</comment>\n      <text xml:space=\"preserve\">",
    },
    StaticDemonSpec {
        opcode: 0x95,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</comment>\n",
    },
    StaticDemonSpec {
        opcode: 0x96,
        category: DemonCategory::XmlEnvelope,
        pattern: b"      <text xml:space=\"preserve\">",
    },
    StaticDemonSpec {
        opcode: 0x97,
        category: DemonCategory::XmlEnvelope,
        pattern: b"<text xml:space=\"preserve\">",
    },
    StaticDemonSpec {
        opcode: 0x98,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</text>\n    </revision>\n  </page>\n",
    },
    StaticDemonSpec {
        opcode: 0x99,
        category: DemonCategory::XmlEnvelope,
        pattern: b"</text>\n    </revision>\n  </page>",
    },
    // --- Category 2: MediaWiki Structural & Category Demons ---
    StaticDemonSpec {
        opcode: 0x9A,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"#REDIRECT [[",
    },
    StaticDemonSpec {
        opcode: 0x9B,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[Category:",
    },
    StaticDemonSpec {
        opcode: 0x9C,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[Image:",
    },
    StaticDemonSpec {
        opcode: 0x9D,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[de:",
    },
    StaticDemonSpec {
        opcode: 0x9E,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[fr:",
    },
    StaticDemonSpec {
        opcode: 0x9F,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[es:",
    },
    StaticDemonSpec {
        opcode: 0xA0,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[ja:",
    },
    StaticDemonSpec {
        opcode: 0xA1,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[nl:",
    },
    StaticDemonSpec {
        opcode: 0xA2,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[pl:",
    },
    StaticDemonSpec {
        opcode: 0xA3,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[pt:",
    },
    StaticDemonSpec {
        opcode: 0xA4,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[sv:",
    },
    StaticDemonSpec {
        opcode: 0xA5,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[it:",
    },
    StaticDemonSpec {
        opcode: 0xA6,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[ru:",
    },
    StaticDemonSpec {
        opcode: 0xA7,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[zh:",
    },
    StaticDemonSpec {
        opcode: 0xA8,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"== See also ==\n",
    },
    StaticDemonSpec {
        opcode: 0xA9,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"== See also ==",
    },
    StaticDemonSpec {
        opcode: 0xAA,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"== References ==\n",
    },
    StaticDemonSpec {
        opcode: 0xAB,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"== References ==",
    },
    StaticDemonSpec {
        opcode: 0xAC,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"== External links ==\n",
    },
    StaticDemonSpec {
        opcode: 0xAD,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"== External links ==",
    },
    StaticDemonSpec {
        opcode: 0xAE,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"=== ",
    },
    StaticDemonSpec {
        opcode: 0xAF,
        category: DemonCategory::WikiLinkCategory,
        pattern: b" ===\n",
    },
    StaticDemonSpec {
        opcode: 0xB0,
        category: DemonCategory::WikiLinkCategory,
        pattern: b" ===",
    },
    StaticDemonSpec {
        opcode: 0xB1,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"== ",
    },
    StaticDemonSpec {
        opcode: 0xB2,
        category: DemonCategory::WikiLinkCategory,
        pattern: b" ==\n",
    },
    StaticDemonSpec {
        opcode: 0xB3,
        category: DemonCategory::WikiLinkCategory,
        pattern: b" ==",
    },
    StaticDemonSpec {
        opcode: 0xB4,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"\n* [[",
    },
    StaticDemonSpec {
        opcode: 0xB5,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"* [[",
    },
    StaticDemonSpec {
        opcode: 0xB6,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"[[",
    },
    StaticDemonSpec {
        opcode: 0xB7,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"]]",
    },
    StaticDemonSpec {
        opcode: 0xB8,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"{{",
    },
    StaticDemonSpec {
        opcode: 0xB9,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"}}",
    },
    StaticDemonSpec {
        opcode: 0xBA,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"'''",
    },
    StaticDemonSpec {
        opcode: 0xBB,
        category: DemonCategory::WikiLinkCategory,
        pattern: b"''",
    },
    // --- Category 3: XML Double-Escaped Entity Demons ---
    StaticDemonSpec {
        opcode: 0xBC,
        category: DemonCategory::XmlEntity,
        pattern: b"&amp;nbsp;",
    },
    StaticDemonSpec {
        opcode: 0xBD,
        category: DemonCategory::XmlEntity,
        pattern: b"&amp;mdash;",
    },
    StaticDemonSpec {
        opcode: 0xBE,
        category: DemonCategory::XmlEntity,
        pattern: b"&amp;ndash;",
    },
    StaticDemonSpec {
        opcode: 0xBF,
        category: DemonCategory::XmlEntity,
        pattern: b"&amp;quot;",
    },
    StaticDemonSpec {
        opcode: 0xC0,
        category: DemonCategory::XmlEntity,
        pattern: b"&amp;amp;",
    },
    StaticDemonSpec {
        opcode: 0xC1,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;ref&gt;",
    },
    StaticDemonSpec {
        opcode: 0xC2,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;/ref&gt;",
    },
    StaticDemonSpec {
        opcode: 0xC3,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;ref name=",
    },
    StaticDemonSpec {
        opcode: 0xC4,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;br&gt;",
    },
    StaticDemonSpec {
        opcode: 0xC5,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;br /&gt;",
    },
    StaticDemonSpec {
        opcode: 0xC6,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;table",
    },
    StaticDemonSpec {
        opcode: 0xC7,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;/table&gt;",
    },
    StaticDemonSpec {
        opcode: 0xC8,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;tr&gt;",
    },
    StaticDemonSpec {
        opcode: 0xC9,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;/tr&gt;",
    },
    StaticDemonSpec {
        opcode: 0xCA,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;td&gt;",
    },
    StaticDemonSpec {
        opcode: 0xCB,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;/td&gt;",
    },
    StaticDemonSpec {
        opcode: 0xCC,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;div",
    },
    StaticDemonSpec {
        opcode: 0xCD,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;/div&gt;",
    },
    StaticDemonSpec {
        opcode: 0xCE,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;small&gt;",
    },
    StaticDemonSpec {
        opcode: 0xCF,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;/small&gt;",
    },
    StaticDemonSpec {
        opcode: 0xD0,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;sup&gt;",
    },
    StaticDemonSpec {
        opcode: 0xD1,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;/sup&gt;",
    },
    StaticDemonSpec {
        opcode: 0xD2,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;!--",
    },
    StaticDemonSpec {
        opcode: 0xD3,
        category: DemonCategory::XmlEntity,
        pattern: b"--&gt;",
    },
    StaticDemonSpec {
        opcode: 0xD4,
        category: DemonCategory::XmlEntity,
        pattern: b"style=&quot;",
    },
    StaticDemonSpec {
        opcode: 0xD5,
        category: DemonCategory::XmlEntity,
        pattern: b"align=&quot;",
    },
    StaticDemonSpec {
        opcode: 0xD6,
        category: DemonCategory::XmlEntity,
        pattern: b"http://www.",
    },
    StaticDemonSpec {
        opcode: 0xD7,
        category: DemonCategory::XmlEntity,
        pattern: b"http://",
    },
    StaticDemonSpec {
        opcode: 0xD8,
        category: DemonCategory::XmlEntity,
        pattern: b"&quot;",
    },
    StaticDemonSpec {
        opcode: 0xD9,
        category: DemonCategory::XmlEntity,
        pattern: b"&amp;",
    },
    StaticDemonSpec {
        opcode: 0xDA,
        category: DemonCategory::XmlEntity,
        pattern: b"&lt;",
    },
    StaticDemonSpec {
        opcode: 0xDB,
        category: DemonCategory::XmlEntity,
        pattern: b"&gt;",
    },
];

/// Opcode for `b"]]"` in `STATIC_DEMONS`, also serving as terminator for `OP_PIPE_PREFIX`.
pub const OP_WIKI_CLOSE: u8 = 0xB7;

/// Per-category invocation and byte-saving statistics from `demon_transform_encode`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DemonCategoryStats {
    pub invocations: u64,
    pub bytes_saved: u64,
}

/// Complete output of `demon_transform_encode`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemonTransformOutput {
    pub encoded: Vec<u8>,
    pub original_len: u64,
    pub transformed_len: u64,
    pub xml_envelope: DemonCategoryStats,
    pub wikilink_category: DemonCategoryStats,
    pub pipe_link_stem: DemonCategoryStats,
    pub xml_entity: DemonCategoryStats,
    pub escaped_high_bytes: u64,
}

impl DemonTransformOutput {
    /// Net bytes shaved by the Demon pre-pass before entropy coding (`original_len - transformed_len`).
    pub fn net_bytes_saved(&self) -> i64 {
        (self.original_len as i64) - (self.transformed_len as i64)
    }

    /// Gross bytes saved by all Demon invocations before subtracting `escaped_high_bytes`.
    pub fn gross_demon_bytes_saved(&self) -> u64 {
        self.xml_envelope.bytes_saved
            + self.wikilink_category.bytes_saved
            + self.pipe_link_stem.bytes_saved
            + self.xml_entity.bytes_saved
    }

    /// Total Demon invocations across all 4 categories.
    pub fn total_invocations(&self) -> u64 {
        self.xml_envelope.invocations
            + self.wikilink_category.invocations
            + self.pipe_link_stem.invocations
            + self.xml_entity.invocations
    }

    /// Exact reduced rational `transformed_len / original_len`.
    pub fn compression_ratio(&self) -> Rational128 {
        Rational128::new(self.transformed_len as u128, self.original_len as u128)
    }

    /// Exact reduced rational `net_bytes_saved / original_len` (0/1 if net <= 0).
    pub fn savings_ratio(&self) -> Rational128 {
        let saved = self.original_len.saturating_sub(self.transformed_len);
        Rational128::new(saved as u128, self.original_len as u128)
    }
}

/// Try to match and encode a `[[Target|Display]]` Pipe-Link Prefix Deduplication Demon
/// at the start of `slice` (where `slice` begins with `b"[["`).
///
/// Format emitted:
/// `[OP_PIPE_PREFIX, flags_and_len, ...target_bytes..., prefix_len, ...display_suffix_bytes..., OP_WIKI_CLOSE]`
/// - `flags_and_len`: bit 6 (`0x40`) = `case_flip` (`Display[0] == Target[0] ^ 0x20`),
///   bits `0..=5` (`0x3F`) = `target.len() - 1` (for `target.len() in 4..=64`).
/// - `prefix_len`: shared prefix length `p` (`4..=target.len()`).
/// - `display_suffix_bytes`: `Display[p..]` (all `< 0x80` printable ASCII).
/// - `OP_WIKI_CLOSE`: `0xB7` terminator.
///
/// Returns `Some((consumed_input_bytes, saved_bytes))` and appends to `out` on match.
#[inline]
fn try_encode_pipe_link(slice: &[u8], out: &mut Vec<u8>) -> Option<(usize, u64)> {
    if slice.len() < 13 || slice[0] != b'[' || slice[1] != b'[' {
        return None;
    }

    // Do not intercept [[Category:... or [[Image:...
    if slice[2..].starts_with(b"Category:") || slice[2..].starts_with(b"Image:") {
        return None;
    }

    // Scan Target up to 64 bytes for '|'
    let max_target_scan = slice.len().min(2 + 65);
    let mut pipe_idx = None;
    let mut i = 2;
    while i < max_target_scan {
        let b = slice[i];
        if b == b'|' {
            pipe_idx = Some(i);
            break;
        }
        // Disallow control chars, non-ASCII, nested brackets, or braces
        if b < 0x20 || b >= 0x80 || b == b'[' || b == b']' || b == b'{' || b == b'}' {
            return None;
        }
        i += 1;
    }

    let pipe_pos = pipe_idx?;
    let target = &slice[2..pipe_pos];
    let target_len = target.len();
    if !(4..=64).contains(&target_len) {
        return None;
    }

    // Scan Display up to 127 bytes for "]]"
    let display_start = pipe_pos + 1;
    let max_display_scan = slice.len().min(display_start + 128);
    let mut close_idx = None;
    let mut j = display_start;
    while j + 1 < max_display_scan {
        let b = slice[j];
        if b == b']' && slice[j + 1] == b']' {
            close_idx = Some(j);
            break;
        }
        if b < 0x20 || b >= 0x80 || b == b'[' || b == b']' || b == b'|' || b == b'{' || b == b'}'
        {
            return None;
        }
        j += 1;
    }

    let close_pos = close_idx?;
    let display = &slice[display_start..close_pos];
    if display.len() < 4 {
        return None;
    }

    // Check first byte match (exact or ASCII letter case flip)
    let t0 = target[0];
    let d0 = display[0];
    let case_flip: u8 = if t0 == d0 {
        0
    } else if t0.is_ascii_alphabetic() && (t0 ^ 0x20) == d0 {
        1
    } else {
        return None;
    };

    // Count exact remaining shared prefix bytes
    let max_prefix = target_len.min(display.len());
    let mut prefix_len = 1usize;
    while prefix_len < max_prefix && target[prefix_len] == display[prefix_len] {
        prefix_len += 1;
    }

    if prefix_len < 4 {
        return None;
    }

    let suffix = &display[prefix_len..];
    let consumed = close_pos + 2; // includes closing "]]"
    let encoded_len = 1 + 1 + target_len + 1 + suffix.len() + 1;
    if encoded_len >= consumed {
        return None;
    }

    let flags_and_len = (case_flip << 6) | ((target_len as u8) - 1);
    out.push(OP_PIPE_PREFIX);
    out.push(flags_and_len);
    out.extend_from_slice(target);
    out.push(prefix_len as u8);
    out.extend_from_slice(suffix);
    out.push(OP_WIKI_CLOSE);

    let saved = (consumed - encoded_len) as u64;
    Some((consumed, saved))
}

/// Build a 256-entry lookup table mapping each possible starting byte `0x00..=0x7F`
/// to the indices of `STATIC_DEMONS` that begin with that byte, sorted by descending
/// pattern length (longest match first).
fn build_first_byte_index() -> [Vec<usize>; 128] {
    let mut table: [Vec<usize>; 128] = std::array::from_fn(|_| Vec::new());
    for (idx, spec) in STATIC_DEMONS.iter().enumerate() {
        let first = spec.pattern[0];
        if first < 128 {
            table[first as usize].push(idx);
        }
    }
    for bucket in &mut table {
        bucket.sort_by(|&a, &b| {
            STATIC_DEMONS[b]
                .pattern
                .len()
                .cmp(&STATIC_DEMONS[a].pattern.len())
        });
    }
    table
}

/// Encode `input` with the Level-0 Bit-Perfect Hutter Demon Preprocessor (`DC10`).
pub fn demon_transform_encode(input: &[u8]) -> DemonTransformOutput {
    let first_byte_index = build_first_byte_index();
    let mut encoded = Vec::with_capacity(input.len());

    let mut xml_envelope = DemonCategoryStats::default();
    let mut wikilink_category = DemonCategoryStats::default();
    let mut pipe_link_stem = DemonCategoryStats::default();
    let mut xml_entity = DemonCategoryStats::default();
    let mut escaped_high_bytes = 0u64;

    let len = input.len();
    let mut pos = 0usize;

    while pos < len {
        let b = input[pos];

        // 1. Escape any literal byte >= 0x80
        if b >= 0x80 {
            encoded.push(OP_ESCAPE);
            encoded.push(b);
            escaped_high_bytes += 1;
            pos += 1;
            continue;
        }

        // 2. Check Pipe-Link Prefix Deduplication Demon on "[["
        if b == b'[' && pos + 1 < len && input[pos + 1] == b'[' {
            if let Some((consumed, saved)) = try_encode_pipe_link(&input[pos..], &mut encoded) {
                pipe_link_stem.invocations += 1;
                pipe_link_stem.bytes_saved += saved;
                pos += consumed;
                continue;
            }
        }

        // 3. Check static Demon table via first-byte bucket (sorted longest-first)
        let candidates = &first_byte_index[b as usize];
        let mut matched = false;
        if !candidates.is_empty() {
            let rem = len - pos;
            for &spec_idx in candidates {
                let spec = &STATIC_DEMONS[spec_idx];
                let pat = spec.pattern;
                let pat_len = pat.len();
                if rem >= pat_len
                    && input[pos + 1] == pat[1]
                    && &input[pos..pos + pat_len] == pat
                {
                    // If a multi-byte prefix pattern ends with "[[", defer to OP_PIPE_PREFIX
                    // at the "[[" boundary if a Pipe-Link Stem Demon applies there.
                    if pat_len > 2 && pat.ends_with(b"[[") {
                        let bracket_pos = pos + pat_len - 2;
                        let mut probe = Vec::new();
                        if try_encode_pipe_link(&input[bracket_pos..], &mut probe).is_some() {
                            continue;
                        }
                    }
                    encoded.push(spec.opcode);
                    let saved = (pat_len - 1) as u64;
                    match spec.category {
                        DemonCategory::XmlEnvelope => {
                            xml_envelope.invocations += 1;
                            xml_envelope.bytes_saved += saved;
                        }
                        DemonCategory::WikiLinkCategory => {
                            wikilink_category.invocations += 1;
                            wikilink_category.bytes_saved += saved;
                        }
                        DemonCategory::XmlEntity => {
                            xml_entity.invocations += 1;
                            xml_entity.bytes_saved += saved;
                        }
                        DemonCategory::PipeLinkStem => {
                            pipe_link_stem.invocations += 1;
                            pipe_link_stem.bytes_saved += saved;
                        }
                    }
                    pos += pat_len;
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            encoded.push(b);
            pos += 1;
        }
    }

    let transformed_len = encoded.len() as u64;
    DemonTransformOutput {
        encoded,
        original_len: len as u64,
        transformed_len,
        xml_envelope,
        wikilink_category,
        pipe_link_stem,
        xml_entity,
        escaped_high_bytes,
    }
}

/// Decode a `DC10` Demon-transformed byte slice back into the exact original bytes.
pub fn demon_transform_decode(encoded: &[u8]) -> Result<Vec<u8>, String> {
    // Build O(1) opcode decode lookup table for 0x80..=0xFD
    let mut decode_table: [Option<&'static [u8]>; 126] = [None; 126];
    for spec in STATIC_DEMONS {
        let idx = (spec.opcode - 0x80) as usize;
        if idx < 126 {
            decode_table[idx] = Some(spec.pattern);
        }
    }

    let mut out = Vec::with_capacity(encoded.len() + (encoded.len() / 8));
    let len = encoded.len();
    let mut pos = 0usize;

    while pos < len {
        let b = encoded[pos];
        if b < 0x80 {
            out.push(b);
            pos += 1;
        } else if b == OP_ESCAPE {
            if pos + 1 >= len {
                return Err(format!("Truncated OP_ESCAPE at byte offset {}", pos));
            }
            out.push(encoded[pos + 1]);
            pos += 2;
        } else if b == OP_PIPE_PREFIX {
            if pos + 2 >= len {
                return Err(format!("Truncated OP_PIPE_PREFIX header at offset {}", pos));
            }
            let flags_and_len = encoded[pos + 1];
            let case_flip = (flags_and_len >> 6) & 1;
            let target_len = ((flags_and_len & 0x3F) as usize) + 1;
            let target_start = pos + 2;
            let target_end = target_start + target_len;
            if target_end >= len {
                return Err(format!(
                    "Truncated OP_PIPE_PREFIX target (len {}) at offset {}",
                    target_len, pos
                ));
            }
            let target = &encoded[target_start..target_end];
            let prefix_len = encoded[target_end] as usize;
            if prefix_len == 0 || prefix_len > target_len {
                return Err(format!(
                    "Invalid OP_PIPE_PREFIX prefix_len {} (target_len {}) at offset {}",
                    prefix_len, target_len, pos
                ));
            }
            let suffix_start = target_end + 1;
            let mut close_pos = suffix_start;
            while close_pos < len && encoded[close_pos] != OP_WIKI_CLOSE {
                close_pos += 1;
            }
            if close_pos >= len {
                return Err(format!(
                    "Unterminated OP_PIPE_PREFIX suffix starting at offset {}",
                    suffix_start
                ));
            }
            let suffix = &encoded[suffix_start..close_pos];

            out.extend_from_slice(b"[[");
            out.extend_from_slice(target);
            out.push(b'|');
            let d0 = if case_flip != 0 {
                target[0] ^ 0x20
            } else {
                target[0]
            };
            out.push(d0);
            out.extend_from_slice(&target[1..prefix_len]);
            out.extend_from_slice(suffix);
            out.extend_from_slice(b"]]");

            pos = close_pos + 1;
        } else {
            let idx = (b - 0x80) as usize;
            if let Some(Some(pat)) = decode_table.get(idx) {
                out.extend_from_slice(pat);
                pos += 1;
            } else {
                return Err(format!("Unknown Demon opcode 0x{:02X} at offset {}", b, pos));
            }
        }
    }

    Ok(out)
}

/// Compress `input` into a self-verifying `DC10` container (`DC10` Demon transform + Zlib/Deflate).
///
/// Header layout (29 bytes):
/// - `0..4`: `b"DC10"`
/// - `4`: `flags` (`0x01` = Zlib-compressed payload, `0x00` = uncompressed Demon bytecode)
/// - `5..13`: `original_len` (`u64` little-endian)
/// - `13..21`: `transformed_len` (`u64` little-endian)
/// - `21..29`: `fnv1a64` checksum of `input` (`u64` little-endian)
/// - `29..`: compressed payload
pub fn compress_dc10_container(input: &[u8], deflate_level: u32) -> Vec<u8> {
    let transform = demon_transform_encode(input);
    let checksum = fnv1a64_bytes(input);
    let orig_len = input.len() as u64;
    let trans_len = transform.encoded.len() as u64;

    let (flags, payload) = if deflate_level == 0 {
        (0x00u8, transform.encoded)
    } else {
        let level = Compression::new(deflate_level.min(9));
        let mut encoder = ZlibEncoder::new(Vec::new(), level);
        encoder
            .write_all(&transform.encoded)
            .expect("ZlibEncoder write cannot fail on Vec<u8>");
        let compressed = encoder
            .finish()
            .expect("ZlibEncoder finish cannot fail on Vec<u8>");
        (0x01u8, compressed)
    };

    let mut container = Vec::with_capacity(DC10_HEADER_LEN + payload.len());
    container.extend_from_slice(DC10_MAGIC);
    container.push(flags);
    container.extend_from_slice(&orig_len.to_le_bytes());
    container.extend_from_slice(&trans_len.to_le_bytes());
    container.extend_from_slice(&checksum.to_le_bytes());
    container.extend_from_slice(&payload);
    container
}

/// Decompress a `DC10` container and verify exact length + 64-bit FNV-1a checksum.
pub fn decompress_dc10_container(container: &[u8]) -> Result<Vec<u8>, String> {
    if container.len() < DC10_HEADER_LEN {
        return Err(format!(
            "Container too short: {} bytes (minimum {})",
            container.len(),
            DC10_HEADER_LEN
        ));
    }
    if &container[0..4] != DC10_MAGIC {
        return Err("Invalid container magic (expected DC10)".to_string());
    }
    let flags = container[4];
    let orig_len = u64::from_le_bytes(container[5..13].try_into().unwrap());
    let trans_len = u64::from_le_bytes(container[13..21].try_into().unwrap());
    let expected_checksum = u64::from_le_bytes(container[21..29].try_into().unwrap());
    let payload = &container[DC10_HEADER_LEN..];

    let transformed = match flags {
        0x00 => payload.to_vec(),
        0x01 => {
            let mut decoder = ZlibDecoder::new(payload);
            let mut buf = Vec::with_capacity(trans_len as usize);
            decoder
                .read_to_end(&mut buf)
                .map_err(|e| format!("Zlib decompression failed: {}", e))?;
            buf
        }
        other => return Err(format!("Unsupported DC10 container flags: 0x{:02X}", other)),
    };

    if transformed.len() as u64 != trans_len {
        return Err(format!(
            "Transformed length mismatch: got {}, expected {}",
            transformed.len(),
            trans_len
        ));
    }

    let reconstructed = demon_transform_decode(&transformed)?;
    if reconstructed.len() as u64 != orig_len {
        return Err(format!(
            "Reconstructed length mismatch: got {}, expected {}",
            reconstructed.len(),
            orig_len
        ));
    }

    let actual_checksum = fnv1a64_bytes(&reconstructed);
    if actual_checksum != expected_checksum {
        return Err(format!(
            "FNV-1a checksum mismatch: got 0x{:016X}, expected 0x{:016X}",
            actual_checksum, expected_checksum
        ));
    }

    Ok(reconstructed)
}

/// Compress raw bytes with baseline Zlib/Deflate at `deflate_level` for direct arena comparison.
pub fn compress_baseline_deflate(input: &[u8], deflate_level: u32) -> Vec<u8> {
    let level = Compression::new(deflate_level.min(9));
    let mut encoder = ZlibEncoder::new(Vec::new(), level);
    encoder
        .write_all(input)
        .expect("ZlibEncoder write cannot fail on Vec<u8>");
    encoder
        .finish()
        .expect("ZlibEncoder finish cannot fail on Vec<u8>")
}

/// Landauer thermodynamic energy extracted for `bits_saved` at `T = 293.15 K`:
/// `E_bit = k_B * T * ln(2) ≈ 2,805,425 / 1,000,000,000` zeptojoules (`10^-21 J`),
/// or equivalently `bits_saved * 2_805_425 / 1_000_000_000` in zeptojoule (`zJ`) units,
/// and `bits_saved * 2_805_425 / 1_000_000_000_000_000_000` in nanojoules/picojoules.
/// Returns exact reduced rational `Rational128` in **picojoules (`10^-12 J`)**:
/// `pJ = bits_saved * 2_805_425 / 1_000_000_000_000_000_000` and in **zeptojoules (`10^-21 J`)**:
/// `zJ = bits_saved * 2_805_425 / 1_000_000_000`.
pub const fn landauer_zeptojoules_rational(bits_saved: u128) -> Rational128 {
    Rational128::new(bits_saved * 2_805_425, 1_000_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_link_prefix_demon_roundtrip() {
        let sample = b"See [[United States|United States of America]], [[computer|Computers]], and [[Chicago, Illinois|Chicago]] for details.";
        let res = demon_transform_encode(sample);
        assert_eq!(res.pipe_link_stem.invocations, 3);
        assert!(res.pipe_link_stem.bytes_saved >= 25);
        assert!(res.transformed_len < res.original_len);

        let decoded = demon_transform_decode(&res.encoded).unwrap();
        assert_eq!(decoded.as_slice(), sample);
    }

    #[test]
    fn test_xml_and_entities_roundtrip_with_utf8() {
        let sample = concat!(
            "  <page>\n    <title>Anarchism</title>\n    <id>12</id>\n",
            "    <revision>\n      <id>18063769</id>\n      <timestamp>2006-02-25T15:43:11Z</timestamp>\n",
            "      <contributor>\n        <username>Docu</username>\n        <id>8029</id>\n      </contributor>\n",
            "      <minor />\n      <comment>Test &amp;nbsp; &quot;quote&quot;</comment>\n",
            "      <text xml:space=\"preserve\">#REDIRECT [[Anarchism|Anarchist philosophy]]\n",
            "[[Category:Political philosophy]]\n[[ja:アナキズム]]\n[[ru:Анархизм]]\n",
            "== See also ==\n* [[Libertarianism]]\n=== History ===\n",
            "&lt;ref&gt;http://www.example.org&lt;/ref&gt;&amp;mdash;done.</text>\n    </revision>\n  </page>\n"
        );
        let input = sample.as_bytes();
        let res = demon_transform_encode(input);
        assert!(res.xml_envelope.invocations >= 6);
        assert!(res.wikilink_category.invocations >= 6);
        assert!(res.pipe_link_stem.invocations >= 1);
        assert!(res.xml_entity.invocations >= 5);
        assert!(res.escaped_high_bytes > 0);
        assert!(res.transformed_len < res.original_len);

        let decoded = demon_transform_decode(&res.encoded).unwrap();
        assert_eq!(decoded.as_slice(), input);

        let container = compress_dc10_container(input, 6);
        let unpacked = decompress_dc10_container(&container).unwrap();
        assert_eq!(unpacked.as_slice(), input);
    }

    #[test]
    fn test_all_byte_values_escape_safety() {
        let mut all_bytes = Vec::with_capacity(512);
        for b in 0..=255u8 {
            all_bytes.push(b);
        }
        // Interleave with Demon patterns
        all_bytes.extend_from_slice(b"[[Category:Test]] &amp;nbsp; [[Target|Target_ext]]");
        for b in (0..=255u8).rev() {
            all_bytes.push(b);
        }

        let res = demon_transform_encode(&all_bytes);
        let decoded = demon_transform_decode(&res.encoded).unwrap();
        assert_eq!(decoded, all_bytes);

        let container = compress_dc10_container(&all_bytes, 6);
        let unpacked = decompress_dc10_container(&container).unwrap();
        assert_eq!(unpacked, all_bytes);
    }

    #[test]
    fn test_rational128_exact_reduction() {
        let r = Rational128::new(25_000_000, 100_000_000);
        assert_eq!(r.num, 1);
        assert_eq!(r.den, 4);
        assert_eq!(r.to_string(), "1/4");
    }
}
