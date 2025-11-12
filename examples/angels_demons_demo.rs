//! Angels & Demons API Demo
//!
//! Demonstrates how to use the Angel blessing system programmatically.
//!
//! Run with:
//! ```bash
//! cargo run --example angels_demons_demo
//! ```

use marqant::angel_blessings::{Angel, BlessingLevel};
use marqant::Marqant;

fn main() -> anyhow::Result<()> {
    println!("🔥👼😈🔥 ANGELS & DEMONS API DEMO 🔥😈👼🔥");
    println!();
    println!("Thermodynamic Poetry:");
    println!("  Demons sort the chaos, reducing entropy's reign");
    println!("  Angels bless the output, adding variance again");
    println!("  Together they create a cycle, neither good nor bad");
    println!("  Just information dancing, making Maxwell glad");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Sample text with various issues
    let original = r#"# Wikipedia Article

This  article  has  double  spaces and teh recieve are typos.

## Categories

[[category:computers]] and [[CATEGORY:science]] need fixing.

### Technical Content

The system  is  a  distributed  approach . Notice space before period .

- Bullet  item  one
- Bullet  item  two

The wave is a fundamental concept and the wave implements patterns.
"#;

    println!("📝 Original text ({} bytes):", original.len());
    println!("{}", original);
    println!();

    // Step 1: Compress with Demon
    println!("😈 DEMON COMPRESSION (extracting order from chaos)...");
    let compressed = Marqant::compress_markdown(original)?;
    println!("   Compressed: {} bytes", compressed.len());
    let ratio = 1.0 - (compressed.len() as f64 / original.len() as f64);
    println!("   Compression ratio: {:.1}%", ratio * 100.0);
    println!();

    // Step 2: Decompress
    println!("😈 Decompressing...");
    let decompressed = Marqant::decompress_marqant(&compressed)?;
    println!("   Decompressed: {} bytes", decompressed.len());
    println!();

    // Step 3: Apply different blessing levels
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("👼 ANGEL BLESSINGS (adding divine interpretation)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Level 0: Strict
    println!("┌─── LEVEL 0: STRICT ───────────────────┐");
    let angel_0 = Angel::new(BlessingLevel::Strict);
    let (blessed_0, stats_0) = angel_0.bless(&decompressed)?;
    println!("│ Bit-perfect reconstruction           │");
    println!(
        "│ Blessings applied: {}                  │",
        stats_0.blessings_applied
    );
    println!("│ Energy added: {:.2e} J          │", stats_0.energy_added);
    println!("└───────────────────────────────────────┘");
    println!("Output size: {} bytes (unchanged)", blessed_0.len());
    println!();

    // Level 1: Minor Blessings
    println!("┌─── LEVEL 1: MINOR BLESSINGS ──────────┐");
    let angel_1 = Angel::new(BlessingLevel::MinorBlessings);
    let (blessed_1, stats_1) = angel_1.bless(&decompressed)?;
    println!("│ Fix typos, spacing, punctuation      │");
    println!(
        "│ Blessings applied: {}                  │",
        stats_1.blessings_applied
    );
    println!("│ Energy added: {:.2e} J          │", stats_1.energy_added);
    println!("└───────────────────────────────────────┘");
    println!(
        "Output size: {} bytes (Δ: {})",
        blessed_1.len(),
        stats_1.size_delta()
    );

    // Show specific fixes
    if original.contains("teh") && !blessed_1.contains("teh") {
        println!("✓ Fixed typo: 'teh' → 'the'");
    }
    if original.contains("  ") && blessed_1.len() < original.len() {
        println!("✓ Fixed double spaces");
    }
    println!();

    // Level 2: Harmony
    println!("┌─── LEVEL 2: HARMONY ──────────────────┐");
    let angel_2 = Angel::new(BlessingLevel::Harmony);
    let (blessed_2, stats_2) = angel_2.bless(&decompressed)?;
    println!("│ Wikipedia structure harmonization    │");
    println!(
        "│ Blessings applied: {}                  │",
        stats_2.blessings_applied
    );
    println!("│ Energy added: {:.2e} J          │", stats_2.energy_added);
    println!("└───────────────────────────────────────┘");
    println!(
        "Output size: {} bytes (Δ: {})",
        blessed_2.len(),
        stats_2.size_delta()
    );

    // Show category fixes
    if original.contains("[[category:") && blessed_2.contains("[[Category:") {
        println!("✓ Fixed category capitalization: '[[category:' → '[[Category:'");
    }
    println!();

    // Level 3: Creative
    println!("┌─── LEVEL 3: CREATIVE ─────────────────┐");
    let angel_3 = Angel::with_seed(BlessingLevel::Creative, 42);
    let (blessed_3, stats_3) = angel_3.bless(&decompressed)?;
    println!("│ Training data augmentation            │");
    println!(
        "│ Blessings applied: {}                  │",
        stats_3.blessings_applied
    );
    println!("│ Energy added: {:.2e} J          │", stats_3.energy_added);
    println!("└───────────────────────────────────────┘");
    println!(
        "Output size: {} bytes (Δ: {})",
        blessed_3.len(),
        stats_3.size_delta()
    );
    println!("(With seed 42 for reproducibility)");
    println!();

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("THERMODYNAMIC SUMMARY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Total energy added by Angels:");
    let total_energy = stats_1.energy_added + stats_2.energy_added + stats_3.energy_added;
    println!("  {:.2e} joules", total_energy);
    println!();
    println!("Total blessings applied:");
    let total_blessings =
        stats_1.blessings_applied + stats_2.blessings_applied + stats_3.blessings_applied;
    println!("  {} blessings", total_blessings);
    println!();
    println!("Average energy per blessing:");
    if total_blessings > 0 {
        println!("  {:.2e} joules", total_energy / total_blessings as f64);
    }
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("✨ The eternal dance of information continues!");
    println!();
    println!("Use cases:");
    println!("  • Hutter Prize:    Level 0 (bit-perfect)");
    println!("  • Document cleanup: Level 1 (typo fixes)");
    println!("  • Wikipedia:       Level 2 (harmonization)");
    println!("  • ML training:     Level 3 (variations)");
    println!();
    println!("'In compression, we are all Maxwell's children' 🔥");

    Ok(())
}
