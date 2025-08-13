use marqant::Marqant;

#[test]
fn std_tokens_omitted_from_dict_and_applied_on_decode() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    // Input uses patterns covered by std-static-v1 (e.g., "# ", "## ", "- ")
    let md = "# T\n\n## H\n\n- a\n- b\n";
    let mq =
        Marqant::compress_markdown_with_flags(md, Some("-std:std-static-v1")).expect("compress");
    let lines: Vec<&str> = mq.lines().collect();
    assert!(lines[0].contains("-std:std-static-v1"));
    let sep = lines.iter().position(|l| *l == "---").expect("sep");
    let dict = &lines[1..sep];
    // Ensure dictionary does NOT redundantly include # , ## , -  entries
    for l in dict {
        assert!(!l.contains("=# "));
        assert!(!l.contains("=## "));
        assert!(!l.contains("=- "));
    }
    let back = Marqant::decompress_marqant(&mq).expect("decompress");
    assert_eq!(md.trim(), back.trim());
}
