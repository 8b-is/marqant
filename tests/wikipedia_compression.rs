//! # Wikipedia Markdown Compression Tests
//!
//! Tests and benchmarks the Marqant compressor against Wikipedia-style
//! markdown content, covering:
//!
//! - Lossless roundtrip (uni-encode / uni-decode)
//! - Semantic compress/decompress roundtrip
//! - Compression ratios across multiple modes
//! - Performance with large files (>30 KB)

use marqant::{mq2_uni_decode, mq2_uni_encode, Marqant};
use std::fs;
use std::path::Path;
use std::time::Instant;

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

struct CompressionResult {
    original_bytes: usize,
    compressed_bytes: usize,
    ratio_pct: f64,
    elapsed_ms: u128,
}

impl CompressionResult {
    fn print(&self, label: &str) {
        println!(
            "  [{label}] {orig} bytes → {comp} bytes  ({ratio:.1}% of original, {ms}ms)",
            label = label,
            orig = self.original_bytes,
            comp = self.compressed_bytes,
            ratio = self.ratio_pct,
            ms = self.elapsed_ms,
        );
    }
}

fn compress_uni(content: &[u8]) -> CompressionResult {
    let start = Instant::now();
    let encoded = mq2_uni_encode(content).expect("uni-encode failed");
    let elapsed_ms = start.elapsed().as_millis();
    let ratio_pct = encoded.len() as f64 / content.len() as f64 * 100.0;
    CompressionResult {
        original_bytes: content.len(),
        compressed_bytes: encoded.len(),
        ratio_pct,
        elapsed_ms,
    }
}

fn compress_semantic(content: &str, flags: Option<&str>) -> CompressionResult {
    let start = Instant::now();
    let mq = Marqant::compress_markdown_with_flags(content, flags).expect("compress failed");
    let elapsed_ms = start.elapsed().as_millis();
    let ratio_pct = mq.len() as f64 / content.len() as f64 * 100.0;
    CompressionResult {
        original_bytes: content.len(),
        compressed_bytes: mq.len(),
        ratio_pct,
        elapsed_ms,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Roundtrip helpers
// ──────────────────────────────────────────────────────────────────────────────

fn assert_uni_roundtrip(content: &[u8], label: &str) {
    let encoded = mq2_uni_encode(content)
        .unwrap_or_else(|e| panic!("[{label}] uni-encode failed: {e}"));
    let decoded = mq2_uni_decode(&encoded)
        .unwrap_or_else(|e| panic!("[{label}] uni-decode failed: {e}"));
    assert_eq!(
        content, decoded.as_slice(),
        "[{label}] uni roundtrip mismatch"
    );
}

fn assert_semantic_roundtrip(content: &str, flags: Option<&str>, label: &str) {
    let mq = Marqant::compress_markdown_with_flags(content, flags)
        .unwrap_or_else(|e| panic!("[{label}] compress failed: {e}"));
    let back = Marqant::decompress_marqant(&mq)
        .unwrap_or_else(|e| panic!("[{label}] decompress failed: {e}"));
    assert_eq!(
        normalize(content),
        normalize(&back),
        "[{label}] semantic roundtrip mismatch"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

/// Verify that all Wikipedia fixture files survive a lossless uni roundtrip.
#[test]
fn test_wikipedia_fixtures_uni_roundtrip() {
    let fixture_dir = Path::new("tests/fixtures/wikipedia");
    if !fixture_dir.exists() {
        eprintln!("Skipping: fixture directory not found");
        return;
    }

    let mut count = 0;
    for entry in fs::read_dir(fixture_dir).expect("read fixture dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let label = path.file_name().unwrap().to_string_lossy().into_owned();
        let content = fs::read(&path).expect("read fixture");
        assert_uni_roundtrip(&content, &label);
        count += 1;
        println!("✅ uni roundtrip: {}", label);
    }
    assert!(count > 0, "No Wikipedia fixtures found — check tests/fixtures/wikipedia/");
}

/// Verify that Wikipedia fixture files survive a semantic compress/decompress roundtrip.
#[test]
fn test_wikipedia_fixtures_semantic_roundtrip() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let fixture_dir = Path::new("tests/fixtures/wikipedia");
    if !fixture_dir.exists() {
        eprintln!("Skipping: fixture directory not found");
        return;
    }

    for entry in fs::read_dir(fixture_dir).expect("read fixture dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let label = path.file_name().unwrap().to_string_lossy().into_owned();
        let content = fs::read_to_string(&path).expect("read fixture");

        // Default mode
        assert_semantic_roundtrip(&content, None, &format!("{} [default]", label));

        // Binary (zlib) mode — best for large files
        assert_semantic_roundtrip(&content, Some("-zlib"), &format!("{} [zlib]", label));

        println!("✅ semantic roundtrip: {}", label);
    }
}

/// Report compression ratios for all Wikipedia fixtures across all compression modes.
/// This test always passes — it is purely informational.
#[test]
fn test_wikipedia_compression_ratios() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let fixture_dir = Path::new("tests/fixtures/wikipedia");
    if !fixture_dir.exists() {
        eprintln!("Skipping: fixture directory not found");
        return;
    }

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║        Wikipedia Markdown Compression Benchmark                 ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");

    let mut total_original = 0usize;
    let mut total_zlib_semantic = 0usize;

    let mut entries: Vec<_> = fs::read_dir(fixture_dir)
        .expect("read fixture dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let label = path.file_name().unwrap().to_string_lossy().into_owned();
        let content_bytes = fs::read(&path).expect("read fixture");
        let content_str = String::from_utf8_lossy(&content_bytes).into_owned();

        println!("\n📄 {}", label);

        let uni_res = compress_uni(&content_bytes);
        uni_res.print("uni-encode");

        let sem_res = compress_semantic(&content_str, None);
        sem_res.print("semantic  ");

        let zlib_res = compress_semantic(&content_str, Some("-zlib"));
        zlib_res.print("zlib      ");

        let zlib_sem_res = compress_semantic(&content_str, Some("-zlib -semantic"));
        zlib_sem_res.print("zlib+sem  ");

        total_original += content_bytes.len();
        total_zlib_semantic += zlib_sem_res.compressed_bytes;
    }

    println!("\n══════════════════════════════════════════════════════════════════");
    let overall_ratio = total_zlib_semantic as f64 / total_original as f64 * 100.0;
    println!(
        "  TOTAL corpus: {} bytes → {} bytes  ({:.1}% via zlib+semantic)",
        total_original, total_zlib_semantic, overall_ratio
    );
    println!("══════════════════════════════════════════════════════════════════\n");
}

/// Assert that the large corpus achieves at least 40% size reduction using
/// semantic + zlib compression.  Tightens the threshold if improvements land.
#[test]
fn test_large_corpus_meets_compression_threshold() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let path = Path::new("tests/fixtures/wikipedia/large_wiki_corpus.md");
    if !path.exists() {
        eprintln!("Skipping: large_wiki_corpus.md not found");
        return;
    }

    let content = fs::read_to_string(path).expect("read large corpus");
    let original_bytes = content.len();

    let mq = Marqant::compress_markdown_with_flags(&content, Some("-zlib -semantic"))
        .expect("compress failed");
    let compressed_bytes = mq.len();

    let ratio = compressed_bytes as f64 / original_bytes as f64;
    println!(
        "Large corpus: {} bytes → {} bytes  ({:.1}% of original)",
        original_bytes,
        compressed_bytes,
        ratio * 100.0
    );

    // Must achieve at least 40% reduction (compressed ≤ 60% of original)
    assert!(
        ratio <= 0.60,
        "Expected compression to at least 60% of original size on the large corpus, \
         but got {:.1}% — check that -zlib -semantic flags are working",
        ratio * 100.0
    );

    // Verify decompression roundtrip
    let back = Marqant::decompress_marqant(&mq).expect("decompress failed");
    assert_eq!(
        normalize(&content),
        normalize(&back),
        "Large corpus roundtrip mismatch"
    );
}

/// Test inline programmatic Wikipedia-style content to verify handling of
/// all header levels, numbered lists, tables, blockquotes, and code blocks.
#[test]
fn test_wikipedia_patterns_inline() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let wiki_md = r#"# Main Title

## Overview

Wikipedia articles use a consistent structure with multiple heading levels.

## History

### Early Period

The early period saw rapid development of the field.

### Modern Era

#### The 2000s

The 2000s brought significant changes:

1. First development in 2001
2. Second milestone in 2003
3. Third breakthrough in 2005
4. Fourth innovation in 2007

##### Sub-topic

Detailed analysis of the sub-topic follows.

## Data Table

| Year | Metric | Value |
|------|--------|-------|
| 2020 | Count  | 1,234 |
| 2021 | Count  | 2,345 |
| 2022 | Count  | 3,456 |

## Notes

> Note 1: Important caveat about the data.
>
> Note 2: Another important observation.

## Code Example

```python
def compress(data: str) -> bytes:
    return zlib.compress(data.encode())
```

## See Also

- Related article one
- Related article two
- Related article three

## References

1. Author A (2020). First reference.
2. Author B (2021). Second reference.
3. Author C (2022). Third reference.
"#;

    // Uni roundtrip
    assert_uni_roundtrip(wiki_md.as_bytes(), "wiki-patterns-inline");

    // Semantic roundtrip (all modes)
    assert_semantic_roundtrip(wiki_md, None, "wiki-patterns-inline [default]");
    assert_semantic_roundtrip(wiki_md, Some("-zlib"), "wiki-patterns-inline [zlib]");
    assert_semantic_roundtrip(
        wiki_md,
        Some("-zlib -semantic"),
        "wiki-patterns-inline [zlib+sem]",
    );

    // Verify that H3/H4/H5 patterns are tokenised (smaller uni-encoded output than naïve)
    let encoded = mq2_uni_encode(wiki_md.as_bytes()).expect("encode");
    // ~H3 (3 bytes) replaces "### " (4 bytes) → should appear in output
    assert!(
        encoded.windows(3).any(|w| w == b"~H3"),
        "Expected ~H3 token in encoded output for ### headers"
    );
    assert!(
        encoded.windows(3).any(|w| w == b"~H4"),
        "Expected ~H4 token in encoded output for #### headers"
    );
    assert!(
        encoded.windows(3).any(|w| w == b"~H5"),
        "Expected ~H5 token in encoded output for ##### headers"
    );
    // Numbered list tokens (first item follows \n\n which becomes ~PP, so ~N1
    // won't appear; subsequent items ~N2 onward do)
    assert!(
        encoded.windows(3).any(|w| w == b"~N2"),
        "Expected ~N2 token for ordered list item 2"
    );

    println!("✅ Wikipedia pattern inline test passed");
    println!(
        "  Original:  {} bytes | Uni-encoded: {} bytes | Ratio: {:.1}%",
        wiki_md.len(),
        encoded.len(),
        encoded.len() as f64 / wiki_md.len() as f64 * 100.0,
    );
}

/// Stress test: compress 200 copies of a typical Wikipedia paragraph
/// to simulate bulk ingestion of repetitive corpus content.
#[test]
fn test_bulk_wikipedia_paragraph_compression() {
    std::env::set_var("MARQANT_TEST_TS", "0");

    let paragraph = "## Section\n\nThe quick brown fox jumped over the lazy dog. \
        This sentence is repeated many times to simulate a real-world Wikipedia corpus. \
        The compression algorithm should tokenise repeated words and phrases effectively.\n\n";

    let n = 200usize;
    let large_doc = paragraph.repeat(n);

    let start = Instant::now();
    let mq = Marqant::compress_markdown_with_flags(&large_doc, Some("-zlib"))
        .expect("compress failed");
    let elapsed = start.elapsed().as_millis();

    let back = Marqant::decompress_marqant(&mq).expect("decompress failed");
    assert_eq!(
        normalize(&large_doc),
        normalize(&back),
        "Bulk paragraph roundtrip mismatch"
    );

    let ratio = mq.len() as f64 / large_doc.len() as f64 * 100.0;
    println!(
        "Bulk ({n}x paragraph): {} bytes → {} bytes  ({ratio:.1}%, {elapsed}ms)",
        large_doc.len(),
        mq.len(),
    );

    // Expect very high compression on repetitive content
    assert!(
        ratio < 5.0,
        "Expected >95% compression on highly repetitive content, got {ratio:.1}%"
    );
}
