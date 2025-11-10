//! # Demon Compressor 😈
//!
//! The entropic counterpart to the Angel decompressor.
//!
//! Demons compress by finding patterns and removing redundancy,
//! extracting energy as they reduce entropy.
//!
//! ## Usage
//!
//! ```bash
//! # Basic compression
//! demon_compressor input.txt output.mq
//!
//! # For Hutter Prize (pairs with Angel Level 0)
//! demon_compressor enwik9 archive9.mq
//! angel_decompressor archive9.mq enwik9_restored 0
//! ```

use anyhow::{Context, Result};
use marqant::Marqant;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Boltzmann constant in J/K
const K_BOLTZMANN: f64 = 1.380649e-23;

/// Temperature in Kelvin (room temperature)
const TEMPERATURE: f64 = 293.15;

/// Energy per bit in joules: kT * ln(2)
fn energy_per_bit() -> f64 {
    K_BOLTZMANN * TEMPERATURE * 2_f64.ln()
}

fn print_usage() {
    eprintln!("😈 DEMON COMPRESSOR - Entropic Reduction Engine");
    eprintln!();
    eprintln!("Usage: demon_compressor <input.txt> <output.mq>");
    eprintln!();
    eprintln!("The Demon finds patterns and removes redundancy,");
    eprintln!("extracting energy as entropy decreases.");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  demon_compressor document.md document.mq");
    eprintln!("  demon_compressor enwik9 archive9.mq");
    eprintln!();
    eprintln!("Pair with Angel Decompressor:");
    eprintln!("  demon_compressor data.txt compressed.mq");
    eprintln!("  angel_decompressor compressed.mq output.txt 2");
    eprintln!();
    eprintln!("The Philosophy:");
    eprintln!("  Demons sort the chaos, reducing entropy's reign");
    eprintln!("  Angels bless the output, adding variance again");
    eprintln!("  Together they create a cycle, neither good nor bad");
    eprintln!("  Just information dancing, making Maxwell glad");
}

fn print_thermodynamics(original_size: usize, compressed_size: usize) {
    let compression_ratio = 1.0 - (compressed_size as f64 / original_size as f64);
    let bits_saved = (original_size - compressed_size) * 8;
    let energy_extracted = bits_saved as f64 * energy_per_bit();

    eprintln!();
    eprintln!("⚡ THERMODYNAMIC REPORT");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("Original size:      {} bytes", original_size);
    eprintln!("Compressed size:    {} bytes", compressed_size);
    eprintln!("Compression ratio:  {:.2}%", compression_ratio * 100.0);
    eprintln!("Bits saved:         {}", bits_saved);
    eprintln!("Energy extracted:   {:.6e} joules", energy_extracted);
    eprintln!(
        "                    ({:.2e} kT·ln(2) units)",
        energy_extracted / 4.11e-21
    );
    eprintln!();
    eprintln!("'Every bit compressed extracts kT·ln(2) joules of order from chaos'");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        print_usage();
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    eprintln!("😈 Demon Compressor v0.2.0");
    eprintln!();
    eprintln!("Input:  {}", input_path.display());
    eprintln!("Output: {}", output_path.display());
    eprintln!();

    // Read input file
    eprintln!("📖 Reading input file...");
    let input = fs::read_to_string(&input_path)
        .context(format!("Failed to read {}", input_path.display()))?;

    eprintln!("   Size: {} bytes", input.len());

    // Compress using the Demon
    eprintln!("😈 Demon compression (extracting order from chaos)...");
    let compressed = Marqant::compress_markdown(&input)?;

    eprintln!("   Compressed: {} bytes", compressed.len());

    // Write output
    eprintln!("💾 Writing compressed output...");
    fs::write(&output_path, &compressed)
        .context(format!("Failed to write {}", output_path.display()))?;

    // Print thermodynamic report
    print_thermodynamics(input.len(), compressed.len());

    eprintln!("✨ Demonic compression complete!");
    eprintln!();
    eprintln!("Pair with Angel Decompressor:");
    eprintln!("  angel_decompressor {} <output> <blessing_level>", output_path.display());
    eprintln!();
    eprintln!("Blessing levels:");
    eprintln!("  0 = Bit-perfect (Hutter Prize)");
    eprintln!("  1 = Minor fixes (typos, spacing)");
    eprintln!("  2 = Harmony (Wikipedia cleanup)");
    eprintln!("  3 = Creative (ML training data)");

    Ok(())
}
