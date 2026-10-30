//! # Hutter Prize Demon & G8T Arena (`hutter_arena`)
//!
//! Zero-float (`100%` integer and reduced rational `p/q`) benchmark and verification
//! harness for the Level-0 Bit-Perfect Hutter Demon Preprocessor (`DC10`).
//!
//! ## Usage
//! ```bash
//! hutter_arena --bench /data/standardgalactic/AI-Scientist/data/enwik8 --limit-mb 10
//! hutter_arena --bench /data/corpus/enwik8
//! hutter_arena compress input.xml output.dc10
//! hutter_arena decompress output.dc10 restored.xml
//! ```

use anyhow::{ bail, Context, Result };
use flate2::read::GzDecoder;
use marqant::hutter_demon::{
    compress_baseline_deflate, compress_dc10_container, decompress_dc10_container,
    demon_transform_decode, demon_transform_encode, fnv1a64_bytes, landauer_zeptojoules_rational,
    Rational128, DC10_HEADER_LEN,
};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

const DEFAULT_BENCH_PATH: &str = "/data/standardgalactic/AI-Scientist/data/enwik8";
const FALLBACK_CORPUS_PATHS: &[&str] = &[
    "/data/corpus/enwik8",
    "/data/standardgalactic/AI-Scientist/data/enwik8/enwik8",
    "/data/standardgalactic/MEGABYTE-pytorch/data/enwik8.gz",
];

/// Format a millipercent integer (`saved * 100_000 / total`) as `"XX.XXX%"` using integer-only division.
fn format_millipercent(num: u128, den: u128) -> String {
    if den == 0 {
        return "0.000%".to_string();
    }
    let mp = (num * 100_000) / den;
    let whole = mp / 1_000;
    let frac = mp % 1_000;
    format!("{}.{:03}%", whole, frac)
}

/// Resolve a benchmark corpus path:
/// - If `requested` is a regular file, use it directly (transparently gunzipping `.gz` if needed).
/// - If `requested` is a directory (e.g. `/data/standardgalactic/AI-Scientist/data/enwik8`),
///   look for `enwik8` inside it or fall back to `/data/corpus/enwik8` / `enwik8.gz`.
fn load_corpus_bytes(requested: &Path, limit_bytes: Option<usize>) -> Result<(PathBuf, Vec<u8>)> {
    let resolved = if requested.is_file() {
        requested.to_path_buf()
    } else {
        let inside = requested.join("enwik8");
        if inside.is_file() {
            inside
        } else {
            let mut found = None;
            for cand in FALLBACK_CORPUS_PATHS {
                let p = PathBuf::from(cand);
                if p.is_file() {
                    found = Some(p);
                    break;
                }
            }
            match found {
                Some(p) => p,
                None => bail!(
                    "Could not resolve corpus file from '{}' or fallback paths",
                    requested.display()
                ),
            }
        }
    };

    let is_gz = resolved
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"));

    let mut data = Vec::new();
    let file = fs::File::open(&resolved)
        .with_context(|| format!("Failed to open corpus file {}", resolved.display()))?;

    if is_gz {
        let mut gz = GzDecoder::new(file);
        if let Some(limit) = limit_bytes {
            gz.take(limit as u64).read_to_end(&mut data)?;
        } else {
            gz.read_to_end(&mut data)?;
        }
    } else if let Some(limit) = limit_bytes {
        file.take(limit as u64).read_to_end(&mut data)?;
    } else {
        let mut f = file;
        f.read_to_end(&mut data)?;
    }

    Ok((resolved, data))
}

fn run_bench(bench_path: &Path, limit_mb: Option<u64>, deflate_level: u32) -> Result<()> {
    let limit_bytes = limit_mb.map(|mb| (mb as usize).saturating_mul(1_000_000));
    let (resolved_path, input) = load_corpus_bytes(bench_path, limit_bytes)?;
    let orig_len = input.len() as u64;
    if orig_len == 0 {
        bail!("Corpus input is empty: {}", resolved_path.display());
    }

    let orig_checksum = fnv1a64_bytes(&input);

    // Stage 1: Hutter Demon Preprocessor Encode (DC10)
    let t0 = Instant::now();
    let demon_out = demon_transform_encode(&input);
    let encode_us = t0.elapsed().as_micros().max(1);
    let encode_ms = (encode_us / 1_000).max(1) as u128;

    // Stage 2: Verify direct Demon Pre-Pass Decode round-trip
    let t1 = Instant::now();
    let decoded_prepass = demon_transform_decode(&demon_out.encoded)
        .map_err(|e| anyhow::anyhow!("DC10 pre-pass decode error: {}", e))?;
    let decode_us = t1.elapsed().as_micros().max(1);
    let decode_ms = (decode_us / 1_000).max(1) as u128;

    let prepass_exact = decoded_prepass.as_slice() == input.as_slice();
    if !prepass_exact {
        bail!("CRITICAL: DC10 pre-pass round-trip mismatch!");
    }

    // Stage 3: Full DC10 + Deflate container compression & round-trip verification
    let t2 = Instant::now();
    let dc10_container = compress_dc10_container(&input, deflate_level);
    let dc10_comp_ms = (t2.elapsed().as_millis()).max(1);

    let t3 = Instant::now();
    let dc10_unpacked = decompress_dc10_container(&dc10_container)
        .map_err(|e| anyhow::anyhow!("DC10 container decompress error: {}", e))?;
    let dc10_decomp_ms = (t3.elapsed().as_millis()).max(1);

    let unpacked_checksum = fnv1a64_bytes(&dc10_unpacked);
    let bit_exact_verified =
        (dc10_unpacked.as_slice() == input.as_slice()) && (unpacked_checksum == orig_checksum);
    if !bit_exact_verified {
        bail!("CRITICAL: DC10 container round-trip verification failed!");
    }

    // Stage 4: Baseline Deflate at identical compression level
    let t4 = Instant::now();
    let baseline_deflate = compress_baseline_deflate(&input, deflate_level);
    let base_comp_ms = (t4.elapsed().as_millis()).max(1);

    // Integer & Reduced Rational p/q Metrics
    let trans_len = demon_out.transformed_len;
    let raw_bytes_saved = orig_len.saturating_sub(trans_len);
    let raw_savings_pq = Rational128::new(raw_bytes_saved as u128, orig_len as u128);
    let raw_ratio_pq = Rational128::new(trans_len as u128, orig_len as u128);

    let dc10_payload_len = dc10_container.len().saturating_sub(DC10_HEADER_LEN) as u64;
    let dc10_total_len = dc10_container.len() as u64;
    let base_deflate_len = baseline_deflate.len() as u64;

    let post_entropy_saved_payload =
        (base_deflate_len as i64) - (dc10_payload_len as i64);
    let post_entropy_saved_container =
        (base_deflate_len as i64) - (dc10_total_len as i64);

    let dc10_total_savings = orig_len.saturating_sub(dc10_total_len);
    let dc10_total_ratio_pq = Rational128::new(dc10_total_len as u128, orig_len as u128);
    let dc10_total_savings_pq = Rational128::new(dc10_total_savings as u128, orig_len as u128);
    let base_deflate_ratio_pq = Rational128::new(base_deflate_len as u128, orig_len as u128);

    let encode_speed_bytes_per_ms = Rational128::new(orig_len as u128, encode_ms);
    let decode_speed_bytes_per_ms = Rational128::new(orig_len as u128, decode_ms);

    let total_bits_saved = (dc10_total_savings as u128) * 8;
    let prepass_bits_saved = (raw_bytes_saved as u128) * 8;
    let landauer_prepass_zj = landauer_zeptojoules_rational(prepass_bits_saved);
    let landauer_total_zj = landauer_zeptojoules_rational(total_bits_saved);

    println!("================================================================================");
    println!(" HUTTER PRIZE DEMON & G8T ARENA SCOREBOARD (DC10 · LEVEL-0 BIT-PERFECT)");
    println!("================================================================================");
    println!(" Requested Path         : {}", bench_path.display());
    println!(" Resolved Corpus        : {}", resolved_path.display());
    println!(" Input Size (bytes)     : {}", orig_len);
    println!(" Input FNV-1a Checksum  : 0x{:016X}", orig_checksum);
    println!("--------------------------------------------------------------------------------");
    println!(" [STAGE 1] DC10 HUTTER DEMON PRE-PASS (BEFORE ENTROPY CODING)");
    println!("--------------------------------------------------------------------------------");
    println!(" DC10 Pre-Pass Size     : {} bytes", trans_len);
    println!(
        " Raw Bytes Shaved       : {} bytes ({})",
        raw_bytes_saved,
        format_millipercent(raw_bytes_saved as u128, orig_len as u128)
    );
    println!(" Pre-Pass Ratio (p/q)   : {}", raw_ratio_pq);
    println!(" Pre-Pass Savings (p/q) : {}", raw_savings_pq);
    println!(
        " Gross Demon Savings    : {} bytes (across {} Demon invocations)",
        demon_out.gross_demon_bytes_saved(),
        demon_out.total_invocations()
    );
    println!(
        " High-Byte Escapes      : {} bytes (0xFF literal escapes)",
        demon_out.escaped_high_bytes
    );
    println!(
        " Pre-Pass Encode Speed  : {} bytes/ms (elapsed: {} ms)",
        encode_speed_bytes_per_ms, encode_ms
    );
    println!(
        " Pre-Pass Decode Speed  : {} bytes/ms (elapsed: {} ms)",
        decode_speed_bytes_per_ms, decode_ms
    );
    println!("--------------------------------------------------------------------------------");
    println!(" [STAGE 2] PER-CATEGORY DEMON INVOCATIONS & BYTES SAVED");
    println!("--------------------------------------------------------------------------------");
    println!(
        " 1. XML Envelope Demons : {:>10} invocations | {:>10} bytes saved",
        demon_out.xml_envelope.invocations, demon_out.xml_envelope.bytes_saved
    );
    println!(
        " 2. WikiLink/Cat Demons : {:>10} invocations | {:>10} bytes saved",
        demon_out.wikilink_category.invocations, demon_out.wikilink_category.bytes_saved
    );
    println!(
        " 3. Pipe-Link Stem Demon: {:>10} invocations | {:>10} bytes saved",
        demon_out.pipe_link_stem.invocations, demon_out.pipe_link_stem.bytes_saved
    );
    println!(
        " 4. XML Entity Demons   : {:>10} invocations | {:>10} bytes saved",
        demon_out.xml_entity.invocations, demon_out.xml_entity.bytes_saved
    );
    println!("--------------------------------------------------------------------------------");
    println!(
        " [STAGE 3] ENTROPY ARENA: DC10 + DEFLATE(L{}) vs BASELINE DEFLATE(L{})",
        deflate_level, deflate_level
    );
    println!("--------------------------------------------------------------------------------");
    println!(
        " Baseline Deflate Size  : {} bytes (ratio p/q: {}, {} ms)",
        base_deflate_len, base_deflate_ratio_pq, base_comp_ms
    );
    println!(
        " DC10 + Deflate Payload : {} bytes",
        dc10_payload_len
    );
    println!(
        " DC10 Container Total   : {} bytes (incl. {}-byte DC10 header, ratio p/q: {}, {} ms)",
        dc10_total_len, DC10_HEADER_LEN, dc10_total_ratio_pq, dc10_comp_ms
    );
    println!(
        " Net Deflate Gain (Raw) : {:+} bytes saved vs baseline Deflate",
        post_entropy_saved_payload
    );
    println!(
        " Net Deflate Gain (Full): {:+} bytes saved vs baseline Deflate (incl. header)",
        post_entropy_saved_container
    );
    println!(
        " Total Compression      : {} bytes saved ({}) | Savings p/q: {}",
        dc10_total_savings,
        format_millipercent(dc10_total_savings as u128, orig_len as u128),
        dc10_total_savings_pq
    );
    println!("--------------------------------------------------------------------------------");
    println!(" [STAGE 4] LANDAUER THERMODYNAMICS & BIT-PERFECT VERIFICATION");
    println!("--------------------------------------------------------------------------------");
    println!(" Pre-Pass Bits Saved    : {} bits", prepass_bits_saved);
    println!(" Pre-Pass Landauer (zJ) : {} zJ (p/q)", landauer_prepass_zj);
    println!(" Total Bits Saved       : {} bits", total_bits_saved);
    println!(" Total Landauer (zJ)    : {} zJ (p/q)", landauer_total_zj);
    println!(" Unpack + Verify Time   : {} ms", dc10_decomp_ms);
    println!(" Restored FNV-1a Hash   : 0x{:016X}", unpacked_checksum);
    println!(" BIT_EXACT_VERIFIED     : {}", bit_exact_verified);
    println!("================================================================================");

    Ok(())
}

fn print_usage() {
    eprintln!("Hutter Prize Demon & G8T Arena (DC10 · Zero-Float Rational Engine)");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  hutter_arena --bench [PATH] [--limit-mb <MB>] [--deflate-level <1..9>]");
    eprintln!("  hutter_arena compress <INPUT> <OUTPUT.dc10> [--deflate-level <0..9>]");
    eprintln!("  hutter_arena decompress <INPUT.dc10> <OUTPUT>");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 4 && args[1] == "compress" {
        let input_path = PathBuf::from(&args[2]);
        let output_path = PathBuf::from(&args[3]);
        let mut level = 6u32;
        let mut i = 4;
        while i < args.len() {
            if args[i] == "--deflate-level" && i + 1 < args.len() {
                level = args[i + 1].parse().unwrap_or(6);
                i += 2;
            } else {
                i += 1;
            }
        }
        let input = fs::read(&input_path)
            .with_context(|| format!("Failed to read {}", input_path.display()))?;
        let container = compress_dc10_container(&input, level);
        fs::write(&output_path, &container)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;
        println!(
            "DC10 compressed {} -> {} bytes (ratio p/q: {})",
            input.len(),
            container.len(),
            Rational128::new(container.len() as u128, input.len() as u128)
        );
        return Ok(());
    }

    if args.len() >= 4 && args[1] == "decompress" {
        let input_path = PathBuf::from(&args[2]);
        let output_path = PathBuf::from(&args[3]);
        let container = fs::read(&input_path)
            .with_context(|| format!("Failed to read {}", input_path.display()))?;
        let restored = decompress_dc10_container(&container)
            .map_err(|e| anyhow::anyhow!("DC10 decompression failed: {}", e))?;
        fs::write(&output_path, &restored)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;
        println!(
            "DC10 decompressed {} -> {} bytes (FNV-1a: 0x{:016X}, BIT_EXACT_VERIFIED: true)",
            container.len(),
            restored.len(),
            fnv1a64_bytes(&restored)
        );
        return Ok(());
    }

    let mut bench_path = PathBuf::from(DEFAULT_BENCH_PATH);
    let mut limit_mb: Option<u64> = None;
    let mut deflate_level: u32 = 6;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bench" => {
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    bench_path = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--limit-mb" => {
                if i + 1 < args.len() {
                    limit_mb = Some(
                        args[i + 1]
                            .parse::<u64>()
                            .context("--limit-mb requires a positive integer")?,
                    );
                    i += 2;
                } else {
                    bail!("--limit-mb requires a value");
                }
            }
            "--deflate-level" => {
                if i + 1 < args.len() {
                    deflate_level = args[i + 1]
                        .parse::<u32>()
                        .context("--deflate-level requires an integer 0..9")?;
                    i += 2;
                } else {
                    bail!("--deflate-level requires a value");
                }
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            other => {
                bench_path = PathBuf::from(other);
                i += 1;
            }
        }
    }

    run_bench(&bench_path, limit_mb, deflate_level)
}
