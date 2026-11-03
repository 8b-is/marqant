//! Public-API conformance for the template/field/exception experiment.
use marqant::salient_templates::{compress, decompress, Options};

fn options(context: bool, lattice_literals: bool, max_templates: u16) -> Options {
    Options {
        max_templates,
        max_record: 512,
        min_gain: 32,
        context,
        lattice_literals,
    }
}

#[test]
fn alternating_templates_reconstruct_exact_original_fields() {
    let mut source = Vec::new();
    for i in 0..1000 {
        source.extend_from_slice(format!("  <number>{:08}</number>\r\n", i).as_bytes());
        source.extend_from_slice(format!("  <value>name {:06} &amp; café</value>\n", i).as_bytes());
    }
    for context in [false, true] {
        for lattice_literals in [false, true] {
            let opts = options(context, lattice_literals, 32);
            let encoded = compress(&source, opts).unwrap();
            assert!(encoded.templates > 0);
            assert!(encoded.template_records > 1000);
            if context {
                assert!(encoded.predicted_records > 1000);
            } else {
                assert_eq!(encoded.predicted_records, 0);
            }
            assert!(encoded.field_bytes > 0);
            assert_eq!(decompress(&encoded.bytes, source.len()).unwrap(), source);
            assert_eq!(compress(&source, opts).unwrap().bytes, encoded.bytes);
        }
    }
}

#[test]
fn arbitrary_binary_long_lines_and_unterminated_tail_are_preserved() {
    let mut source = (0u8..=255).collect::<Vec<_>>();
    source.extend(std::iter::repeat_n(b'x', 20_000));
    source.extend_from_slice(b"\r\n\0\0\xff\xfe\n");
    for i in 0..180 {
        source.extend_from_slice(b"<field>\xff\0<not-a-tag>|");
        source.extend_from_slice(i.to_string().as_bytes());
        source.extend_from_slice(b"</field>\n");
    }
    source.extend_from_slice(b"final tail without newline \0\xff");
    for context in [false, true] {
        for lattice_literals in [false, true] {
            for count in [0, 4, 32] {
                let encoded = compress(&source, options(context, lattice_literals, count)).unwrap();
                assert_eq!(decompress(&encoded.bytes, source.len()).unwrap(), source);
                if count == 0 {
                    assert_eq!(encoded.templates, 0);
                    assert_eq!(encoded.literal_bytes, source.len());
                }
            }
        }
    }
}

#[test]
fn empty_zero_promotion_and_explicit_output_bounds() {
    for lattice_literals in [false, true] {
        let empty = compress(b"", options(true, lattice_literals, 32)).unwrap();
        assert_eq!(decompress(&empty.bytes, 0).unwrap(), b"");
        let source = b"<item>00123</item>\n".repeat(100);
        let opts = Options {
            min_gain: u32::MAX,
            ..options(true, lattice_literals, 32)
        };
        let encoded = compress(&source, opts).unwrap();
        assert_eq!(encoded.templates, 0);
        assert_eq!(encoded.literal_bytes, source.len());
        assert_eq!(encoded.estimated_gain_bits, 0);
        assert_eq!(decompress(&encoded.bytes, source.len()).unwrap(), source);
        assert!(decompress(&encoded.bytes, source.len() - 1).is_err());
    }
}

#[test]
fn truncation_trailing_bytes_and_corruption_never_silently_change_output() {
    let source = b" <item>000001</item>\n <other>name</other>\n".repeat(100);
    for backend in [false, true] {
        let encoded = compress(&source, options(true, backend, 16)).unwrap();
        for end in 0..encoded.bytes.len() {
            assert!(
                decompress(&encoded.bytes[..end], source.len()).is_err(),
                "truncation {}",
                end
            );
        }
        let mut trailing = encoded.bytes.clone();
        trailing.push(0);
        assert!(decompress(&trailing, source.len()).is_err());
        for i in 0..encoded.bytes.len() {
            let mut damaged = encoded.bytes.clone();
            damaged[i] ^= 0x80;
            if let Ok(restored) = decompress(&damaged, source.len()) {
                assert_eq!(restored, source, "mutation at {}", i);
            }
        }
    }
}
