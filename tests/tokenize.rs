use marqant::Marqant;

#[test]
fn tokenize_reduces_length_for_repetitions() {
    // Repeat header enough times so static token "# " yields positive savings
    let content = "# Title\n\n".repeat(10);
    let (tokens, tokenized) = Marqant::tokenize_content(&content);
    assert!(!tokens.is_empty());
    assert!(tokenized.len() < content.len());
}

#[test]
fn tokenize_static_tokens_present() {
    // Ensure "## " and "- " appear enough times to meet savings > 0
    let mut content = String::new();
    for _ in 0..4 {
        content.push_str("## Heading\n\n");
    }
    content.push_str("- a\n- b\n- c\n- d\n- e\n- f\n");

    let (tokens, _tokenized) = Marqant::tokenize_content(&content);
    let has_expected = tokens.values().any(|v| v == "## " || v == "- ");
    assert!(has_expected);
}
