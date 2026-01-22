//! # Angels & Demons: The Duality of Compression
//!
//! This module implements the Angel Decompressor with blessing levels.
//!
//! ## Philosophy
//!
//! - **DEMONS** compress data by finding patterns and removing redundancy
//! - **ANGELS** decompress with divine interpretation, adding blessed variations
//!
//! ## Thermodynamic Poetry
//!
//! ```text
//! Demons sort the chaos, reducing entropy's reign
//! Angels bless the output, adding variance again
//! Together they create a cycle, neither good nor bad
//! Just information dancing, making Maxwell glad
//! ```
//!
//! ## Blessing Levels
//!
//! - **Level 0 (STRICT)**: Bit-perfect reconstruction (for Hutter Prize)
//! - **Level 1 (MINOR_BLESSINGS)**: Fix typos, spacing, obvious errors
//! - **Level 2 (HARMONY)**: Wikipedia structure fixes, template harmonization
//! - **Level 3 (CREATIVE)**: Training data augmentation, semantic variations

use anyhow::Result;
use std::collections::HashMap;

/// Boltzmann constant in J/K
const K_BOLTZMANN: f64 = 1.380649e-23;

/// Temperature in Kelvin (room temperature)
const TEMPERATURE: f64 = 293.15;

/// Energy per bit in joules: kT * ln(2)
fn energy_per_bit() -> f64 {
    K_BOLTZMANN * TEMPERATURE * 2_f64.ln()
}

/// Blessing levels for angel decompression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlessingLevel {
    /// Level 0: Bit-perfect reconstruction (no blessings)
    Strict = 0,

    /// Level 1: Minor fixes (typos, spacing)
    MinorBlessings = 1,

    /// Level 2: Harmony (Wikipedia structure fixes)
    Harmony = 2,

    /// Level 3: Creative (training data augmentation)
    Creative = 3,
}

impl BlessingLevel {
    /// Parse blessing level from integer
    pub fn from_i32(level: i32) -> Result<Self> {
        match level {
            0 => Ok(BlessingLevel::Strict),
            1 => Ok(BlessingLevel::MinorBlessings),
            2 => Ok(BlessingLevel::Harmony),
            3 => Ok(BlessingLevel::Creative),
            _ => Err(anyhow::anyhow!(
                "Invalid blessing level: {}. Must be 0-3.",
                level
            )),
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            BlessingLevel::Strict => "STRICT",
            BlessingLevel::MinorBlessings => "MINOR_BLESSINGS",
            BlessingLevel::Harmony => "HARMONY",
            BlessingLevel::Creative => "CREATIVE",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            BlessingLevel::Strict => "Bit-perfect reconstruction (for Hutter Prize)",
            BlessingLevel::MinorBlessings => "Fix typos, spacing, obvious errors",
            BlessingLevel::Harmony => "Wikipedia structure fixes, template harmonization",
            BlessingLevel::Creative => "Training data augmentation, semantic variations",
        }
    }
}

/// Statistics about the blessing process
#[derive(Debug, Default)]
pub struct BlessingStats {
    /// Original length in bytes
    pub original_length: usize,

    /// Blessed length in bytes
    pub blessed_length: usize,

    /// Number of blessings applied
    pub blessings_applied: usize,

    /// Entropy added (in bits)
    pub entropy_added: f64,

    /// Energy added (in joules)
    pub energy_added: f64,
}

impl BlessingStats {
    /// Calculate thermodynamic properties
    pub fn calculate_thermodynamics(&mut self) {
        // Each blessing adds approximately 1 bit of entropy
        self.entropy_added = self.blessings_applied as f64;

        // Energy = entropy * kT * ln(2)
        self.energy_added = self.entropy_added * energy_per_bit();
    }

    /// Get size change
    pub fn size_delta(&self) -> i64 {
        self.blessed_length as i64 - self.original_length as i64
    }
}

/// The Angel Decompressor
pub struct Angel {
    /// Blessing level to apply
    level: BlessingLevel,

    /// Typo correction dictionary (for Level 1+)
    typo_dict: HashMap<String, String>,

    /// Random seed for creative mode
    seed: u64,
}

impl Angel {
    /// Create a new Angel with specified blessing level
    pub fn new(level: BlessingLevel) -> Self {
        Self {
            level,
            typo_dict: Self::create_typo_dictionary(),
            seed: 0,
        }
    }

    /// Create with custom seed (for reproducible creative mode)
    pub fn with_seed(level: BlessingLevel, seed: u64) -> Self {
        Self {
            level,
            typo_dict: Self::create_typo_dictionary(),
            seed,
        }
    }

    /// Create the typo correction dictionary
    fn create_typo_dictionary() -> HashMap<String, String> {
        let mut dict = HashMap::new();

        // Common typos
        dict.insert("teh".to_string(), "the".to_string());
        dict.insert("recieve".to_string(), "receive".to_string());
        dict.insert("occured".to_string(), "occurred".to_string());
        dict.insert("seperate".to_string(), "separate".to_string());
        dict.insert("definately".to_string(), "definitely".to_string());
        dict.insert("wierd".to_string(), "weird".to_string());
        dict.insert("accomodate".to_string(), "accommodate".to_string());
        dict.insert("beleive".to_string(), "believe".to_string());

        dict
    }

    /// Apply blessings to decompressed text
    pub fn bless(&self, text: &str) -> Result<(String, BlessingStats)> {
        let mut stats = BlessingStats {
            original_length: text.len(),
            ..Default::default()
        };

        let blessed = match self.level {
            BlessingLevel::Strict => {
                // No blessings - return as-is
                text.to_string()
            }
            BlessingLevel::MinorBlessings => self.apply_minor_blessings(text, &mut stats),
            BlessingLevel::Harmony => {
                // Apply minor blessings first, then harmony
                let minor = self.apply_minor_blessings(text, &mut stats);
                self.apply_harmony_blessings(&minor, &mut stats)
            }
            BlessingLevel::Creative => {
                // Apply all previous levels, then creative
                let minor = self.apply_minor_blessings(text, &mut stats);
                let harmony = self.apply_harmony_blessings(&minor, &mut stats);
                self.apply_creative_blessings(&harmony, &mut stats)
            }
        };

        stats.blessed_length = blessed.len();
        stats.calculate_thermodynamics();

        Ok((blessed, stats))
    }

    /// Apply Level 1: Minor blessings (typos, spacing)
    fn apply_minor_blessings(&self, text: &str, stats: &mut BlessingStats) -> String {
        let mut result = text.to_string();

        // Fix double spaces -> single space
        let double_space_count = result.matches("  ").count();
        result = result.replace("  ", " ");
        if double_space_count > 0 {
            stats.blessings_applied += double_space_count;
        }

        // Fix common typos
        for (typo, correct) in &self.typo_dict {
            let before = result.clone();
            result = result.replace(typo, correct);
            if result != before {
                stats.blessings_applied += 1;
            }
        }

        // Fix triple+ newlines -> double newline
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
            stats.blessings_applied += 1;
        }

        // Fix space before punctuation
        let before = result.clone();
        result = result.replace(" .", ".");
        if result != before {
            stats.blessings_applied += 1;
        }
        let before = result.clone();
        result = result.replace(" ,", ",");
        if result != before {
            stats.blessings_applied += 1;
        }
        let before = result.clone();
        result = result.replace(" !", "!");
        if result != before {
            stats.blessings_applied += 1;
        }
        let before = result.clone();
        result = result.replace(" ?", "?");
        if result != before {
            stats.blessings_applied += 1;
        }

        result
    }

    /// Apply Level 2: Harmony blessings (Wikipedia structure fixes)
    fn apply_harmony_blessings(&self, text: &str, stats: &mut BlessingStats) -> String {
        let mut result = text.to_string();

        // Fix Wikipedia category capitalization
        if result.contains("[[category:") {
            result = result.replace("[[category:", "[[Category:");
            stats.blessings_applied += 1;
        }

        if result.contains("[[CATEGORY:") {
            result = result.replace("[[CATEGORY:", "[[Category:");
            stats.blessings_applied += 1;
        }

        // Fix template capitalization
        if result.contains("{{template ") {
            result = result.replace("{{template ", "{{Template ");
            stats.blessings_applied += 1;
        }

        // Fix broken wikilinks [[link ]] -> [[link]]
        let prev_result = result.clone();
        result = result.replace("]] ", "]]");
        if result != prev_result {
            stats.blessings_applied += 1;
        }
        let prev_result = result.clone();
        result = result.replace(" [[", "[[");
        if result != prev_result {
            stats.blessings_applied += 1;
        }

        // Fix heading spacing: ensure single space after #
        for i in 1..=6 {
            let wrong = format!("{}  ", "#".repeat(i));
            let right = format!("{} ", "#".repeat(i));
            if result.contains(&wrong) {
                result = result.replace(&wrong, &right);
                stats.blessings_applied += 1;
            }
        }

        // Fix list formatting
        result = result.replace("\n*", "\n* "); // Ensure space after bullet
        result = result.replace("\n-", "\n- "); // Ensure space after dash

        result
    }

    /// Apply Level 3: Creative blessings (training data augmentation)
    fn apply_creative_blessings(&self, text: &str, stats: &mut BlessingStats) -> String {
        let mut result = text.to_string();

        // Simple pseudo-random number generator based on seed
        let mut rng = self.seed;
        let mut next_random = || {
            rng = (rng.wrapping_mul(1103515245).wrapping_add(12345)) % (1 << 31);
            rng
        };

        // Add semantic variations (simple version)
        // In a full implementation, this would use ML models

        let variations = vec![
            ("is a", "is an example of"),
            ("the", "this"),
            ("and", "as well as"),
            ("but", "however"),
            ("also", "additionally"),
        ];

        // Apply random variations (about 5% of the time)
        for (from, to) in variations {
            if next_random() % 100 < 5 && result.contains(from) {
                // Replace first occurrence
                result = result.replacen(from, to, 1);
                stats.blessings_applied += 1;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blessing_level_parsing() {
        assert_eq!(BlessingLevel::from_i32(0).unwrap(), BlessingLevel::Strict);
        assert_eq!(
            BlessingLevel::from_i32(1).unwrap(),
            BlessingLevel::MinorBlessings
        );
        assert_eq!(BlessingLevel::from_i32(2).unwrap(), BlessingLevel::Harmony);
        assert_eq!(BlessingLevel::from_i32(3).unwrap(), BlessingLevel::Creative);
        assert!(BlessingLevel::from_i32(4).is_err());
    }

    #[test]
    fn test_strict_mode() {
        let angel = Angel::new(BlessingLevel::Strict);
        let input = "This  has  double  spaces";
        let (output, stats) = angel.bless(input).unwrap();

        // Strict mode doesn't change anything
        assert_eq!(output, input);
        assert_eq!(stats.blessings_applied, 0);
    }

    #[test]
    fn test_minor_blessings() {
        let angel = Angel::new(BlessingLevel::MinorBlessings);
        let input = "This  has  double  spaces and teh is a typo";
        let (output, stats) = angel.bless(input).unwrap();

        // Should fix spacing and typo
        assert!(output.contains("the is a typo"));
        assert!(!output.contains("  "));
        assert!(stats.blessings_applied > 0);
    }

    #[test]
    fn test_harmony_blessings() {
        let angel = Angel::new(BlessingLevel::Harmony);
        let input = "[[category:test]] and [[CATEGORY:other]]";
        let (output, stats) = angel.bless(input).unwrap();

        // Should fix category capitalization
        assert!(output.contains("[[Category:test]]"));
        assert!(output.contains("[[Category:other]]"));
        assert!(stats.blessings_applied >= 2);
    }

    #[test]
    fn test_thermodynamics() {
        let angel = Angel::new(BlessingLevel::MinorBlessings);
        let input = "This  has  typos  and  teh  recieve";
        let (_output, stats) = angel.bless(input).unwrap();

        // Should have positive energy added
        assert!(stats.energy_added > 0.0);
        assert_eq!(stats.entropy_added, stats.blessings_applied as f64);
    }
}
