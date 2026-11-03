//! Independent public-API checks for the experimental integer lattice codec.
use marqant::mem8_lattice::{compress, decompress, Options};

#[test]
fn arbitrary_byte_roundtrips_across_contexts_and_modes() {
    for context_bytes in [4, 8] {
        for min_match in [4, 8, 16] {
            for dc10 in [false, true] {
                let options = Options {
                    context_bytes,
                    min_match,
                    dc10,
                };
                for seed in 0..12u64 {
                    let mut state = seed + 1;
                    let mut input = Vec::new();
                    for i in 0..(seed as usize * 137) {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        // Mix noise with repeats and boundaries, preserving arbitrary bytes.
                        let byte = if i % 23 < 15 {
                            (i % 7) as u8
                        } else {
                            state as u8
                        };
                        input.push(byte);
                    }
                    let encoded = compress(&input, options).unwrap();
                    assert_eq!(decompress(&encoded.bytes, input.len()).unwrap(), input);
                    assert_eq!(compress(&input, options).unwrap().bytes, encoded.bytes);
                }
            }
        }
    }
}

#[test]
fn truncated_containers_and_too_small_output_limits_fail() {
    let input = b"<page>[[United States|United States of America]]</page>".repeat(20);
    let encoded = compress(
        &input,
        Options {
            context_bytes: 8,
            min_match: 4,
            dc10: true,
        },
    )
    .unwrap();
    for end in 0..encoded.bytes.len() {
        assert!(
            decompress(&encoded.bytes[..end], input.len()).is_err(),
            "truncation at {}",
            end
        );
    }
    assert!(decompress(&encoded.bytes, input.len() - 1).is_err());
    let mut trailing = encoded.bytes.clone();
    trailing.push(0);
    assert!(decompress(&trailing, input.len()).is_err());
}

#[test]
fn every_static_dc10_pattern_passes_bounded_inverse_preflight() {
    let mut combined = Vec::new();
    for spec in marqant::hutter_demon::STATIC_DEMONS {
        combined.extend_from_slice(spec.pattern);
        combined.push(b'\n');
    }
    combined.extend_from_slice(b"[[computer|Computers]] [[Chicago, Illinois|Chicago]]");
    combined.extend(0u8..=255);
    for context_bytes in [4, 8] {
        let encoded = compress(
            &combined,
            Options {
                context_bytes,
                min_match: 16,
                dc10: true,
            },
        )
        .unwrap();
        assert_eq!(
            decompress(&encoded.bytes, combined.len()).unwrap(),
            combined
        );
    }
}

#[test]
fn repeated_runs_and_long_overlapping_matches_restore_exactly() {
    for dc10 in [false, true] {
        for period in [1usize, 2, 3, 17, 257] {
            let input: Vec<u8> = (0..100_000).map(|i| (i % period) as u8).collect();
            let encoded = compress(
                &input,
                Options {
                    context_bytes: 4,
                    min_match: 4,
                    dc10,
                },
            )
            .unwrap();
            assert!(encoded.matched_bytes > 90_000);
            assert_eq!(decompress(&encoded.bytes, input.len()).unwrap(), input);
        }
    }
}
