// MQ2-UNI reference encoder/decoder (deterministic, fixed dictionary)
// Minimal, no-alloc hot path where possible.

pub struct Dictionary {
    // token -> bytes
    pub tokens: Vec<(u8, Vec<u8>)>,
}

impl Dictionary {
    pub fn new(tokens: Vec<(u8, Vec<u8>)>) -> Self { Self { tokens } }
}

// Simple header: MQ2~UNI~<ts_hex>~<orig_hex>~<comp_hex>~<tokc_hex>~text
fn header(ts_hex: &str, orig: usize, comp: usize, tokc: usize) -> String {
    format!("MQ2~UNI~{}~{:X}~{:X}~{:X}~text\n", ts_hex, orig, comp, tokc)
}

pub fn encode(input: &[u8], dict: &Dictionary, ts_hex: &str) -> Vec<u8> {
    // Build lookup by first byte for greedy longest match
    let mut buckets: Vec<Vec<(usize, u8, Vec<u8>)>> = vec![Vec::new(); 256];
    for (tok, pat) in &dict.tokens {
        if let Some(&b0) = pat.first() {
            buckets[b0 as usize].push((pat.len(), *tok, pat.clone()));
        }
    }
    for bucket in buckets.iter_mut() {
        bucket.sort_by(|a, b| b.0.cmp(&a.0)); // longest first
    }

    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i];
        let mut matched = false;
        for (len, tok, pat) in &buckets[b0 as usize] {
            if i + *len <= input.len() && &input[i..i + *len] == &pat[..] {
                out.push(*tok);
                i += *len;
                matched = true;
                break;
            }
        }
        if !matched {
            out.push(input[i]); // passthrough
            i += 1;
        }
    }

    // Build ~T map (sorted by token)
    let mut tok_map = dict.tokens.clone();
    tok_map.sort_by_key(|(t, _)| *t);
    let mut t_section = Vec::new();
    t_section.extend_from_slice(b"~T");
    for (tok, pat) in tok_map {
        t_section.push(tok);
        let len = pat.len() as u16;
        t_section.extend_from_slice(&len.to_be_bytes());
        t_section.extend_from_slice(&pat);
    }
    t_section.extend_from_slice(b"\n~~~~\n");

    let header = header(ts_hex, input.len(), out.len(), dict.tokens.len());
    let mut result = header.into_bytes();
    result.extend_from_slice(&t_section);
    result.extend_from_slice(&out);
    result
}

pub fn decode(encoded: &[u8]) -> Option<Vec<u8>> {
    // Parse header line
    let mut parts = encoded.splitn(2, |&b| b == b'\n');
    let header = parts.next()?;
    let rest = parts.next()?;
    if !header.starts_with(b"MQ2~UNI~") { return None; }

    // Parse ~T section until ~~~~\n
    if !rest.starts_with(b"~T") { return None; }
    let mut i = 2;
    let mut tok_map: Vec<Option<Vec<u8>>> = vec![None; 256];
    while i + 1 < rest.len() {
        if i + 6 <= rest.len() && &rest[i..i+6] == b"\n~~~~\n" { i += 6; break; }
        let tok = rest[i];
        let len = u16::from_be_bytes([rest[i+1], rest[i+2]]) as usize;
        i += 3;
        if i + len > rest.len() { return None; }
        tok_map[tok as usize] = Some(rest[i..i+len].to_vec());
        i += len;
    }
    let stream = &rest[i..];
    let mut out = Vec::with_capacity(stream.len());
    for &b in stream {
        if let Some(pat) = &tok_map[b as usize] { out.extend_from_slice(pat) } else { out.push(b) }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_basic() {
        let dict = Dictionary::new(vec![
            (0x80, b"## ".to_vec()),
            (0x81, b"# ".to_vec()),
            (0x82, b"\n\n".to_vec()),
            (0x83, b"**".to_vec()),
        ]);
        let input = b"# T\n\n## H\n\n**x**";
        let enc = encode(input, &dict, "00000000");
        let dec = decode(&enc).unwrap();
        assert_eq!(input, &dec[..]);
    }
}


