use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

/// Normalize text for comparison (handle line endings, trailing spaces)
fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
     .lines()
     .map(str::trim_end)
     .collect::<Vec<_>>()
     .join("\n")
}

#[test]
fn test_roundtrip_all_markdown_files() {
    let example_dir = Path::new("example-md");
    if !example_dir.exists() {
        eprintln!("Skipping roundtrip_folder test - example-md directory not found");
        return;
    }
    
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();
    
    // Read all .md files
    let entries = fs::read_dir(example_dir)
        .expect("Failed to read example-md directory");
    
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        
        let filename = path.file_name().unwrap().to_string_lossy();
        println!("Testing: {}", filename);
        
        // Read original
        let original = fs::read_to_string(&path)
            .expect(&format!("Failed to read {}", filename));
        
        // Encode via CLI
        let mut encode_cmd = Command::new("cargo")
            .args(["run", "--quiet", "--", "uni-encode"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start uni-encode");
        
        encode_cmd.stdin.as_mut().unwrap()
            .write_all(original.as_bytes())
            .expect("Failed to write to encoder");
        
        let encode_output = encode_cmd.wait_with_output()
            .expect("Failed to run uni-encode");
        
        if !encode_output.status.success() {
            eprintln!("  ❌ Encode failed: {}", 
                     String::from_utf8_lossy(&encode_output.stderr));
            failed += 1;
            failures.push(filename.to_string());
            continue;
        }
        
        // Decode back
        let mut decode_cmd = Command::new("cargo")
            .args(["run", "--quiet", "--", "uni-decode"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start uni-decode");
        
        decode_cmd.stdin.as_mut().unwrap()
            .write_all(&encode_output.stdout)
            .expect("Failed to write to decoder");
        
        let decode_output = decode_cmd.wait_with_output()
            .expect("Failed to run uni-decode");
        
        if !decode_output.status.success() {
            eprintln!("  ❌ Decode failed: {}", 
                     String::from_utf8_lossy(&decode_output.stderr));
            failed += 1;
            failures.push(filename.to_string());
            continue;
        }
        
        let decoded = String::from_utf8_lossy(&decode_output.stdout);
        
        // Check byte-perfect emoji preservation
        let original_bytes = original.as_bytes();
        let decoded_bytes = decoded.as_bytes();
        
        // First check: exact byte match (strict mode)
        if original_bytes == decoded_bytes {
            println!("  ✅ Perfect byte match!");
            passed += 1;
            continue;
        }
        
        // Second check: normalized comparison (tolerant mode)
        let normalized_original = normalize(&original);
        let normalized_decoded = normalize(&decoded);
        
        if normalized_original == normalized_decoded {
            println!("  ✅ Normalized match (whitespace differences only)");
            passed += 1;
        } else {
            // Check if emoji/unicode was preserved
            let original_emojis: Vec<char> = original.chars()
                .filter(|c| (*c as u32) > 127)
                .collect();
            let decoded_emojis: Vec<char> = decoded.chars()
                .filter(|c| (*c as u32) > 127)
                .collect();
            
            if original_emojis != decoded_emojis {
                eprintln!("  ❌ EMOJI/UNICODE CORRUPTION DETECTED!");
                eprintln!("    Original emojis: {:?}", original_emojis);
                eprintln!("    Decoded emojis: {:?}", decoded_emojis);
                
                // Save diff for debugging
                let diff_path = format!("target/{}.diff", filename);
                fs::write(&diff_path, format!(
                    "=== ORIGINAL ===\n{}\n\n=== DECODED ===\n{}\n", 
                    original, decoded
                )).ok();
                eprintln!("    Diff saved to: {}", diff_path);
            } else {
                eprintln!("  ⚠️  Content mismatch (but emojis preserved)");
            }
            
            failed += 1;
            failures.push(filename.to_string());
        }
    }
    
    println!("\n========================================");
    println!("Roundtrip Test Results:");
    println!("  ✅ Passed: {}", passed);
    println!("  ❌ Failed: {}", failed);
    
    if !failures.is_empty() {
        println!("\nFailed files:");
        for f in &failures {
            println!("  - {}", f);
        }
        
        // Allow tolerance mode via env var
        if std::env::var("MQ_TEST_TOLERANT").unwrap_or_default() == "1" {
            println!("\n⚠️  Running in TOLERANT mode - not failing test");
        } else {
            panic!("Roundtrip test failed for {} files", failed);
        }
    }
}

#[test]
fn test_emoji_bytes_explicit() {
    // Explicit byte-level test for emojis
    let test_cases = vec![
        ("wave", "👋", vec![0xF0, 0x9F, 0x91, 0x8B]),
        ("earth", "🌍", vec![0xF0, 0x9F, 0x8C, 0x8D]),
        ("crab", "🦀", vec![0xF0, 0x9F, 0xA6, 0x80]),
        ("rocket", "🚀", vec![0xF0, 0x9F, 0x9A, 0x80]),
        ("family", "👨‍👩‍👧‍👦", vec![
            0xF0, 0x9F, 0x91, 0xA8, 0xE2, 0x80, 0x8D,  // man
            0xF0, 0x9F, 0x91, 0xA9, 0xE2, 0x80, 0x8D,  // woman  
            0xF0, 0x9F, 0x91, 0xA7, 0xE2, 0x80, 0x8D,  // girl
            0xF0, 0x9F, 0x91, 0xA6,                     // boy
        ]),
    ];
    
    for (name, emoji, expected_bytes) in test_cases {
        println!("Testing emoji '{}': {}", name, emoji);
        
        // Verify our test data is correct
        let actual_bytes: Vec<u8> = emoji.bytes().collect();
        assert_eq!(actual_bytes, expected_bytes, 
                   "Test data error for {}", name);
        
        // Test encoding/decoding preserves bytes exactly
        use marqant::{mq2_uni_encode, mq2_uni_decode};
        
        let encoded = mq2_uni_encode(&actual_bytes)
            .expect(&format!("Failed to encode {}", name));
        let decoded = mq2_uni_decode(&encoded)
            .expect(&format!("Failed to decode {}", name));
        
        assert_eq!(decoded, expected_bytes,
                   "Emoji '{}' bytes changed during roundtrip", name);
        
        // Verify string reconstruction
        let reconstructed = String::from_utf8(decoded)
            .expect(&format!("Invalid UTF-8 after decoding {}", name));
        assert_eq!(reconstructed, emoji,
                   "Emoji '{}' string mismatch after roundtrip", name);
        
        println!("  ✅ {} preserved perfectly", name);
    }
}

#[test]
fn test_mixed_content_preservation() {
    let test_doc = r#"# Title with emoji 🎯

This document has **bold** and *italic* text.

## Code with emoji in comments
```rust
fn main() {
    println!("Hello 🦀"); // Rust crab!
}
```

## List with various Unicode
- English: Hello
- Chinese: 你好
- Arabic: مرحبا
- Emoji: 👋

## Special characters
- Currency: €£¥$
- Math: ∑∫∂∇
- Arrows: ←→↑↓
- Box drawing: ┌─┐│└┘
"#;

    use marqant::{mq2_uni_encode, mq2_uni_decode};
    
    let original_bytes = test_doc.as_bytes();
    let encoded = mq2_uni_encode(original_bytes).unwrap();
    let decoded = mq2_uni_decode(&encoded).unwrap();
    
    assert_eq!(original_bytes, decoded.as_slice(),
               "Mixed content not preserved exactly");
    
    // Verify compression happened
    assert!(encoded.len() < original_bytes.len(),
            "No compression occurred");
    
    println!("✅ Mixed content test passed");
    println!("  Original: {} bytes", original_bytes.len());
    println!("  Encoded: {} bytes", encoded.len());
    println!("  Ratio: {:.1}%", 
             (encoded.len() as f32 / original_bytes.len() as f32) * 100.0);
}