# Hutter development baseline — 2026-11-02 (Asia/Tokyo)

## Scope

Existing, unmodified DC10 arena on full local enwik8, not enwik9 and not a prize submission. One run per compression level; timing is indicative, not a statistical comparison. No current prize record or eligibility rules were verified.

## Reproduction and identity

- Repository commit: `fbfb9457cdb73441fa9dd5c8af5db423a41e02f5` (initial worktree clean).
- Corpus: `/data/corpus/enwik8`, exactly 100,000,000 bytes.
- Corpus SHA-256: `2b49720ec4d78c3c9fabaee6e4179a5e997302b3a70029f30f2d582218c024a8`.
- Arena input/restored FNV-1a: `0xF2F5945BC2F61147`.
- Linux x86_64, kernel 7.2.11-arch1-1, glibc 2.44; 48 logical CPUs visible.
- rustc 1.99.1 (58cf8f9c3); cargo 1.99.1 (5f94df478).
- flate2 1.1.2 with miniz_oxide 0.8.9 in the build.
- Release binary SHA-256: `63f809539d13801aaca40398a634445113937c7bf44a570e0ad8530e453e0a92`.

```sh
cargo test --lib hutter_demon
cargo build --release --bin hutter_arena
./target/release/hutter_arena --bench /data/corpus/enwik8 --deflate-level 6
./target/release/hutter_arena --bench /data/corpus/enwik8 --deflate-level 9
```

Cargo automatically updated only the root package version in Cargo.lock from 1.1.8 to match Cargo.toml's 1.2.0. That incidental edit was reverted after measurement; dependency versions were unchanged. A future Cargo invocation may repeat this metadata update.

## Measured results

The arena labels its baseline Deflate; source inspection confirms both paths use **zlib-wrapped Deflate**. DC10 totals additionally include a 29-byte header.

| Metric | Level 6 | Level 9 |
|---|---:|---:|
| Baseline zlib bytes | 36,513,697 | 36,460,101 |
| DC10 + zlib payload bytes | 35,786,001 | 35,736,936 |
| DC10 container bytes | 35,786,030 | 35,736,965 |
| Net bytes saved vs same-level baseline | 727,667 | 723,136 |
| Relative archive reduction | 1.9929% | 1.9834% |
| Container bits per original byte | 2.8628824 | 2.8589572 |
| DC10 compression stage, ms | 5,003 | 5,969 |
| DC10 decompression stage, ms | 576 | 642 |
| Baseline compression stage, ms | 4,320 | 5,036 |
| Whole arena wall time, seconds | 11.444950 | 13.078171 |
| Whole arena user CPU seconds | 11.337296 | 12.980688 |
| Whole arena system CPU seconds | 0.088708 | 0.074768 |
| Whole arena peak RSS, KiB | 509,956 | 509,864 |
| Direct transformed-byte round trip | PASS | PASS |
| Full container round trip (byte comparison + checksum) | PASS | PASS |
| Process exit | 0 | 0 |

Resource measurements used Python subprocess + perf_counter and Linux resource.getrusage(RUSAGE_CHILDREN), with a fresh Python process per arena invocation. Peak RSS covers the entire arena, which retains multiple buffers and performs multiple stages; it is **not** standalone compressor/decompressor memory. No compressed archive was persisted by --bench. Executable/model size is not included in the table, so these are not prize-accounted totals.

## Transform observations

Both levels produced 90,694,529 transformed bytes: 9,305,471 bytes smaller than input (9.305471%). Gross substitution savings of 9,979,883 bytes were offset by 674,412 high-byte escape bytes.

| Category | Invocations | Gross bytes saved before entropy coding |
|---|---:|---:|
| XML envelope | 110,673 | 3,413,096 |
| Wiki links/categories | 2,209,259 | 2,959,255 |
| Pipe-link stems | 83,559 | 785,837 |
| XML entities | 470,001 | 2,821,695 |

These category figures are not compressed-byte contributions. Independent ablations are needed before crediting a category with actual archive savings. The arena's Landauer figures are formulas, not measured energy usage, and are not used to evaluate compression quality here.

## Tests

- All 39 library tests passed in the initial full-suite invocation, including all four Hutter tests.
- Dedicated `cargo test --lib hutter_demon`: 4 passed.
- Initial `cargo test` stopped at tests/dns.rs: resolve_dns_success and resolve_dns_nxdomain failed with Invalid padding. Later integration tests were not reached, so this is not a full-suite pass.
- Serial diagnostic `cargo test --test dns -- --test-threads=1`: invalid_pair and nxdomain passed; success still failed with Invalid padding. The DNS tests share a mock executable and environment variable. No DNS fixes were made as part of this benchmark.

## Interpretation and next experiments

The existing preprocessor provides a real but modest approximately 2% archive reduction over this Deflate implementation. Its 9.3% intermediate-byte reduction must not be reported as a 9.3% compressed-archive gain. Level 9 saves only 49,065 container bytes over level 6 in this run. This is a useful working baseline, not evidence of prize competitiveness.

The user identified sibling projects under /data/8b-is. Initial read-only inspection located `/data/8b-is/kompress-ultra`, `/data/8b-is/mem8`, and other MEM8 variants. Kompress Ultra's README describes a context composer/pruner/rewriter/circulator and a 1-bit engine, not a demonstrated lossless enwik codec. MEM8's README advertises high compression ratios, but those claims were not verified by this baseline. The local marqant MEM8 bridge maps phonetic packets into float wave signatures; it is not an established reversible original-byte codec.

A promising hypothesis to test, not an implemented result: use deterministic MEM8 context retrieval and a suitable Kompress predictor to improve next-byte probabilities or exact matches, then encode exact residuals with a lossless entropy coder. Decoder state must be reproducible from already decoded bytes, or all additional state, model weights, and metadata must be included in accounted size. Semantic summaries, pruning, and hashes alone cannot restore exact source bytes.

Next: inspect actual sibling encoder/decoder APIs, run isolated round-trip tests, compare identical corpus slices with all overhead included, and independently ablate DC10 categories. Then validate promising candidates on full enwik9 under verified official constraints.
