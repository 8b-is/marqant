//! Integer-only experimental AYEOS-inspired lattice codec driver.
//! Not a production MEM8 binding or a prize-ready compressor.
use anyhow::{bail, Context, Result};
use marqant::hutter_demon::{compress_baseline_deflate, compress_dc10_container};
use marqant::mem8_lattice::{compress, decompress, Options};
use std::{env, fs, io::Read, time::Instant};

fn usage() -> &'static str {
    "Usage:\n  mem8_lattice_arena bench INPUT [--limit-bytes N] [--offset N] [--context 4|8] [--min-match N] [--dc10]\n  mem8_lattice_arena compress INPUT OUTPUT [--context 4|8] [--min-match N] [--dc10]\n  mem8_lattice_arena decompress INPUT OUTPUT --max-output N\nAll sizes include container headers; timing uses integer nanoseconds."
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] == "--help" {
        println!("{}", usage());
        return Ok(());
    }
    let command = args[1].as_str();
    if !matches!(command, "bench" | "compress" | "decompress") {
        bail!("Unknown command\n{}", usage());
    }
    let needs_output = command != "bench";
    if needs_output && args.len() < 4 {
        bail!("{}", usage());
    }
    let mut options = Options {
        context_bytes: 8,
        min_match: 8,
        dc10: false,
    };
    let mut limit = None;
    let mut max_output = None;
    let mut offset = 0u64;
    let mut i = if needs_output { 4 } else { 3 };
    while i < args.len() {
        let arg = &args[i];
        if arg == "--dc10" {
            options.dc10 = true;
            i += 1;
            continue;
        }
        let value = args.get(i + 1).context("Missing option value")?;
        match arg.as_str() {
            "--context" => options.context_bytes = value.parse()?,
            "--min-match" => options.min_match = value.parse()?,
            "--limit-bytes" => limit = Some(value.parse::<u64>()?),
            "--offset" => offset = value.parse()?,
            "--max-output" => max_output = Some(value.parse::<usize>()?),
            _ => bail!("Unknown option: {}", arg),
        }
        i += 2;
    }
    if command == "decompress" {
        let bound = max_output.context("decompress requires --max-output N")?;
        let archive = fs::read(&args[2])?;
        let start = Instant::now();
        let restored = decompress(&archive, bound).map_err(anyhow::Error::msg)?;
        let elapsed = start.elapsed().as_nanos();
        fs::write(&args[3], &restored)?;
        println!(
            "decompressed_bytes={} decode_ns={}",
            restored.len(),
            elapsed
        );
        return Ok(());
    }
    let mut file = fs::File::open(&args[2])?;
    if offset != 0 {
        use std::io::{Seek, SeekFrom};
        file.seek(SeekFrom::Start(offset))?;
    }
    let mut input = Vec::new();
    file.take(limit.unwrap_or(u64::MAX))
        .read_to_end(&mut input)?;
    let start = Instant::now();
    let encoded = compress(&input, options).map_err(anyhow::Error::msg)?;
    let encode_ns = start.elapsed().as_nanos();
    if command == "compress" {
        fs::write(&args[3], &encoded.bytes)?;
        println!(
            "input_bytes={} archive_bytes={} encode_ns={}",
            input.len(),
            encoded.bytes.len(),
            encode_ns
        );
        return Ok(());
    }
    let start = Instant::now();
    let restored = decompress(&encoded.bytes, input.len()).map_err(anyhow::Error::msg)?;
    let decode_ns = start.elapsed().as_nanos();
    if restored != input {
        bail!("BIT-EXACT ROUND TRIP FAILED");
    }
    let baseline = compress_baseline_deflate(&input, 9);
    let dc10 = compress_dc10_container(&input, 9);
    println!("input_bytes,offset,context_bytes,min_match,dc10,lattice_bytes,zlib9_bytes,dc10_zlib9_bytes,gain_vs_dc10_bytes,matches,matched_bytes,control_bytes,literal_bytes,encode_ns,decode_ns,bit_exact");
    println!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},true",
        input.len(),
        offset,
        options.context_bytes,
        options.min_match,
        u8::from(options.dc10),
        encoded.bytes.len(),
        baseline.len(),
        dc10.len(),
        dc10.len() as i128 - encoded.bytes.len() as i128,
        encoded.matches,
        encoded.matched_bytes,
        encoded.control_bytes,
        encoded.literal_bytes,
        encode_ns,
        decode_ns
    );
    Ok(())
}
