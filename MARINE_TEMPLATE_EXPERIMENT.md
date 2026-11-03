# Context, salient templates, and exact exceptions

## Evaluation status

Completed on 2026-11-02 (Asia/Tokyo). On the complete 100,000,000-byte local enwik8, the frozen template pipeline produces **35,395,976 bytes**, versus **35,588,067** for the preceding lattice pipeline and **35,736,965** for DC10. It saves another **192,091 bytes (approximately 0.54%)** beyond the prior best, with all dictionary, control, field, literal, and framing bytes counted. Separate-process decoder-only restoration passed external byte comparison and SHA-256 checks. This is a bounded, line-oriented prototype, not a prize submission.

## Hypothesis

Repeated flow can be represented as a learned pattern, its exact changing fields, and literal exceptions. Compression salience means expected coding-cost reduction, not semantic importance. A pattern should earn back its definition and reference costs before it is retained. Every byte not explained by a pattern remains in the exception stream.

This prototype is deliberately bounded and line-oriented, not an optimal grammar search. It does not claim to find the globally shortest description. Candidate shapes include exact lines, digit-run skeletons, and XML-like prefix/middle/suffix records; definitions are learned from the actual input, not an external Wikipedia dictionary. Candidate selection uses integer accounting. Actual compressed archive bytes, not the estimated score, decide whether the experiment helps.

## Marine lineage and important distinctions

Read-only audit of local `/data/8b-is/mem8-runtime` commit `3143ef049c59b6b097d69349cfbd8dd08d86cf6c` found genuine integer-native Marine processing in `crates/marine-core/src/marine_int.rs`. It detects strict three-sample peaks/valleys and compares periods/amplitudes against EMAs, combining amplitude, stability, and harmonic alignment. Its configured score is based on rational arithmetic and quantization; it is not a text-compression bit-cost model. The floating historical salience wrapper is not used here.

The conceptual link is stable recurrence, selective retention, and context-dependent predictability. This prototype replaces the signal-domain score with an integer raw-byte promotion heuristic and explicitly preserves all exceptions. That heuristic is not a bound on compressed-bit savings: independent candidates overlap, zlib can expand fields, and splitting streams can lose cross-stream redundancy. It does **not** import Marine's detector, EMA, harmonic analysis, or MEM8 wave math. No byte is discarded for low salience.

G8T search for `template compression` returned no capsule. Salience search returned existing scalar salience steps (including `qwave_marine_jitter_salience`, equivalence class `(u64)->u64::b0335fa708b6`), but those are not byte-reconstructing template codecs. Their proposed conceptual interface joint would be signal salience to coding-cost salience; no capsule implementation source was reused.

## Planned comparisons

- Same raw input bytes, same zlib level 9 backend.
- Compare unchanged DC10 and the existing context-lattice prototype.
- Dictionary/fields/control/exception streams and every header counted.
- Context transition prediction on/off at identical template settings.
- No-template control to isolate literal-backend/framing effects.
- Plain zlib exception backend versus the existing integer DC10+lattice codec on exceptions.
- Small-prefix tuning followed by frozen parameters on a separate slice and the full file.
- Exact byte comparisons, independent saved-archive decode, SHA-256, integer-nanosecond timing, and process peak RSS.

## Development entry points

- Codec: `src/salient_templates.rs`.
- Driver: `src/bin/salient_template_arena.rs`.
- Independent tests: `tests/salient_template_conformance.rs`.
- Serial measurement script: `examples/bench_salient_templates.py`.

The script creates new CSV files exclusively and stops on a failed child or missing exact round trip. It uses integer `time.monotonic_ns()` and per-child Linux `os.wait4` peak RSS. A validation CSV is not complete until the script exits successfully.

## Implementation and framing

The candidate pool is bounded to 4,096 patterns using deterministic Space-Saving counts. A second pass discards the approximate estimates and exactly re-counts survivors. A raw-byte heuristic charges literal definitions, reference/field-length allowances, and stream metadata before promotion. Up to 256 templates can be retained; selection ties have fixed ordering. Candidate scores can overlap and do not guarantee compressed savings.

A selected template is a sequence of fixed byte segments with up to eight exact variable fields. Records split at LF, including the delimiter; CRLF, indentation, invalid UTF-8, leading zeros and unterminated final lines remain unchanged. Records outside the configured size limit are literal exceptions. The decoder does not repeat template learning: all definitions and chosen transitions are serialized.

STP1 has a 96-byte fixed header, a 16-byte raw/stored length pair per template, and separately framed streams. Dictionary, program and each template's field payload are complete independent zlib level-9 members. Exceptions are either zlib or a complete M8L1 archive using the earlier DC10+lattice codec with context length 4 and minimum match 16. Program tokens describe literal runs, explicit template IDs, or a predicted next template, followed by exact field lengths. The source documents the full byte-level format.

## Selection protocol and evidence

`MARINE_TEMPLATE_TUNING_2026-11-02.csv` contains ten successful tuning runs on the first 2,000,000 bytes: 32/128 template limits, 512-byte eligible records, context prediction on/off, and plain-zlib versus lattice-coded exceptions; two additional runs checked 4,096-byte records. Minimum raw promotion margin was 64 bytes. No timings were used for parameter selection.

The smallest archive for each exception backend used **32 templates, 512-byte records, margin 64, next-template prediction off**. These settings were frozen before validation. `MARINE_TEMPLATE_VALIDATION_2026-11-02.csv` contains eight successful runs: four ablations on a separate 2 MB slice at byte offset 50,000,000, then the same four on the full file. Definitions are learned anew from each input and serialized; this is not an unaccounted pretrained dictionary. Full-file results include the tuning prefix and are not evidence on an independent unseen corpus.

| Pipeline | Tuning 2 MB | Separate 2 MB slice | Full 100 MB |
|---|---:|---:|---:|
| zlib level 9 | 722,405 | 734,995 | 36,460,101 |
| DC10 + zlib level 9 | 707,652 | 720,462 | 35,736,965 |
| Previous DC10 + lattice pipeline | 704,514 | 719,090 | 35,588,067 |
| Templates + plain-zlib exceptions, no prediction | 713,494 | 728,462 | 35,947,854 |
| **Templates + lattice exceptions, no prediction** | **702,583** | **718,525** | **35,395,976** |
| Same templates/backend, next-template prediction on | 702,641 | 718,557 | 35,396,801 |
| STP1 framing with no templates, lattice exceptions | not tuned | 719,207 | 35,588,185 |

**What helped:** template/field separation adds a real archive gain on all three ranges when combined with the existing literal backend. The no-template control is 118 bytes larger than the underlying full-file lattice archive, isolating framing overhead. Relative to that wrapped control, template separation saves 192,209 bytes; relative to the actual previous archive it saves 192,091 bytes. Relative to original DC10, the combined improvement is 340,989 bytes, approximately 0.95%.

**What did not help:** plain-zlib exceptions still lose to both stronger baselines. The simple next-template predictor correctly predicted 120,159 of 150,891 template records (about 80%) but made the full archive **825 bytes larger**. High prediction accuracy alone is not evidence of lower coded cost; transition-table cost and interactions with the existing zlib model matter. All negative results remain in the CSVs.

## Patterns actually learned

The full-file winner retains 32 definitions and represents 150,891 records. Inspection of the saved archive, rather than a hypothetical example, found patterns including:

```text
      <timestamp>{year}-{month}-{day}T{hour}:{minute}:{second}Z</timestamp>\n
        <username>{exact bytes}</username>\n
    <title>{exact bytes}</title>\n
      <id>{exact bytes}</id>\n
        <ip>{a}.{b}.{c}.{d}</ip>\n
      <comment>Revert to revision {digits} using [[:en:Wikipedia:Tools/Navigation_popups|popups]]</comment>\n
```

Indentation and line terminators above belong to the definitions. There are also exact templates such as `==External links==\n`, `==References==\n`, and contributor/revision boundary lines. All were selected from data, not entered as a hardcoded dictionary. Both generic and more specific timestamp/IP patterns can be promoted; overlapping or unused templates still cost bytes and are not silently omitted from accounting.

The winner has 95,633,357 raw exception bytes and 1,132,273 raw field bytes. Templates explain the remaining 3,234,370 fixed source bytes. The dictionary is 1,167 bytes before zlib. These intermediate quantities are **not** the final archive gain. Its promotion heuristic reports 24,453,272 estimated bits across independently scored candidates, not measured entropy savings.

### Complete saved-archive byte accounting

| Stored component | Bytes |
|---|---:|
| Fixed header plus 32 field-length pairs | 608 |
| Compressed dictionary, including next-table entries | 513 |
| Compressed program | 111,710 |
| Complete lattice-coded exception archive | 34,874,106 |
| All 32 compressed field streams, including empty members | 409,039 |
| **Total STP1 archive** | **35,395,976** |

No external dictionary, model weights, memory snapshot, or remote service is needed. The archive is 2.83167808 bits per original byte.

## Decoder size and standalone resources

New decoder-only entry points deliberately omit learning, compression and benchmarking calls:

| Release decoder artifact | Executable bytes | Archive bytes | Sum |
|---|---:|---:|---:|
| `target/release/lattice_decode` | 611,544 | 35,588,067 | 36,199,611 |
| `target/release/template_decode` | 643,752 | 35,395,976 | 36,039,728 |

Counting these respective unstripped decoder-only release executables, the new pipeline is still **159,883 bytes smaller**. Both were built with the same Cargo release defaults. These sums are controlled development comparisons, **not official Hutter Prize totals**: packaging, runtime dependencies, official corpus/rules/limits and minimized submitted executables have not been validated. The multipurpose new arena executable is 932,800 bytes and is not substituted for the decoder in this comparison.

`MARINE_TEMPLATE_RESOURCES_2026-11-02.csv` records four serial, separate-process measurements with no other benchmark intentionally running alongside them. One invocation per mode; timings are indicative, not a statistical speed claim. Wall times include startup and file I/O.

| Process | Wall ns | Peak RSS KiB |
|---|---:|---:|
| Prior lattice compress | 11,228,110,288 | 345,360 |
| New template compress | 13,131,372,622 | 442,244 |
| Prior lattice decoder-only | 2,778,539,570 | 310,844 |
| New template decoder-only | 2,712,440,503 | 303,444 |

Encoding used more time and memory. Decode time/memory were comparable in this single run; do not infer a reliable speed improvement from a small timing difference. Full comparison-arena peak memory was 555,300 KiB for the chosen pipeline; it retains multiple baselines and buffers and is not standalone decoder memory.

## Verification, identities, and limitations

- Base marqant commit: `fbfb9457cdb73441fa9dd5c8af5db423a41e02f5`, plus the prior lattice additions and this experiment's working-tree files.
- Toolchain/backend: rustc 1.99.1, cargo 1.99.1, flate2 1.1.2 / miniz_oxide 0.8.9, Linux x86_64; same environment as the earlier baseline.
- Corpus: `/data/corpus/enwik8`, exactly 100,000,000 bytes. Corpus and both restored files SHA-256: `2b49720ec4d78c3c9fabaee6e4179a5e997302b3a70029f30f2d582218c024a8`.
- Saved prior archive: `target/marine-control.m8l`, SHA-256 `45fa9ba83889a1445a67957c764792a92fe88c91a0b33ea444edc5df3207e59f`.
- Saved new archive: `target/salient-c32-r512-g64.stp`, SHA-256 `478a7e34776bec0e73028ece395dd2c9305bd757c582c52a475b0186e5614919`.
- Separate-process outputs: `target/marine-control-restored.enwik8`, `target/salient-restored.enwik8`. Both passed external `cmp` against the source and SHA-256 matching.
- Binary SHA-256: arena `39ff74078839370aac7fa700f532007d56ac784b062329cbe0bd8ef6f9387a3e`; template decoder `f7c3a283c1c9a7f790a4a1f0f0838fc9d9374d5e8df6ffad71db79ea324ea69d`; lattice decoder `14773ee73422f98f69c9ba3c7ff48ed6f561a06d2fa45ca8f6216ade836414e9`.
- `cargo test --lib`: **59 passed**, including 11 template unit tests, nine lattice unit tests and the existing Hutter tests. Both relevant integration targets passed: **four template + four lattice conformance tests**. The unrelated DNS failures documented in the earlier baseline were not claimed fixed; this is not a full-repository test-suite pass.
- Tests cover deterministic output, fields/delimiters, binary/UTF-8/CRLF/leading zeros, long/final records, no-template and promotion-off paths, context/backend combinations, saturated candidate pools, malformed headers/tokens/dictionaries, exact stream consumption, missing/trailing/concatenated zlib members, output limits and corruption.
- A static review found theoretical-bound overflow in `usize` on 32-bit platforms. Comparisons were widened to `u64`, without changing encoding or wire format; regression tests passed. No native 32-bit execution was available.
- Decoder requires complete zlib members, canonical varints, bounded declared lengths, valid dictionaries/context IDs, exact field/control/literal consumption, output length and FNV integrity. FNV is not authentication. `max_output` limits original output bytes, not aggregate working memory. Experimental format, not production hardening certification.
- New codec, drivers, decoder-only entry points and benchmark arithmetic are integer-only at source level. This does not claim all legacy marqant/MEM8 APIs are float-free or that all linked machine instructions were audited.
- Cargo's incidental root-package lockfile version refresh was reverted; no dependency versions or existing codec logic were changed.

## Reproduce with fresh output names

```sh
cargo test --lib
cargo test --test salient_template_conformance --test mem8_lattice_conformance
cargo build --release --bin salient_template_arena --bin template_decode --bin lattice_decode

python3 examples/bench_salient_templates.py tune --output target/reproduction-template-tuning.csv
python3 examples/bench_salient_templates.py validate --tuning target/reproduction-template-tuning.csv --output target/reproduction-template-validation.csv

./target/release/salient_template_arena compress /data/corpus/enwik8 target/reproduction.stp --templates 32 --record-bytes 512 --min-gain 64 --no-context --lattice-literals
./target/release/template_decode target/reproduction.stp target/reproduction.enwik8 100000000
cmp /data/corpus/enwik8 target/reproduction.enwik8
```

Use fresh CSV names for reruns; the script intentionally refuses to overwrite them. Files under `target/` are ignored, reproducible development artifacts. Definitions and all residuals remain counted regardless of whether the result wins.

## Recommendation

Keep the measured template + field separation as an experimental candidate. Keep simple next-template prediction disabled for this backend: it increased size. This demonstrates a modest benefit from describing recurring structure and exact changing fields; it does not prove global minimum description length, production Marine integration, arbitrary semantic reconstruction, or prize competitiveness.

Next improvements should target net compressed coding cost rather than raw-byte recurrence alone: prune unused/overlapping definitions, model template/field-length tokens separately, and compare stronger context entropy coding with proper ablations. Explore larger-scale phrase/record grammars only with definition and exception costs included. Full enwik9 and verified prize constraints remain future validation, not results of this run.
