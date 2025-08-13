use marqant::Marqant;

#[test]
fn compress_semantic_inserts_section_markers_and_roundtrips() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let md = "# Title\n\n## Sub\n\nContent\n```rust\n# not a section\n```\n";
    let mq = Marqant::compress_markdown_with_flags(md, Some("-semantic")).expect("compress");
    assert!(mq.starts_with("MARQANT 0"));
    assert!(mq.contains("::section:Title::"));
    assert!(mq.contains("::section:Sub::"));
    // Ensure code block text did not get section-tagged
    assert!(mq.contains("```rust"));
    assert!(!mq.contains("::section:# not a section::"));
    let back = Marqant::decompress_marqant(&mq).expect("decompress");
    assert_eq!(md.trim(), back.trim());
}

#[test]
fn compress_zlib_roundtrip() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let md = "# Title\n\n## Sub\n\nContent\n";
    let mq = Marqant::compress_markdown_with_flags(md, Some("-zlib")).expect("compress");
    let lines: Vec<&str> = mq.lines().collect();
    assert!(lines[0].contains("-zlib"));
    let sep = lines.iter().position(|l| *l == "---").expect("separator");
    let body = lines[(sep + 1)..].join("\n");
    assert!(!body.is_empty());
    // Base64-encoded body should be all ASCII printable or '=' padding
    assert!(body
        .chars()
        .all(|c| c.is_ascii() && (c.is_alphanumeric() || "+/=".contains(c))));
    let back = Marqant::decompress_marqant(&mq).expect("decompress");
    assert_eq!(md.trim(), back.trim());
}

#[test]
fn compress_zlib_and_semantic_roundtrip() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let md = "# Title\n\n## Sub\n\nContent\n";
    let mq = Marqant::compress_markdown_with_flags(md, Some("-zlib -semantic")).expect("compress");
    let header = mq.lines().next().unwrap();
    assert!(header.contains("-zlib"));
    assert!(header.contains("-semantic"));
    let back = Marqant::decompress_marqant(&mq).expect("decompress");
    assert_eq!(md.trim(), back.trim());
}
