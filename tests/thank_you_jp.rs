use marqant::{mq2_uni_decode, mq2_uni_encode};

#[test]
fn thank_you_bondee_in_japanese_roundtrip() {
    // A short thank-you message in Japanese
    let original =
        "この修正をありがとうございます！あなたのおかげで、この機能は日本でも正しく動作します。";

    // Encode → Decode
    let encoded = mq2_uni_encode(original.as_bytes()).unwrap();
    let decoded = mq2_uni_decode(&encoded).unwrap();

    let decoded_str = String::from_utf8(decoded).expect("Decoded output not valid UTF-8");

    // Check that round-trip preserved the original text exactly
    assert_eq!(
        decoded_str, original,
        "Round-trip failed for Japanese thank-you message"
    );

    println!("✅ Japanese thank-you message preserved perfectly!");
    println!("   Original: {}", original);
    println!("   Bytes: {} → {} (encoded)", original.len(), encoded.len());
}
