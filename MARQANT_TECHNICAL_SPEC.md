# Marqant Technical Specification

**Doc ID:** MARQANT-TECH-001
**Version:** 0.3.0
**Status:** Draft (Implementable)
**Authors:** Hue, Aye, Omni @ 8b.is
**Last Updated:** 2026-01-20

> "Same meaning. Fewer notes. Faster recall." 🎼

---

## 0. Terms & Notation

| Term | Definition |
|------|------------|
| **MUST / SHALL** | Absolute requirement |
| **MUST NOT / SHALL NOT** | Absolute prohibition |
| **SHOULD / RECOMMENDED** | Strong preference, exceptions require justification |
| **MAY / OPTIONAL** | Truly optional |
| **Decoder** | Implementation that reconstructs original text |
| **Encoder** | Implementation that produces `.mq` / `.mqc` / `.mqa` |
| **Token** | Single-byte code representing a string expansion |
| **Dictionary** | Mapping from token byte → UTF-8 expansion |
| **Chunk** | Self-contained block of encoded content with local metadata |
| **Wave Tree** | Hierarchical index for chunked documents |

---

## 1. Design Goals

### 1.1 Primary Goals

1. **Deterministic decode**: Level 0 decode MUST exactly reproduce the original byte sequence
2. **Copy/paste safe**: Base format MUST survive transport through plaintext channels
3. **Self-contained**: Single-doc `.mq` MUST decode without network access
4. **Streaming-friendly**: Decoders SHOULD operate in one pass with bounded memory
5. **AI-optimized**: Reduce token count for LLM consumption by 70-90%

### 1.2 Extended Goals (Chunked Format)

6. **Chunkable**: Large documents SHOULD be fetchable in pieces
7. **Content-addressable**: Chunks SHOULD be locatable by semantic/structural position
8. **Progressive loading**: Summaries first, detail on demand
9. **Shared dictionaries**: Common patterns cached globally, not per-document

### 1.3 Non-Goals

- General-purpose binary compression (use zstd/brotli)
- Lossy summarization (unless explicitly requested via Angel stages)
- Encryption or DRM
- Mandatory AI/LLM involvement at decode time

---

## 2. File Types

### 2.1 `.mq` — Single-Document Marqant

Self-contained compressed document. Contains:
- Header
- Dictionary section(s)
- Encoded content stream

**Use case:** Individual files, chat transport, embedding in other formats.

### 2.2 `.mqc` — Chunked Marqant

Streamable, content-addressable chunked document. Contains:
- Header with index metadata
- Wave Tree index (hierarchical structure)
- Shared dictionary references
- Data blocks (fetched on demand)

**Use case:** Large documents (books, documentation sets), streaming, partial fetch.

### 2.3 `.mqa` — Aggregate Marqant

Bundle of multiple documents with shared dictionary. Contains:
- Header
- Shared dictionary section(s)
- Manifest with file metadata
- Embedded document streams

**Use case:** Project documentation, repository bundles, archival.

---

## 3. MQ2 Wire Format (Single Document)

### 3.1 Top-Level Layout

```
[HEADER LINE]
[DICTIONARY SECTION(S)]
~~~~
[CONTENT STREAM]
```

The `~~~~` (four tildes) delimiter separates dictionary from content. Alternative: `---` (three dashes) for legacy compatibility.

### 3.2 Header Format

Header MUST be a single ASCII line. Two formats are supported:

**Compact format (RECOMMENDED):**
```
MQ2~<variant>~<timestamp_hex>~<orig_size_hex>~<comp_size_hex>~<token_count_hex>~<flags>
```

Example:
```
MQ2~UNI~6789ABCD~1A3F~0E2B~42~text
```

**Verbose format (OPTIONAL):**
```
MQ2
ver=0.3
ts=1736000000
orig=6719
comp=3627
tokens=66
flags=ASCII_SAFE,SEMANTIC
```

Decoders MUST accept both formats. Unknown header fields MUST be ignored.

### 3.3 Header Fields

| Field | Type | Description |
|-------|------|-------------|
| `variant` | string | Encoder variant (e.g., `UNI`, `STD`, `SEM`) |
| `ts` / `timestamp` | hex/dec | Unix epoch timestamp |
| `orig` / `orig_size` | hex/dec | Original size in bytes |
| `comp` / `comp_size` | hex/dec | Compressed size in bytes |
| `tokens` / `token_count` | hex/dec | Number of dictionary entries |
| `flags` | string | Comma-separated or bitfield |

### 3.4 Flags

| Flag | Meaning |
|------|---------|
| `ASCII_SAFE` | Content stream uses only transport-safe escapes |
| `BINARY` | Content stream may contain raw bytes 0x80-0xFF |
| `SEMANTIC` | Encoder used semantic heuristics (decode still exact) |
| `STD:<id>` | References a standard dictionary by ID |
| `ANGEL=<n>` | Indicates Angel blessing level available |

Decoders MUST default to `ASCII_SAFE` behavior unless `BINARY` is specified.

---

## 4. Dictionary Section

### 4.1 Dictionary Markers

| Marker | Purpose |
|--------|---------|
| `~T` | Primary token dictionary |
| `~S` | Secondary/semantic token hints |
| `~D` | Generic dictionary section (verbose format) |

### 4.2 Token Map Format

**Compact binary (RECOMMENDED for ~T):**
```
~T<token_byte><len_hi><len_lo><pattern_bytes>...
```

Where:
- `token_byte`: Single byte (0x80-0xFE)
- `len_hi`, `len_lo`: Big-endian u16 pattern length
- `pattern_bytes`: UTF-8 pattern

**Text format (for debugging/verbose):**
```
~D
0x80	#
0x81	##
0x82	```
```

### 4.3 Token Byte Allocation

```
0x00-0x1F: Control tokens (reserved)
0x20-0x7E: ASCII passthrough (in ASCII_SAFE mode)
0x7F:      X-token gateway (extension prefix)
0x80-0xFE: Dynamic tokens (assignable)
0xFF:      Reserved for future use
```

### 4.4 Dictionary Rules

- Token bytes MUST be unique within a dictionary
- Expansions MUST be valid UTF-8
- Expansions MUST NOT create recursive loops
- Dictionary MUST be complete before content stream

### 4.5 Dictionary ID Computation

```
dict_id = fnv1a64(~T_payload + "|" + ~S_payload)
```

Displayed as: `fnv1a64:<16_hex_digits>`

---

## 5. Content Stream

### 5.1 Encoding Modes

**ASCII_SAFE mode:**
- Literal ASCII (0x20-0x7E) passed through
- Tokens and special bytes use escape sequences
- Safe for chat, email, markdown embedding

**BINARY mode:**
- Token bytes (0x80-0xFE) appear directly
- Higher compression, not transport-safe

### 5.2 Escape Sequences

| Escape | Meaning | Mode |
|--------|---------|------|
| `~~` | Literal `~` | Both |
| `~n` | Newline (0x0A) | ASCII_SAFE |
| `~t` | Tab (0x09) | ASCII_SAFE |
| `~xHH` | Literal byte 0xHH | ASCII_SAFE |
| `~tHH` | Token byte 0xHH | ASCII_SAFE |

**Named escapes (MQ2-UNI variant):**

| Escape | Expansion |
|--------|-----------|
| `~H1` | `# ` |
| `~H2` | `## ` |
| `~PP` | `\n\n` (paragraph) |
| `~LI` | `\n- ` (list item) |
| `~CB` | ` ```\n` (code block) |
| `~CE` | ` ``` ` (code end) |
| `~IN` | `    ` (4-space indent) |
| `~CO` | `: ` |
| `~CM` | `, ` |

Decoders MUST support both hex and named escapes.

### 5.3 Decoding Algorithm

```rust
fn decode_level0(stream: &[u8], dict: &HashMap<u8, String>) -> Result<String> {
    let mut output = String::new();
    let mut i = 0;

    while i < stream.len() {
        let b = stream[i];

        if b == b'~' {
            // Handle escape sequence
            i += decode_escape(&stream[i..], &mut output)?;
        } else if b >= 0x80 && b <= 0xFE {
            // Token expansion
            output.push_str(dict.get(&b).ok_or("unknown token")?);
            i += 1;
        } else if b == 0x7F {
            // X-token (extension) - skip for Level 0
            i += 2; // Skip prefix + type byte
        } else {
            // ASCII passthrough
            output.push(b as char);
            i += 1;
        }
    }

    Ok(output)
}
```

---

## 6. Decoder Levels

### 6.1 Level 0: STRICT (Bit-Perfect)

- MUST reproduce original exactly
- MUST be supported by all decoders
- No interpretation, no corrections

### 6.2 Level 1: EXTENDED

- Supports X-token extensions (0x7F prefix)
- Additional 4096 extended pattern tokens
- Falls back to Level 0 for unknown extensions

### 6.3 Level 2: FULL

- All Level 1 features
- Semantic markers
- Cross-file references
- Compression mode switching

### 6.4 Compatibility Matrix

| Feature | L0 | L1 | L2 |
|---------|----|----|-----|
| Base tokens (0x80-0xFE) | ✓ | ✓ | ✓ |
| Skip unknown X-tokens | ✓ | ✓ | ✓ |
| Extended patterns | skip | ✓ | ✓ |
| Semantic markers | skip | skip | ✓ |
| Cross-file refs | skip | skip | ✓ |

---

## 7. Chunked Format (MQC)

### 7.1 Design Principles

Inspired by MEM|8's Wave Tree architecture:

1. **Hierarchical index**: Document structure as a tree
2. **Content-addressable blocks**: Hilbert curve for locality
3. **Progressive loading**: Summaries before detail
4. **Shared dictionaries**: Global tokens, local overrides

### 7.2 MQC Layout

```
[HEADER]
~~~~
~I (INDEX - Wave Tree)
[serialized tree nodes]
~~~~
~D:shared (SHARED DICTIONARY REFS)
[dictionary references or inline]
~~~~
~B:<hilbert_index> (DATA BLOCK)
[block content]
~~~~
~B:<hilbert_index>
[block content]
...
```

### 7.3 Header (MQC)

```
MQC
ver=0.3
index_size=<bytes>
block_count=<n>
shared_dict=<dict_id>,<dict_id>,...
hilbert_order=<n>
total_orig=<bytes>
total_comp=<bytes>
```

### 7.4 Wave Tree Index (~I Section)

The index is a serialized quadtree where each node contains:

```rust
struct WaveTreeNode {
    // Logical position in document
    bounds: (u32, u32, u32, u32),  // (start, end, depth, sibling)

    // Level in tree (0 = root)
    level: u8,

    // Low-resolution summary (title, keywords, topic)
    summary: CompactSummary,

    // For leaf nodes: Hilbert index to data block
    block_address: Option<u64>,

    // Child node indices (for internal nodes)
    children: Option<[u32; 4]>,
}

struct CompactSummary {
    title: String,           // Section/chapter title
    word_count: u32,         // Approximate words
    topics: Vec<u8>,         // Ayanese concept IDs (optional)
    sentiment: u8,           // Wave ID (optional)
}
```

Serialization: Bincode or Postcard (compact binary).

### 7.5 Shared Dictionary References (~D:shared)

```
~D:shared
ref=literary-en-v1,fetch=dns:literary-en.m8.is
ref=tolstoy-v1,fetch=inline
~~~~
[inline dictionary if fetch=inline]
```

Decoders SHOULD cache shared dictionaries by ID.

### 7.6 Data Blocks (~B Section)

Each block is self-contained:

```
~B:0x4A2F
dict_local=<n>
orig=<bytes>
comp=<bytes>
~~~~
~D:local (optional local dictionary overrides)
~~~~
[encoded content stream]
```

### 7.7 Hilbert Curve Addressing

Blocks are addressed using Hilbert curve indices for locality:

```rust
fn block_address(logical_x: u32, logical_y: u32, order: u8) -> u64 {
    hilbert_d2xy(order, logical_x, logical_y)
}
```

Benefits:
- Spatially adjacent content → adjacent block addresses
- Efficient range queries
- Cache-friendly sequential reads

### 7.8 Progressive Loading Protocol

1. **Fetch header** (tiny) → know structure
2. **Fetch index** (~I) → see full TOC with summaries
3. **Fetch shared dicts** → cache globally
4. **Fetch specific block** (~B:addr) → read chapter/section
5. **Prefetch adjacent blocks** → anticipate navigation

---

## 8. Aggregate Format (MQA)

### 8.1 Layout

```
MQA
ver=0.3
doc_count=<n>
shared_dict=<dict_id>
~~~~
~D:shared
[shared dictionary]
~~~~
~M (MANIFEST)
[JSON or line-based manifest]
~~~~
~F:path/to/file1.md
[MQ2 stream for file1]
~~~~
~F:path/to/file2.md
[MQ2 stream for file2]
...
```

### 8.2 Manifest Format

```json
{
  "files": [
    {
      "path": "README.md",
      "orig_size": 4523,
      "comp_size": 1205,
      "offset": 1024,
      "checksum": "fnv1a64:abcd1234"
    },
    ...
  ]
}
```

### 8.3 Extraction

Decoders MUST support:
- List all files (`mq list archive.mqa`)
- Extract single file (`mq extract archive.mqa --file README.md`)
- Extract all (`mq extract archive.mqa --all`)

---

## 9. Angels & Demons (Post-Processing)

### 9.1 Philosophy

- **Demons** compress (entropy reduction)
- **Angels** decompress with optional blessings (interpretation)

### 9.2 Blessing Levels

| Level | Name | Behavior | Deterministic |
|-------|------|----------|---------------|
| 0 | STRICT | Bit-perfect reconstruction | Yes |
| 1 | MINOR | Fix double spaces, obvious typos | Yes |
| 2 | HARMONY | Normalize structure, fix formatting | Yes |
| 3 | CREATIVE | Semantic variations, synonyms | No (seeded) |

### 9.3 Rules

- Decoders MUST default to Level 0
- Level 1-2 MUST be deterministic (same input → same output)
- Level 3 MAY use seeded randomness for reproducibility
- Angel level MUST be explicitly requested, never automatic

### 9.4 API

```rust
use marqant::angel_blessings::{Angel, BlessingLevel};

let angel = Angel::new(BlessingLevel::Harmony);
let (blessed, stats) = angel.bless(decoded_text)?;
```

---

## 10. Phoenix Extensions (Semantic Layer)

### 10.1 Lock + Residual Model

Inspired by Phoenix audio codec:

```
Text = SemanticLock + SurfaceResidual
```

- **SemanticLock**: Canonical meaning (Ayanese tokens, stems, concepts)
- **SurfaceResidual**: Exact surface form (case, punctuation, word choice)

### 10.2 Extension Sections

| Section | Purpose | Required for L0? |
|---------|---------|------------------|
| `~A` | Ayanese semantic stream | No |
| `~W` | Wave signature (affect/resonance) | No |
| `~R` | Surface residual (for higher-level synthesis) | No |
| `~C` | Culture/temporal context | No |

### 10.3 Ayanese Section (~A)

```
~A
[WTL token stream - 32-bit tokens]
[ConceptID|RelationID|WaveID|ModID] × N
```

See Ayanese specification for token format.

### 10.4 Wave Section (~W)

```
~W
dominant_freq=<hz>
valence=<0-7>
arousal=<0-7>
dominance=<0-3>
```

For search, clustering, and emotional context.

### 10.5 Culture Section (~C)

```
~C
locale=en-US
era=2024
register=casual
sensitivity=none
```

Enables culture-aware rendering without infecting core codec.

---

## 11. Ayanese Integration

### 11.1 Universal Translation Vision

```
Any Language → Ayanese (semantic) → Any Language
```

- **256 concept tokens** (core vocabulary)
- **Wave ID** encodes affect (valence/arousal/dominance)
- **Bridge models** handle language-specific mapping
- **Core LLM** trained on Ayanese only (smaller, universal)

### 11.2 Token Structure

```
[ConceptID:8][RelationID:8][WaveID:8][ModID:8] = 32 bits
```

- **ConceptID**: 0x00-0xFF (256 universal concepts)
- **RelationID**: Semantic link operator
- **WaveID**: Affect packed as valence(3)+arousal(3)+dominance(2)
- **ModID**: Temporal flags + render mode

### 11.3 Bridge Model Architecture

```
Input (any language)
    ↓
[Small encoder per language] ← Culture-aware, temporal-aware
    ↓
Ayanese tokens
    ↓
[Core reasoning in semantic space]
    ↓
Ayanese tokens
    ↓
[Small decoder per language] ← Culture-appropriate rendering
    ↓
Output (any language)
```

---

## 12. MEM|8 Integration

### 12.1 Storage Substrate

MQC chunks are MEM|8 blocks:

- **Wave Tree** = MEM|8's hierarchical index
- **Hilbert addressing** = MEM|8's locality-preserving layout
- **Progressive loading** = MEM|8's attention model

### 12.2 Cache Strategy

```rust
struct MarqantCache {
    // Global dictionary cache (LRU)
    dict_cache: LruCache<DictId, Dictionary>,

    // Block cache (LRU)
    block_cache: LruCache<HilbertIndex, DataBlock>,

    // Index cache (always keep recent)
    index_cache: LruCache<DocId, WaveTree>,
}
```

### 12.3 Streaming API

```rust
// Returns channel, streams data as available
fn fetch_progressive(doc_id: &str) -> Receiver<ProgressiveChunk> {
    // 1. Emit header immediately
    // 2. Emit index (summaries visible)
    // 3. Emit blocks as requested/prefetched
}
```

---

## 13. Compliance & Testing

### 13.1 Required Tests

1. **Round-trip**: Encode → Decode → Compare (must match)
2. **Escape integrity**: `~` literal handling
3. **Token collision**: No duplicate tokens
4. **Unicode**: CJK, emoji, RTL text
5. **Large files**: Streaming memory bounds
6. **Aggregate**: Extract all files correctly
7. **Chunked**: Progressive load, partial fetch

### 13.2 Reference Corpus

Implementations SHOULD test against:
- `testy/*.md` → `testy/*.mq` (provided in repo)
- Wikipedia sample articles
- Code files with complex indentation
- Multilingual documents

### 13.3 Conformance Levels

| Level | Requirements |
|-------|--------------|
| **L0 Decoder** | Decode any valid `.mq` to exact original |
| **L1 Decoder** | L0 + X-token support |
| **L2 Decoder** | L1 + semantic markers, cross-refs |
| **Encoder** | Produce valid L0-decodable output |
| **Full** | All formats (MQ, MQC, MQA) + Angels |

---

## 14. Reference Implementation

### 14.1 Rust Crate

```toml
[package]
name = "marqant"
version = "1.0.0"

[[bin]]
name = "mq"           # CLI tool
[[bin]]
name = "demon_compressor"
[[bin]]
name = "angel_decompressor"
```

### 14.2 CLI Usage

```bash
# Compress
mq compress input.md -o output.mq

# Decompress (Level 0)
mq decompress input.mq -o output.md

# Decompress with blessings
mq decompress input.mq -o output.md --angel=2

# Aggregate
mq aggregate ./docs -o docs.mqa

# Extract from aggregate
mq extract docs.mqa --file README.md

# Inspect
mq inspect file.mq --show-tokens --show-dict
```

### 14.3 Library API

```rust
use marqant::{Marqant, mq2_uni_encode, mq2_uni_decode};
use marqant::angel_blessings::{Angel, BlessingLevel};

// Simple encode/decode
let compressed = mq2_uni_encode(input.as_bytes())?;
let decompressed = mq2_uni_decode(&compressed)?;

// With Angel blessings
let angel = Angel::new(BlessingLevel::Harmony);
let (blessed, stats) = angel.bless(&String::from_utf8(decompressed)?)?;
```

---

## 15. Future Extensions

### 15.1 Planned

- Delta compression between versions
- Real-time streaming encode/decode
- WASM decoder for browser
- Standard dictionary registry (DNS-based)

### 15.2 Research

- Neural-assisted token selection
- Cross-document token sharing
- Ayanese LLM training corpus

---

## 16. Changelog

| Version | Date | Changes |
|---------|------|---------|
| 0.3.0 | 2026-01-20 | Added MQC chunked format, MEM|8 integration, Ayanese extensions |
| 0.2.0 | 2025-12-xx | Added X-token extensions, decoder levels |
| 0.1.0 | 2025-xx-xx | Initial specification |

---

*"Why send words when you can send thoughts?"* — The Future of Marqant

**Built with ❤️ by Hue, Aye, and the 8b.is collective**
