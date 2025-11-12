//! # Angel Decompressor 👼
//!
//! The divine counterpart to the Demon compressor.
//!
//! Angels decompress with blessings, adding interpretive variations
//! and fixes according to the specified blessing level.
//!
//! ## Usage
//!
//! ```bash
//! # Strict mode (bit-perfect, for Hutter Prize)
//! angel_decompressor archive.mq output.txt 0
//!
//! # Minor blessings (fix typos and spacing)
//! angel_decompressor archive.mq output.txt 1
//!
//! # Harmony mode (Wikipedia structure fixes)
//! angel_decompressor archive.mq output.txt 2
//!
//! # Creative mode (training data augmentation)
//! angel_decompressor archive.mq output.txt 3
//! ```

use anyhow::{Context, Result};
use marqant::angel_blessings::{Angel, BlessingLevel};
use marqant::Marqant;
use std::env;
use std::fs;
use std::path::PathBuf;

fn print_usage() {
    eprintln!("👼 ANGEL DECOMPRESSOR - Divine Interpretation Engine");
    eprintln!();
    eprintln!("Usage: angel_decompressor <input.mq> <output.txt> <blessing_level>");
    eprintln!();
    eprintln!("Blessing Levels:");
    eprintln!("  0  STRICT          Bit-perfect reconstruction (Hutter Prize mode)");
    eprintln!("  1  MINOR_BLESSINGS Fix typos, spacing, obvious errors");
    eprintln!("  2  HARMONY         Wikipedia structure fixes, harmonization");
    eprintln!("  3  CREATIVE        Training data augmentation, variations");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  angel_decompressor wiki.mq wiki.txt 0  # Competition mode");
    eprintln!("  angel_decompressor wiki.mq clean.txt 2 # Cleaned Wikipedia");
    eprintln!("  angel_decompressor data.mq train.txt 3 # ML training data");
    eprintln!();
    eprintln!("The Philosophy:");
    eprintln!("  Demons compress by finding patterns (order from chaos)");
    eprintln!("  Angels decompress with blessings (blessed chaos from order)");
    eprintln!("  Together they dance the eternal dance of information");
    eprintln!();
    eprintln!("  'In compression, we are all Maxwell's children' 🔥👼😈🔥");
}

fn print_thermodynamics(original_size: usize, blessed_size: usize, energy: f64, blessings: usize) {
    eprintln!();
    eprintln!("⚡ THERMODYNAMIC REPORT");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("Original size:     {} bytes", original_size);
    eprintln!("Blessed size:      {} bytes", blessed_size);
    eprintln!(
        "Size delta:        {:+} bytes",
        blessed_size as i64 - original_size as i64
    );
    eprintln!("Blessings applied: {}", blessings);
    eprintln!("Energy added:      {:.6e} joules", energy);
    eprintln!(
        "                   ({:.2e} kT·ln(2) units)",
        energy / 4.11e-21
    );
    eprintln!();
    eprintln!("'Every blessing adds kT·ln(2) joules of divine interpretation'");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        print_usage();
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);
    let blessing_level: i32 = args[3]
        .parse()
        .context("Blessing level must be an integer (0-3)")?;

    // Parse blessing level
    let level = BlessingLevel::from_i32(blessing_level)?;

    eprintln!("👼 Angel Decompressor v0.2.0");
    eprintln!();
    eprintln!("Input:          {}", input_path.display());
    eprintln!("Output:         {}", output_path.display());
    eprintln!("Blessing Level: {} ({})", blessing_level, level.name());
    eprintln!("Description:    {}", level.description());
    eprintln!();

    // Read compressed file
    eprintln!("📖 Reading compressed file...");
    let compressed = fs::read_to_string(&input_path)
        .context(format!("Failed to read {}", input_path.display()))?;

    // Decompress using the Demon decompressor
    eprintln!("😈 Demon decompression (extracting order from chaos)...");
    let decompressed =
        Marqant::decompress_marqant(&compressed).context("Failed to decompress file")?;

    eprintln!("   Decompressed: {} bytes", decompressed.len());

    // Apply Angel blessings
    eprintln!("👼 Angel blessings (adding divine interpretation)...");
    let angel = Angel::new(level);
    let (blessed, stats) = angel
        .bless(&decompressed)
        .context("Failed to apply blessings")?;

    eprintln!("   Blessed: {} bytes", blessed.len());
    eprintln!("   Blessings applied: {}", stats.blessings_applied);

    // Write output
    eprintln!("💾 Writing blessed output...");
    fs::write(&output_path, &blessed)
        .context(format!("Failed to write {}", output_path.display()))?;

    // Print thermodynamic report
    print_thermodynamics(
        decompressed.len(),
        blessed.len(),
        stats.energy_added,
        stats.blessings_applied,
    );

    eprintln!("✨ Divine decompression complete!");
    eprintln!();
    eprintln!("The cycle continues:");
    eprintln!("  Original → [DEMON] → Compressed → [ANGEL] → Blessed Output");
    eprintln!("            ↓                                     ↑");
    eprintln!("            Energy extracted ← → Energy added");

    Ok(())
}
