//! Development driver for the integer-only template/field/exception experiment.
use anyhow::{bail, Context, Result};
use marqant::hutter_demon::{compress_baseline_deflate, compress_dc10_container};
use marqant::{mem8_lattice, salient_templates};
use salient_templates::{compress, decompress, Options};
use std::{env, fs, io::Read, time::Instant};

fn usage() -> &'static str {
    "Usage:\n  salient_template_arena bench INPUT [OPTIONS]\n  salient_template_arena compress INPUT OUTPUT [OPTIONS]\n  salient_template_arena decompress INPUT OUTPUT --max-output N\nOptions: --templates 0..256 --record-bytes 16..4096 --min-gain N --no-context --lattice-literals --limit-bytes N --offset N\nSizes include every dictionary, field, exception and header byte. Times are integer nanoseconds."
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
    let output = command != "bench";
    if output && args.len() < 4 {
        bail!("{}", usage());
    }
    let mut options = Options {
        max_templates: 64,
        max_record: 512,
        min_gain: 64,
        context: true,
        lattice_literals: false,
    };
    let mut limit = None;
    let mut offset = 0u64;
    let mut max_output = None;
    let mut i = if output { 4 } else { 3 };
    while i < args.len() {
        match args[i].as_str() {
            "--no-context" => {
                options.context = false;
                i += 1;
                continue;
            }
            "--lattice-literals" => {
                options.lattice_literals = true;
                i += 1;
                continue;
            }
            _ => {}
        }
        let value = args.get(i + 1).context("Missing option value")?;
        match args[i].as_str() {
            "--templates" => options.max_templates = value.parse()?,
            "--record-bytes" => options.max_record = value.parse()?,
            "--min-gain" => options.min_gain = value.parse()?,
            "--limit-bytes" => limit = Some(value.parse::<u64>()?),
            "--offset" => offset = value.parse::<u64>()?,
            "--max-output" => max_output = Some(value.parse::<usize>()?),
            unknown => bail!("Unknown option {}", unknown),
        }
        i += 2;
    }
    if command == "decompress" {
        let bound = max_output.context("decompress requires --max-output N")?;
        let archive = fs::read(&args[2])?;
        let t = Instant::now();
        let restored = decompress(&archive, bound).map_err(anyhow::Error::msg)?;
        let elapsed = t.elapsed().as_nanos();
        fs::write(&args[3], &restored)?;
        println!("restored_bytes={} decode_ns={}", restored.len(), elapsed);
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
    let t = Instant::now();
    let encoded = compress(&input, options).map_err(anyhow::Error::msg)?;
    let encode_ns = t.elapsed().as_nanos();
    if command == "compress" {
        fs::write(&args[3], &encoded.bytes)?;
        println!("input_bytes={} archive_bytes={} encode_ns={} templates={} template_records={} predicted_records={}",
            input.len(), encoded.bytes.len(), encode_ns, encoded.templates, encoded.template_records, encoded.predicted_records);
        return Ok(());
    }
    let t = Instant::now();
    let restored = decompress(&encoded.bytes, input.len()).map_err(anyhow::Error::msg)?;
    let decode_ns = t.elapsed().as_nanos();
    if restored != input {
        bail!("Template codec BIT-EXACT ROUND TRIP FAILED");
    }
    let zlib = compress_baseline_deflate(&input, 9);
    let dc10 = compress_dc10_container(&input, 9);
    let lattice = mem8_lattice::compress(
        &input,
        mem8_lattice::Options {
            context_bytes: 4,
            min_match: 16,
            dc10: true,
        },
    )
    .map_err(anyhow::Error::msg)?;
    println!("input_bytes,offset,max_templates,max_record,min_gain,context,lattice_literals,archive_bytes,zlib9_bytes,dc10_bytes,lattice_bytes,gain_vs_dc10,gain_vs_lattice,templates,template_records,predicted_records,literal_bytes,field_bytes,dictionary_bytes,estimated_gain_bits,encode_ns,decode_ns,bit_exact");
    println!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},true",
        input.len(),
        offset,
        options.max_templates,
        options.max_record,
        options.min_gain,
        u8::from(options.context),
        u8::from(options.lattice_literals),
        encoded.bytes.len(),
        zlib.len(),
        dc10.len(),
        lattice.bytes.len(),
        dc10.len() as i128 - encoded.bytes.len() as i128,
        lattice.bytes.len() as i128 - encoded.bytes.len() as i128,
        encoded.templates,
        encoded.template_records,
        encoded.predicted_records,
        encoded.literal_bytes,
        encoded.field_bytes,
        encoded.dictionary_bytes,
        encoded.estimated_gain_bits,
        encode_ns,
        decode_ns
    );
    Ok(())
}
