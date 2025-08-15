use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn test_emoji_preservation() {
    // Test that emojis are preserved through encoding/decoding
    let test_cases = vec![
        ("simple_emoji", "Hello 👋 World! 🌍"),
        ("markdown_with_emoji", "# Title 🎯\n\n- Item 1 ✅\n- Item 2 ❌\n- Item 3 🚀"),
        ("complex_unicode", "Emotions: 😀 😢 😡 🥰\nSymbols: ♠️ ♣️ ♥️ ♦️\nArrows: ↑ → ↓ ←"),
        ("mixed_content", "## Section 📊\n\nThis is **bold** and _italic_ with emoji 🎨\n\n```rust\nfn main() { println!(\"🦀\"); }\n```"),
    ];

    for (name, content) in test_cases {
        println!("\n=== Testing: {} ===", name);
        println!("Original: {}", content);

        // Encode
        let mut encode_cmd = Command::new("cargo")
            .args(["run", "--quiet", "--", "uni-encode"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start uni-encode");

        encode_cmd
            .stdin
            .as_mut()
            .unwrap()
            .write_all(content.as_bytes())
            .expect("Failed to write to stdin");

        let encode_output = encode_cmd
            .wait_with_output()
            .expect("Failed to run uni-encode");

        // Save encoded for inspection
        let encoded_path = format!("target/test_{}_encoded.mq", name);
        fs::write(&encoded_path, &encode_output.stdout).expect("Failed to write encoded file");

        println!("Encoded saved to: {}", encoded_path);
        println!(
            "Encoded bytes: {:?}",
            &encode_output.stdout[..encode_output.stdout.len().min(50)]
        );

        // Decode
        let mut decode_cmd = Command::new("cargo")
            .args(["run", "--quiet", "--", "uni-decode"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start uni-decode");

        decode_cmd
            .stdin
            .as_mut()
            .unwrap()
            .write_all(&encode_output.stdout)
            .expect("Failed to write encoded data to decoder");

        let decode_output = decode_cmd
            .wait_with_output()
            .expect("Failed to run uni-decode");

        let decoded = String::from_utf8_lossy(&decode_output.stdout);

        // Save decoded for inspection
        let decoded_path = format!("target/test_{}_decoded.md", name);
        fs::write(&decoded_path, decoded.as_bytes()).expect("Failed to write decoded file");

        println!("Decoded saved to: {}", decoded_path);
        println!("Decoded: {}", decoded);

        // Check if they match
        assert_eq!(
            content,
            decoded.as_ref(),
            "Round-trip failed for test case '{}'\nOriginal: {}\nDecoded: {}",
            name,
            content,
            decoded
        );

        println!("✅ Test '{}' passed!", name);
    }
}

#[test]
fn test_byte_level_emoji() {
    // Direct test of the encoding functions
    use marqant::{mq2_uni_decode, mq2_uni_encode};

    let test_emoji = "Hello 👋 World!";
    let bytes = test_emoji.as_bytes();

    println!("Original string: {}", test_emoji);
    println!("Original bytes: {:?}", bytes);

    let encoded = mq2_uni_encode(bytes).expect("Failed to encode");
    println!("Encoded bytes: {:?}", encoded);

    let decoded = mq2_uni_decode(&encoded).expect("Failed to decode");
    println!("Decoded bytes: {:?}", decoded);

    let decoded_str =
        String::from_utf8(decoded.clone()).expect("Decoded bytes are not valid UTF-8");
    println!("Decoded string: {}", decoded_str);

    assert_eq!(bytes, decoded.as_slice(), "Byte-level round-trip failed");
    assert_eq!(test_emoji, decoded_str, "String round-trip failed");
}
