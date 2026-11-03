# Integer-only MEM8 / AYEOS context-lattice experiment

## Status

Completed bounded prototype evaluation on 2026-11-02 (Asia/Tokyo). On full 100,000,000-byte enwik8, the selected DC10 + context-lattice container is **35,588,067 bytes**, versus **35,736,965 bytes** for unchanged DC10 + zlib level 9. That is **148,898 bytes smaller (approximately 0.4166%)**, at a substantial time and memory cost. Both in-process and separate-process restoration were byte-exact. This is a small development-corpus gain, not evidence of prize competitiveness or a test of production MEM8 resonance.

## What is being tested

A deterministic 256 × 256 context lattice remembers a previous byte position for each context slot. At a new position, the encoder checks whether the remembered context predicts an exact following byte sequence. It emits either literal bytes or a match length. The decoder reconstructs the same lattice from already decoded bytes, so no external memory snapshot or pointer dictionary is needed.

Control tokens and literal bytes are independently zlib-compressed. Headers carry the decoding options, lengths, and integrity checksum. Both raw-input and reversible-DC10-input paths are tested. All codec state and calculations use integers; timings are recorded as integer nanoseconds.

## Source lineage and limits

The local AYEOS checkout inspected is commit `06306b0`. `src/bin/ayeos-mobile.rs`, lines 846–880, packs a 79-byte WaveInt frame and derives byte-sized lattice coordinates using integer multipliers `2654435761` and `1597334677`. The experiment adapts that coordinate/layout idea to byte contexts. It does **not** call AYEOS or a production MEM8 retrieval service, and it does not establish that wave storage improves compression. Its matching method is an ordinary implicit-context LZP-style experiment.

A read-only audit also located the actual integer-native MEM8 implementation in `/data/8b-is/mem8-runtime` at commit `3143ef0` (`mem8-core` / `marine-core` 1.0.0): `crates/mem8-core/src/wave_int.rs`, `grid.rs`, and `crates/marine-core/src/{rational.rs,internal_math.rs}`. It supplies rational resonance/consolidation scores, integer coordinate binning, and tick-based retention. Legacy float shims remain in that crate, but its integer-native paths are real. The current experiment does not import those APIs. In particular, it does not test MEM8 resonance, decay, or consolidation. `CognitiveGrid::find_resonant_int` scans stored entries; directly running that scan at every byte would require a separate indexing/performance design. A lattice experiment result must not be generalized to all MEM8 techniques.

The AYEOS mobile benchmark accumulates coordinate/checksum outputs; that loop alone does not demonstrate a persistent lattice, retrieval accuracy, or enwik compression. The local baremetal lattice-fold loop likewise is not a lossless text codec. These distinctions prevent treating benchmark labels as measured codec capabilities.

G8T search identified `hutter_uleb128_encode_u64`, equivalence class `(u64)->u64::398a2f2d8912`. The relevant interface joint changes from a scalar value result to emitted token bytes. The search supplied capsule metadata, not implementation source; the byte-stream implementation here is local code, not claimed source reuse. No context-match capsule was returned by the additional search.

## Evaluation requirements

- Exact byte-for-byte restoration, not semantic similarity or checksum alone.
- Repeated/overlapping matches, all byte values, malformed/truncated input, empty input, deterministic outputs, and declared output bounds tested.
- Compare identical corpus bytes and the same zlib backend/level.
- Include every header and control/literal stream in archive size.
- Report whole-process resources separately from internal stage times.
- Report executable size separately; these development binaries are not submission packages.
- Treat small-sample parameter selection as tuning, not held-out evidence.
- Keep the baseline codec unchanged and retain negative results.

## Driver

`src/bin/mem8_lattice_arena.rs` supplies `bench`, `compress`, and `decompress` commands. Decode requires an explicit `--max-output` bound. `bench` compares to zlib level 9 and DC10 + zlib level 9 and fails if restored bytes differ.

```sh
cargo test --lib mem8_lattice
cargo test --test mem8_lattice_conformance
cargo build --release --bin mem8_lattice_arena --bin hutter_arena
./target/release/mem8_lattice_arena bench /data/corpus/enwik8 --context 4 --min-match 16 --dc10
./target/release/mem8_lattice_arena compress /data/corpus/enwik8 target/experiment.m8l --context 4 --min-match 16 --dc10
./target/release/mem8_lattice_arena decompress target/experiment.m8l target/restored.enwik8 --max-output 100000000
cmp /data/corpus/enwik8 target/restored.enwik8
```

This is a prototype, not a production archive format or a Hutter Prize eligibility claim. No pretrained model, remote service, or hidden corpus memory is used.

## Measurements

### Selection protocol and retained negative results

`MEM8_LATTICE_SWEEP_2026-11-02.csv` records all 20 tuning runs on the first 2,000,000 bytes: context length 4 or 8, minimum match 4/8/16/32/64, and raw or DC10-transformed input. Initial 12 runs covered minimum lengths 4/8/16; a second eight-run sweep checked the longer thresholds. Each run reset memory and verified restored bytes. Timings on these small tuning runs varied and were not used to select settings.

Four-byte context with a 16-byte minimum was the smallest archive in each input mode. Settings were then frozen. `MEM8_LATTICE_VALIDATION_2026-11-02.csv` records a separate 2,000,000-byte slice starting at byte offset 50,000,000, followed by the full corpus. The held-out slice is from the same enwik8 file; full-corpus results include the tuning prefix and are not independent unseen-corpus evidence.

| Corpus range | DC10 baseline bytes | Raw lattice bytes | DC10 + lattice bytes |
|---|---:|---:|---:|
| First 2 MB, tuning | 707,652 | 717,147 | 704,514 |
| 2 MB at offset 50 MB, validation | 720,462 | 732,952 | 719,090 |
| Full 100 MB | 35,736,965 | 36,227,969 | **35,588,067** |

The raw lattice variant loses to DC10 by 491,004 bytes on the full file; it is retained as an ablation. The combined variant saves 148,898 bytes beyond DC10. More aggressive short-match settings also lose: all measurements, not just the winning row, remain in the CSV. The full-file combined run emitted 235,016 matches covering 6,065,120 transformed-source bytes, with 614,690 control bytes and 84,629,409 literal bytes before their separate zlib coding.

### Identity and archive accounting

- Corpus: `/data/corpus/enwik8`, 100,000,000 bytes.
- Corpus/restored SHA-256: `2b49720ec4d78c3c9fabaee6e4179a5e997302b3a70029f30f2d582218c024a8`.
- Base marqant commit: `fbfb9457cdb73441fa9dd5c8af5db423a41e02f5`, with this experiment's working-tree additions.
- Toolchain/backend: rustc 1.99.1, cargo 1.99.1; flate2 1.1.2 / miniz_oxide 0.8.9, zlib level 9.
- Existing DC10 archive: 35,736,965 bytes including 29-byte header.
- Experimental M8L1 archive: 35,588,067 bytes including 64-byte header and both complete zlib members; 2.84704536 bits per original byte.
- Saved baseline: `target/hutter-baseline-l9.dc10`, SHA-256 `2199766e85fd5ed6a901eb9dc799f653832551bbdea4e7c053023d17425a1a57`.
- Saved experiment: `target/mem8-lattice-c4-m16-dc10.m8l`, SHA-256 `45fa9ba83889a1445a67957c764792a92fe88c91a0b33ea444edc5df3207e59f`.
- Restored files: `target/hutter-baseline-restored.enwik8` and `target/mem8-lattice-restored.enwik8`. Separate decoder processes succeeded; external `cmp` succeeded and SHA-256 matched the original for both.
- Files under `target/` are reproducible, ignored development artifacts, not source-controlled fixtures.

Every archive byte is counted. The empty-start lattice is reconstructed, so there is no omitted dictionary, memory snapshot, or trained model. Development executables are reported separately:

| Development artifact | Bytes | SHA-256 |
|---|---:|---|
| `target/release/hutter_arena` | 826,920 | `7001fce53d1bab77991b9ea85a51cb3bd19736cadda34de90292192cbc3ecc62` |
| `target/release/mem8_lattice_arena` | 770,168 | `fc1c2d798255e286894803f863916cda2fb155da4f26216317ca58090e3be871` |

Archive plus these respective development executables totals 36,563,885 bytes for DC10 and 36,358,235 bytes for the experiment. These are **not official prize totals**: these are multipurpose benchmark/codec tools, not minimized submitted decoders, and submission rules, runtime dependencies, packaging, and execution limits have not been validated. The archive-only gain is the primary comparable result.

### Time and peak memory

Single standalone invocation per mode, without another benchmark intentionally running alongside it. Wall time includes process startup and file I/O. It is indicative, not a repeated statistical benchmark. Parent measurement used integer `time.monotonic_ns()` and per-child Linux `os.wait4` peak RSS.

| Standalone process | Wall time, ns | Peak RSS, KiB |
|---|---:|---:|
| DC10 level-9 compress | 6,201,140,387 | 258,764 |
| Experimental combined compress | 9,922,546,656 | 343,716 |
| DC10 decompress | 1,211,959,057 | 224,008 |
| Experimental combined decompress | 2,658,552,355 | 312,412 |

The saved experiment's internal encode time was 9,873,361,594 ns and decode time 2,625,002,829 ns. Whole-arena full-file runs peaked at 433,108 / 433,168 KiB for raw / combined variants; that includes comparative baseline stages and multiple retained buffers, so standalone figures above are more useful.

**Tradeoff:** approximately 0.42% fewer archive bytes, but slower compression and roughly twice the standalone decoder wall time, with increased peak memory. No energy measurements were made. Do not replace DC10 by default based on this small gain.

## Validation and maintenance

- `cargo test --lib`: all 48 library tests passed, including nine new lattice unit tests and four existing Hutter tests.
- `cargo test --test mem8_lattice_conformance`: four additional tests cover parameter combinations, arbitrary bytes, long overlaps, every static DC10 pattern, deterministic outputs, truncation, trailing bytes, and output bounds.
- The full repository suite was not claimed to pass: pre-existing DNS failures are recorded in `HUTTER_BASELINE_2026-11-02.md`.
- Source inspection found no `f32` or `f64` in the new codec or driver. This statement is scoped to this execution path, not all legacy marqant/MEM8 APIs or all machine instructions emitted by dependencies.
- Decoder checks complete zlib members, exact lengths, bounded inflation, canonical varints, original checksum, and DC10 expansion preflight. `max_output` caps original bytes, not aggregate working memory. FNV is not an authentication mechanism.
- M8L1 is experimental. Its DC10 preflight follows the existing grammar and uses the shared dictionary; future grammar changes need corresponding tests and format review.
- Cargo's incidental root-version refresh in Cargo.lock was reverted; no dependency versions or baseline codec logic were changed.

## Recommendation

Keep this as a measured control, not a production integration. It establishes that deterministic past-context matching can add a small gain after DC10. It does not establish that MEM8 wave resonance, retention, semantic coordinates, Kompress, or trained predictors improve compression.

The next genuine MEM8 experiment should pin `mem8-runtime` commit `3143ef049c59b6b097d69349cfbd8dd08d86cf6c`, expose a bounded candidate index rather than scanning every wave per byte, derive all features/ticks from decoded bytes, and ablate integer retention/ranking against this ordinary LZP control. Any external state must be serialized and counted. Test full enwik9 and official constraints only after a stronger gain survives these comparisons.
