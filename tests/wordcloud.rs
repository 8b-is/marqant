use marqant::wordcloud_svg;

#[test]
fn svg_empty_minimal() {
    let svg = wordcloud_svg("", 200, 100);
    assert_eq!(
        svg,
        "<svg xmlns='http://www.w3.org/2000/svg' width='200' height='100' viewBox='0 0 200 100'/>"
    );
}

#[test]
fn svg_contains_words_and_is_deterministic() {
    let text = "Hello hello world";
    let a = wordcloud_svg(text, 320, 200);
    let b = wordcloud_svg(text, 320, 200);
    assert_eq!(a, b);
    assert!(a.starts_with("<svg "));
    assert!(a.contains("<text "));
    // Expect two unique words >=2 letters => 2 text nodes
    let count = a.match_indices("<text ").count();
    assert_eq!(count, 2);
    assert!(a.contains(">hello<"));
    assert!(a.contains(">world<"));
}
