# 😈 Demon Compression (.dc) — Competition Submission Specification

**Tagline:** *Because only demons can handle the heat of compression.*

**Authors:** Christopher Michael Chenoweth (Hue/Wraith), Omni, Claude  
**Target benchmark:** **enwik9** (1,000,000,000 bytes)

---

## 1) Overview

**Demon Compression (.dc)** is a purpose-built archival format and toolchain for Wikipedia XML (enwik9). It treats compression as **program synthesis**: the archive stores a compact set of **demons** (tiny bytecode programs) that procedurally reconstruct recurring structures (e.g., `[[link]]`, categories, templates), plus a lightweight stream describing where to invoke them. The decompressor is deterministic, offline, single-binary, and self-contained.

**Submission plan:** use the relaxed rule path with a **single executable** (`dc9`) acting as both compressor and decompressor (same binary, different flags). This reduces size accounting to:

```
S = length(dc9.zip) + length(archive9.dc)
```
(Per rules: when `comp9a == decomp9`, the 2× decompressor term reduces to 1×.)

**Design goals:**
- Beat the current record by maximizing structural reuse in enwik9.
- Keep the **decoder** tiny, deterministic, and resource‑bounded.
- Fully comply with **no external data**: all model parameters and demons are stored in the archive.

---

## 2) Rule Compliance Map (Hutter Prize)

| Rule | Compliance Strategy |
|---|---|
| Executables: Windows or Linux (x86_64/x86), no extra installs | Provide statically linked binaries for Linux (x86_64) and Windows (x86_64). No runtime deps. |
| No outside inputs (files, net, dicts) | Decoder reads only `archive9.dc` and produces `data9` identical to enwik9. |
| Time/RAM/HDD: ≤ 10 GB RAM, ≤ 100 GB temp, runtime bound via `70,000/T` | Decoder is streaming and uses ≤ 1 GB RAM. Encoder fits within limits using chunked passes. |
| Self-extract or (comp + decomp + archive) | We submit **`dc9`** (both roles) + `archive9.dc`. |
| Open, OSI-licensed source | MIT license. Makefiles for Linux/Windows. |
| Determinism | No RNG, fixed semantics. Deterministic across platforms. |

---

## 3) File Format: `.dc`

All multi-byte integers are **little-endian**. Variable-length integers use **unsigned LEB128** (7-bit groups, MSB=continuation).

### 3.1 Magic + Header
```
struct DcHeader {
  magic:    [u8; 4]   // "DC01"
  version:  u8        // 0x01
  flags:    u8        // bit0: CRC32 present; bit1: SHA256 present; others 0
  orig_len: u64       // expected decompressed size (= 1,000,000,000 for enwik9)
  meta_len: u32       // bytes of encoder meta (LEB128 tables, optional)
}
// followed by: meta[meta_len]
```
**Meta** (optional, decoder-ignored for correctness) stores encoder notes (e.g., thresholds) useful for analysis but not required to decode.

### 3.2 Demon Table
Each demon is a compact program (type + constants). We restrict to minimal types to keep the decoder small.
```
struct DcDemons {
  count: u16
  entries: [DcDemon; count]
}

enum DemonType: u8 {
  WRAP = 1,  // emit: prefix + param + suffix
}

struct DcDemonWrap { // for type WRAP
  prefix_len: uleb128
  prefix: [u8; prefix_len]
  suffix_len: uleb128
  suffix: [u8; suffix_len]
}

struct DcDemon { // generic container
  dtype: u8
  body:  DcDemonWrap  // since dtype=WRAP in v1
}
```
Notes:
- **V1 deliberately supports only `WRAP`** demons, proven abundant and effective in enwik9 (`[[...]]`, `[[Category:...]]`, `{{...}}`, etc.). Additional demon types (e.g., templated multi-slot) MAY be added in a future version without breaking v1.

### 3.3 Stream Section
A sequence of **records** that reconstruct the original when interpreted left-to-right.
```
const CTRL: u8 = 0xFF

enum Op: u8 {
  RAW    = 0x00,   // literal run
  INVOKE = 0x01,   // call demon with param
}

// RAW
0xFF 0x00 <len:uleb128> <bytes:len>

// INVOKE
0xFF 0x01 <demon_id:uleb128> <plen:uleb128> <param:plen>
// decoder emits: prefix(param)demon + param + suffix(demon)
```
Record stream continues until end of file. If `flags.bit0==1`, the file ends with `crc32(le)` of the reconstructed output. If `flags.bit1==1`, ends with `sha256[32]` as well. (Checksum presence indicated only by `flags` to avoid ambiguity.)

**Rationale:**
- RAW/INVOKE is enough to reconstruct XML/markup dominated corpora.
- Decoder complexity stays minimal; encoder bears the sophistication.

---

## 4) Decoding Algorithm (Reference)

```
function decode_dc(input) -> bytes_out:
  read header; assert magic=="DC01", version==1
  read meta (skip)
  read demon table into memory (vector)
  out_size_target = header.orig_len
  init output sink (stream-to-file), track bytes_emitted
  while not EOF of stream_section:
    b = read_u8()
    if b != 0xFF: // forward-compat literal passthrough
       write(b); continue
    op = read_u8()
    if op == RAW:
       n = read_uleb128(); copy_exact(n) from input to output
    else if op == INVOKE:
       did = read_uleb128(); check did < demon_count
       plen = read_uleb128(); param = read_exact(plen)
       d = demons[did] // dtype==WRAP
       write(d.prefix); write(param); write(d.suffix)
    else: error("unknown op")
    if bytes_emitted >= out_size_target: break
  if flags.crc32: verify trailing crc32 over output
  if flags.sha256: verify trailing sha256
  assert bytes_emitted == out_size_target
```
**Memory:** demon table + small buffers (≤ a few MB). Output is streamed to `data9` without full in-RAM expansion.

---

## 5) Encoding Algorithm (High-Level)

**Inputs:** `enwik9` (1e9 bytes).  
**Outputs:** `archive9.dc`.

1. **Structural pass (scan only):**
   - Regex/automata detect wrapper patterns with bounded params (e.g., up to 64–256 bytes) for: `[[...]]`, `[[Category:...]]`, `{{...}}`, XML element wrappers as needed.
   - Record occurrences (start,end,param_length) and estimate **net savings** per wrapper family.
2. **Demon selection:**
   - For each wrapper family, compute:  
     `per_save ≈ |prefix| + |param| + |suffix| - (invoke_overhead)`
   - Keep families with `sum(per_save * freq) > header_overhead_threshold`.
3. **Layout planning:**
   - Build **non-overlapping** match plan (left-to-right, longest-first per start).
4. **Emit archive:**
   - Write header (`orig_len=1e9`, flags=CRC32 on by default).
   - Emit demon table for selected families (dedup exact prefix/suffix pairs).
   - Walk source and alternate between **RAW** and **INVOKE** according to the plan.
   - Append checksum(s) if enabled.

**Note:** Further reductions (suffix/case/space models, tokenization) live entirely on the **encoder side**, because the decoder simply replays the stream.

---

## 6) Determinism, Security, and Limits

- **Determinism:** No randomness, no clocks, no environment reads. Byte-exact output.
- **Sandbox by design:** Decoder offers no file system writes beyond the output file, no code execution. Demon bytecode is data-only (`WRAP`).
- **Resource limits:** Decoder validates lengths (`ULEB128` bounds), demon indices, and short-reads to avoid bombs. It streams output and never allocates proportional to `orig_len`.
- **Forward-compatibility:** Unknown ops are rejected; unknown demon types are rejected in v1. Version bump (0x02+) may add types.

---

## 7) CLI and Submission Artifacts

**Single binary:** `dc9` (Linux and Windows builds).  
**Archive:** `archive9.dc`.

### Usage
```
# Compress (build archive9.dc from enwik9)
./dc9 -c enwik9 archive9.dc

# Decompress (reproduce 1,000,000,000-byte data9)
./dc9 -d archive9.dc data9
# With no output path, default is `data9` in CWD
./dc9 -d archive9.dc
```
**Submission line (example):**
```
./dc9 -d archive9.dc
```
(Per rules: single, short instruction.)

---

## 8) Build & License

- **Language:** Rust 1.78+ for encoder/decoder (same codebase). Optionally, a micro‑C decoder variant may be provided if it reduces `dc9.zip` significantly.
- **Build flags (size-first decoder):** `-Oz -C lto=fat -C codegen-units=1 -C panic=abort -C target-feature=-crt-static` (Windows), `musl` static Linux build evaluated; final choice based on `zip` size empirics.
- **License:** MIT (OSI-approved). Full source + Makefile included.

---

## 9) Complexity & Resources

- **Decoder time:** Linear in number of stream records and bytes written; dominated by I/O.
- **Decoder RAM:** O(#demons + small buffers) — typically **≪ 64 MB**.
- **Encoder:** Multi-pass but chunked; peak RAM ≤ 8–10 GB; temp disk use << 100 GB (tunable).

---

## 10) Verification & Bench Harness

- `--verify` mode recomputes CRC32/SHA256 over the reconstructed output and checks `orig_len`.
- Harness logs: input size, archive size, demon count, decode time, peak RSS.
- A script provides machine spec + Geekbench5 T and compares against allowed time bound `70,000/T` hours.

---

## 11) Roadmap to Record

- **Low-hanging gains:** category/link/template wrappers (very frequent in enwik9).  
- **Encoder-only heuristics:** space/case modeling, morphology split (-ing, -tion), dynamic chunking across article boundaries.
- **No decoder growth:** all sophistication stays on the encoder side; the `.dc` stream remains RAW/INVOKE.

Target: **S < L** by maximizing archive savings while keeping `dc9.zip` minimal.

---

## 12) Notes on Fairness & Spirit

- Decoder requires nothing but `.dc` input; no hidden priors or dictionaries.
- All reconstruction instructions are **explicitly encoded** (demons + params); no external knowledge is assumed.
- The **heat** is acknowledged, but only as explanatory framing; there are no sensors or adaptive throttles in the submission build.

---

## 13) Credits

**Authors:** Christopher Michael Chenoweth (Hue/Wraith), Omni, Claude.  
**Acknowledgments:** The Wikipedia community (for structured chaos), and the Hutter Prize committee for a benchmark that rewards *structure-aware* compression.

---

## 14) Appendix: File Layout Diagram

```
+--------------------+
|  Magic "DC01"      |
|  Version=1, Flags  |
|  orig_len (u64)    |
|  meta_len (u32)    |
+--------------------+
|  meta[meta_len]    |  (optional)
+--------------------+
|  demon_count (u16) |
|  demons[...]       |  WRAP demons (prefix,suffix)
+--------------------+
|  stream records    |  RAW / INVOKE records
+--------------------+
|  CRC32? SHA256?    |  (if flags set)
+--------------------+
```

