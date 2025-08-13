use anyhow::{anyhow, Result};

// MQ2-UNI: tiny fixed dictionary demo (UTF-8-safe byte sequences)
// This is intentionally small and deterministic; decoder does not depend on AI.

pub const MQ2_UNI_DICT_ID: &str = "mq2-uni-demo-0001"; // placeholder stable ID

fn base_map() -> &'static [(&'static [u8], u8)] {
    // Map common UTF-8-safe sequences to tokens 0x80..0x8F (demo subset)
    // Reserved ranges (per spec idea): 0x00-0x1F control; 0x20-0x7E ASCII passthrough; 0xFF reserved
    &[
        (b"\n\n", 0x80),
        (b"  ", 0x81),
        (b"\n- ", 0x82),
        (b"## ", 0x83),
        (b"# ", 0x84),
        (b"```\n", 0x85),
        (b"```", 0x86),
        (b"{\n", 0x87),
        (b"}\n", 0x88),
        (b"[\n", 0x89),
        (b"\n]", 0x8A),
        (b": ", 0x8B),
        (b", ", 0x8C),
        (b"\"", 0x8D),
        (b"    ", 0x8E),
        (b"\n\n\n", 0x8F),
    ]
}

pub fn mq2_uni_encode(input: &[u8]) -> Result<Vec<u8>> {
    // Greedy longest-match replacement using base_map; ASCII passthrough; others raw
    let dict = base_map();
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        // Try matches
        let mut matched = false;
        for &(pat, tok) in dict {
            if i + pat.len() <= input.len() && &input[i..i + pat.len()] == pat {
                out.push(tok);
                i += pat.len();
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }
        let b = input[i];
        out.push(b); // passthrough (text-safe variant would escape non-ASCII if needed)
        i += 1;
    }
    Ok(out)
}

pub fn mq2_uni_decode(input: &[u8]) -> Result<Vec<u8>> {
    let dict = base_map();
    let mut out = Vec::with_capacity(input.len());
    'outer: for &b in input {
        if (0x80..=0x8F).contains(&b) {
            for &(pat, tok) in dict {
                if tok == b {
                    out.extend_from_slice(pat);
                    continue 'outer;
                }
            }
            return Err(anyhow!("unknown MQ2-UNI token: 0x{:02X}", b));
        }
        out.push(b);
    }
    Ok(out)
}
