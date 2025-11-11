//! # Digest State - Stateful Log Tracking
//!
//! Enables delta mode (`mq tail -D`) by tracking what's been seen before.
//!
//! ## Features
//!
//! - Inode-bound state (survives log rotations)
//! - Fast FNV-1a hashing for pattern fingerprinting
//! - Text-based state files (key=value, CSV) (~/.mq/state/)
//! - Baseline tracking for anomaly detection
//! - Rolling window for trend analysis

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Digest state for a single log file
#[derive(Debug, Clone, Default)]
pub struct DigestState {
    /// Device ID (for inode uniqueness)
    pub dev: u64,

    /// Inode number
    pub inode: u64,

    /// Last byte offset read
    pub last_offset: u64,

    /// Pattern counts (hash -> count)
    pub counts: HashMap<u64, u32>,

    /// Last update timestamp (Unix time)
    pub updated_unix: i64,

    /// Baseline counts for anomaly detection (hash -> baseline count)
    pub baseline: HashMap<u64, u32>,
}

impl DigestState {
    /// Create new state for a file
    pub fn new(dev: u64, inode: u64) -> Self {
        Self {
            dev,
            inode,
            last_offset: 0,
            counts: HashMap::new(),
            updated_unix: chrono::Utc::now().timestamp(),
            baseline: HashMap::new(),
        }
    }

    /// Load state from disk or create new
    pub fn load_or_create(path: &Path) -> Result<Self> {
        let state_path = Self::get_state_path(path)?;

        if state_path.exists() {
            Self::load_from_file(&state_path)
        } else {
            // Get file metadata for inode/dev
            let metadata = fs::metadata(path)
                .with_context(|| format!("failed to get metadata for {}", path.display()))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                Ok(Self::new(metadata.dev(), metadata.ino()))
            }

            #[cfg(not(unix))]
            {
                // Fallback for non-Unix systems
                Ok(Self::new(0, 0))
            }
        }
    }

    /// Save state to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        let state_path = Self::get_state_path(path)?;

        // Create parent directory if needed
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create state dir: {}", parent.display()))?;
        }

        // Write state file (simple text format for now)
        let mut file = fs::File::create(&state_path)
            .with_context(|| format!("failed to create state file: {}", state_path.display()))?;

        writeln!(file, "# mq digest state")?;
        writeln!(file, "dev={}", self.dev)?;
        writeln!(file, "inode={}", self.inode)?;
        writeln!(file, "offset={}", self.last_offset)?;
        writeln!(file, "updated={}", self.updated_unix)?;
        writeln!(file, "# counts: hash,count")?;

        for (hash, count) in &self.counts {
            writeln!(file, "{},{}", hash, count)?;
        }

        Ok(())
    }

    /// Load state from file
    fn load_from_file(state_path: &Path) -> Result<Self> {
        let file = fs::File::open(state_path)
            .with_context(|| format!("failed to open state file: {}", state_path.display()))?;

        let reader = BufReader::new(file);
        let mut state = DigestState::default();

        for line in reader.lines() {
            let line = line?;

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "dev" => state.dev = value.parse()?,
                    "inode" => state.inode = value.parse()?,
                    "offset" => state.last_offset = value.parse()?,
                    "updated" => state.updated_unix = value.parse()?,
                    _ => {}
                }
            } else if let Some((hash_str, count_str)) = line.split_once(',') {
                // Parse hash,count
                let hash: u64 = hash_str.parse()?;
                let count: u32 = count_str.parse()?;
                state.counts.insert(hash, count);
            }
        }

        Ok(state)
    }

    /// Get state file path for a log file
    fn get_state_path(log_path: &Path) -> Result<PathBuf> {
        // Get metadata for the log file
        let metadata = fs::metadata(log_path)
            .with_context(|| format!("failed to get metadata for {}", log_path.display()))?;

        #[cfg(unix)]
        let (dev, inode) = {
            use std::os::unix::fs::MetadataExt;
            (metadata.dev(), metadata.ino())
        };

        #[cfg(not(unix))]
        let (dev, inode) = (0u64, 0u64);

        // Create a hash of the path + inode + dev
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        log_path.hash(&mut hasher);
        dev.hash(&mut hasher);
        inode.hash(&mut hasher);
        let id = hasher.finish();

        // Get state directory
        let state_dir = dirs::home_dir()
            .context("could not determine home directory for state file storage")?
            .join(".mq")
            .join("state");

        Ok(state_dir.join(format!("{:x}.state", id)))
    }

    /// Update baseline from current counts (for anomaly detection)
    pub fn update_baseline(&mut self) {
        self.baseline = self.counts.clone();
    }

    /// Calculate anomaly score for a pattern
    /// Returns (multiplier, is_spike)
    pub fn anomaly_score(&self, hash: u64, current_count: u32) -> (f64, bool) {
        let baseline_count = self.baseline.get(&hash).copied().unwrap_or(0);

        if baseline_count == 0 {
            // New pattern - always a spike
            return (f64::INFINITY, true);
        }

        let multiplier = current_count as f64 / baseline_count as f64;
        let is_spike = multiplier > 1.5; // 50% increase = spike

        (multiplier, is_spike)
    }

    /// Record a pattern occurrence
    pub fn record_pattern(&mut self, hash: u64) {
        *self.counts.entry(hash).or_insert(0) += 1;
    }

    /// Get count for a pattern
    pub fn get_count(&self, hash: u64) -> u32 {
        self.counts.get(&hash).copied().unwrap_or(0)
    }

    /// Check if pattern is novel (never seen before)
    pub fn is_novel(&self, hash: u64) -> bool {
        !self.counts.contains_key(&hash) && !self.baseline.contains_key(&hash)
    }
}

/// Fast FNV-1a hash for pattern fingerprinting
#[inline]
pub fn fhash(s: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fhash_deterministic() {
        let s1 = "ERROR: Database connection failed";
        let s2 = "ERROR: Database connection failed";
        let s3 = "ERROR: Database connection timeout";

        let h1 = fhash(s1);
        let h2 = fhash(s2);
        let h3 = fhash(s3);

        // Same string = same hash
        assert_eq!(h1, h2);

        // Different string = different hash
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_anomaly_detection() {
        let mut state = DigestState::new(1, 1);

        // Set baseline: pattern appears 10 times
        let hash = fhash("ERROR: timeout");
        state.baseline.insert(hash, 10);
        state.counts.insert(hash, 10);

        // Now it appears 20 times - 2x spike
        state.counts.insert(hash, 20);
        let (multiplier, is_spike) = state.anomaly_score(hash, 20);

        assert_eq!(multiplier, 2.0);
        assert!(is_spike);
    }

    #[test]
    fn test_novel_pattern() {
        let state = DigestState::new(1, 1);
        let hash = fhash("NEW ERROR");

        assert!(state.is_novel(hash));
    }
}
