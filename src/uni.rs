use anyhow::Result;

// MQ2-UNI: UTF-8 safe encoding with ASCII escape sequences
// FIXED: No more collision with UTF-8 continuation bytes!

pub const MQ2_UNI_DICT_ID: &str = "mq2-uni-v3-wiki";

const ESC: u8 = b'~'; // The marqant sigil for escape sequences

/// ASCII-safe token mappings using escape sequences.
///
/// Patterns are matched left-to-right, longest-first within each position.
/// Tokens that are SHORTER than their pattern save bytes directly;
/// others normalise structure for downstream gzip/zlib or AI consumption.
fn get_token_map() -> Vec<(&'static [u8], &'static [u8])> {
    vec![
        // ── Wikipedia / multi-level headers (all save bytes) ──────────────
        (b"##### ", b"~H5"), // 6 bytes -> 3 bytes  (saves 3)
        (b"#### ", b"~H4"),  // 5 bytes -> 3 bytes  (saves 2)
        (b"### ", b"~H3"),   // 4 bytes -> 3 bytes  (saves 1)
        (b"## ", b"~H2"),    // 3 bytes -> 3 bytes  (break-even; normalises)
        (b"# ", b"~H1"),     // 2 bytes -> 3 bytes  (structural marker)
        // ── Block-level markdown ───────────────────────────────────────────
        (b"\n\n\n", b"~TB"), // triple break  (must come before ~PP)
        (b"\n\n", b"~PP"),   // paragraph break
        (b"\n- ", b"~LI"),   // unordered list item
        (b"\n* ", b"~LB"),   // alt list item (Wikipedia uses *)
        (b"\n> ", b"~BQ"),   // blockquote
        (b"\n1. ", b"~N1"),  // 4 bytes -> 3 bytes  (saves 1; ordered list)
        (b"\n2. ", b"~N2"),  // 4 bytes -> 3 bytes
        (b"\n3. ", b"~N3"),  // 4 bytes -> 3 bytes
        (b"\n4. ", b"~N4"),  // 4 bytes -> 3 bytes
        (b"\n5. ", b"~N5"),  // 4 bytes -> 3 bytes
        // ── Code blocks ───────────────────────────────────────────────────
        (b"```\n", b"~CB"),  // code block open  (```<newline>)
        (b"```", b"~CE"),    // code block close / inline fence
        // ── Inline / spacing ──────────────────────────────────────────────
        (b"    ", b"~IN"),   // 4-space indent  (saves 1)
        (b"  ", b"~SP"),     // double space
        (b": ", b"~CO"),     // colon-space  (very common in Wikipedia prose)
        (b", ", b"~CM"),     // comma-space
        // ── JSON / code-block structural chars ───────────────────────────
        (b"{\n", b"~OB"),    // open-brace newline
        (b"}\n", b"~CL"),    // close-brace newline
        (b"[\n", b"~OS"),    // open-bracket newline
        (b"\n]", b"~CS"),    // close-bracket newline
    ]
}

/// Skip to the next UTF-8 character boundary
fn skip_utf8_char(bytes: &[u8], i: usize) -> usize {
    if i >= bytes.len() {
        return i;
    }

    let b = bytes[i];
    if b < 0x80 {
        return i + 1; // ASCII - single byte
    }

    // Multi-byte UTF-8 sequence
    let len = if b & 0b1111_0000 == 0b1111_0000 {
        4
    }
    // 4-byte char
    else if b & 0b1110_0000 == 0b1110_0000 {
        3
    }
    // 3-byte char
    else if b & 0b1100_0000 == 0b1100_0000 {
        2
    }
    // 2-byte char
    else {
        1
    }; // Continuation byte or invalid - skip conservatively

    (i + len).min(bytes.len())
}

pub fn mq2_uni_encode(input: &[u8]) -> Result<Vec<u8>> {
    let token_map = get_token_map();
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        // Try to match patterns (only in ASCII range)
        if input[i] < 0x80 {
            let mut matched = false;

            for (pattern, token) in &token_map {
                if i + pattern.len() <= input.len() && &input[i..i + pattern.len()] == *pattern {
                    out.extend_from_slice(token);
                    i += pattern.len();
                    matched = true;
                    break;
                }
            }

            if matched {
                continue;
            }
        }

        // Handle UTF-8 properly - copy entire character
        let next_i = skip_utf8_char(input, i);
        out.extend_from_slice(&input[i..next_i]);
        i = next_i;
    }

    Ok(out)
}

pub fn mq2_uni_decode(input: &[u8]) -> Result<Vec<u8>> {
    let token_map = get_token_map();
    let mut out = Vec::with_capacity(input.len() * 2);
    let mut i = 0;

    while i < input.len() {
        // Check for escape sequences
        if i + 2 < input.len() && input[i] == ESC {
            let mut decoded = false;

            // Try 3-byte tokens first (~XX)
            if i + 3 <= input.len() {
                let token = &input[i..i + 3];
                for (pattern, tok) in &token_map {
                    if *tok == token {
                        out.extend_from_slice(pattern);
                        i += 3;
                        decoded = true;
                        break;
                    }
                }
            }

            if decoded {
                continue;
            }
        }

        // Not a token - copy UTF-8 character
        let next_i = skip_utf8_char(input, i);
        out.extend_from_slice(&input[i..next_i]);
        i = next_i;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_preservation() {
        let test_cases = vec![
            "Hello 👋 World! 🌍",
            "Rust 🦀 is awesome! 🚀",
            "😀😢😡🥰 emotions",
            "Complex: 你好世界 🎯 مرحبا 🌟",
        ];

        for original in test_cases {
            let bytes = original.as_bytes();
            let encoded = mq2_uni_encode(bytes).unwrap();
            let decoded = mq2_uni_decode(&encoded).unwrap();

            assert_eq!(
                bytes,
                decoded.as_slice(),
                "Failed to preserve: {}",
                original
            );

            // Verify we can reconstruct the string
            let reconstructed = String::from_utf8(decoded).unwrap();
            assert_eq!(original, reconstructed);
        }
    }

    #[test]
    fn test_markdown_patterns() {
        let markdown = "# Title\n\n## Subtitle\n\n- Item 1\n- Item 2";
        let bytes = markdown.as_bytes();

        let encoded = mq2_uni_encode(bytes).unwrap();
        let decoded = mq2_uni_decode(&encoded).unwrap();

        assert_eq!(bytes, decoded.as_slice());

        // Note: ASCII escapes may be longer than originals for short strings
        // What matters is correctness, not size for small test cases
    }

    #[test]
    fn test_utf8_boundaries() {
        // Test that we never split multi-byte sequences
        let text = "UTF-8: €£¥ Emoji: 👨‍👩‍👧‍👦 Chinese: 中文";
        let bytes = text.as_bytes();

        let encoded = mq2_uni_encode(bytes).unwrap();
        let decoded = mq2_uni_decode(&encoded).unwrap();

        assert_eq!(bytes, decoded.as_slice());
    }
}
