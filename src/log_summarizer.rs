//! # Smart Log Summarization
//!
//! AI-friendly log analysis that surfaces what matters and filters noise.
//!
//! ## Philosophy
//!
//! Logs are 90% repetitive noise. This module uses marqant's compression
//! intelligence to find patterns and novelty detection to surface anomalies.
//!
//! ## Usage
//!
//! ```bash
//! tail -100 app.log | mq tail
//! mq tail -n 1000 app.log
//! tail -f app.log | mq tail --stream
//! ```

use crate::novelty::{NoveltyClass, NoveltyTracker};
use crate::semantic::{SemanticToken, SemanticUnit};
use std::collections::HashMap;

/// Configuration for log summarization
#[derive(Debug, Clone)]
pub struct SummarizerConfig {
    /// Minimum novelty threshold (0.0-1.0)
    pub novelty_threshold: f32,

    /// Show background noise stats
    pub show_noise: bool,

    /// Maximum lines to show per category
    pub max_per_category: usize,

    /// Use emojis in output
    pub use_emojis: bool,
}

impl Default for SummarizerConfig {
    fn default() -> Self {
        Self {
            novelty_threshold: 0.1,
            show_noise: true,
            max_per_category: 10,
            use_emojis: true,
        }
    }
}

/// A categorized log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Original log line
    pub line: String,

    /// Pattern signature (for grouping similar lines)
    pub pattern: String,

    /// Novelty score
    pub novelty: f32,

    /// Classification
    pub class: NoveltyClass,

    /// Occurrence count
    pub count: usize,
}

/// Summary of analyzed logs
#[derive(Debug)]
pub struct LogSummary {
    /// Total lines analyzed
    pub total_lines: usize,

    /// Revolutionary patterns (never seen before)
    pub revolutionary: Vec<LogEntry>,

    /// Fresh/interesting patterns
    pub important: Vec<LogEntry>,

    /// Familiar patterns
    pub familiar: Vec<LogEntry>,

    /// Background noise (filtered)
    pub noise: Vec<(String, usize)>,

    /// Total lines filtered as noise
    pub noise_count: usize,
}

/// Smart log summarizer using marqant compression + novelty detection
pub struct LogSummarizer {
    /// Novelty tracker
    novelty_tracker: NoveltyTracker,

    /// Pattern frequency map
    pattern_counts: HashMap<String, usize>,

    /// Configuration (reserved for future use)
    _config: SummarizerConfig,
}

impl LogSummarizer {
    /// Create a new log summarizer
    pub fn new(config: SummarizerConfig) -> Self {
        Self {
            novelty_tracker: NoveltyTracker::new(),
            pattern_counts: HashMap::new(),
            _config: config,
        }
    }

    /// Analyze log lines and generate summary
    pub fn summarize(&mut self, lines: &[String]) -> LogSummary {
        let mut revolutionary = Vec::new();
        let mut important = Vec::new();
        let mut familiar = Vec::new();
        let mut noise = Vec::new();
        let mut noise_count = 0;

        // Analyze each line
        for line in lines {
            let entry = self.analyze_line(line);

            match entry.class {
                NoveltyClass::Revolutionary => revolutionary.push(entry),
                NoveltyClass::Fresh | NoveltyClass::Interesting => important.push(entry),
                NoveltyClass::Familiar => familiar.push(entry),
                NoveltyClass::Stale | NoveltyClass::BackgroundNoise => {
                    noise.push((entry.pattern, entry.count));
                    noise_count += entry.count;
                }
            }
        }

        // Deduplicate and sort by novelty
        revolutionary.sort_by(|a, b| b.novelty.partial_cmp(&a.novelty).unwrap());
        important.sort_by(|a, b| b.novelty.partial_cmp(&a.novelty).unwrap());
        familiar.sort_by(|a, b| b.novelty.partial_cmp(&a.novelty).unwrap());

        // Deduplicate by pattern
        revolutionary = Self::deduplicate_entries(revolutionary);
        important = Self::deduplicate_entries(important);
        familiar = Self::deduplicate_entries(familiar);

        // Aggregate noise patterns
        let noise = Self::aggregate_noise_patterns(noise);

        LogSummary {
            total_lines: lines.len(),
            revolutionary,
            important,
            familiar,
            noise,
            noise_count,
        }
    }

    /// Analyze a single log line
    fn analyze_line(&mut self, line: &str) -> LogEntry {
        // Extract pattern (simplified: ignore timestamps, IPs, numbers)
        let pattern = self.extract_pattern(line);

        // Track pattern frequency
        *self.pattern_counts.entry(pattern.clone()).or_insert(0) += 1;
        let count = self.pattern_counts[&pattern];

        // Create semantic unit for novelty detection
        let semantic_unit = self.line_to_semantic_unit(line);

        // Calculate novelty
        let novelty_score = self.novelty_tracker.calculate_novelty(&[semantic_unit]);

        LogEntry {
            line: line.to_string(),
            pattern,
            novelty: novelty_score.value,
            class: novelty_score.classification,
            count,
        }
    }

    /// Extract pattern from log line (remove variable parts)
    fn extract_pattern(&self, line: &str) -> String {
        let mut pattern = line.to_string();

        // Remove timestamps (ISO 8601, syslog, etc.)
        pattern = regex::Regex::new(r"\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}")
            .unwrap()
            .replace_all(&pattern, "<TIMESTAMP>")
            .to_string();

        // Remove IPs
        pattern = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}")
            .unwrap()
            .replace_all(&pattern, "<IP>")
            .to_string();

        // Remove numbers (but keep log levels)
        pattern = regex::Regex::new(r"\b\d+\b")
            .unwrap()
            .replace_all(&pattern, "<NUM>")
            .to_string();

        // Remove UUIDs
        pattern =
            regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
                .unwrap()
                .replace_all(&pattern, "<UUID>")
                .to_string();

        pattern
    }

    /// Convert log line to semantic unit for novelty tracking
    fn line_to_semantic_unit(&self, line: &str) -> SemanticUnit {
        // Analyze log line and extract semantic meaning
        let mut tokens = Vec::new();
        let line_lower = line.to_lowercase();

        // Detect log level/severity
        if line_lower.contains("error") || line_lower.contains("fatal") {
            tokens.push(SemanticToken::EmotionFrustrated);
        } else if line_lower.contains("warn") {
            tokens.push(SemanticToken::QualifierMedium);
        } else if line_lower.contains("info") {
            tokens.push(SemanticToken::QualifierLow);
        }

        // Detect process states
        if line_lower.contains("start") || line_lower.contains("begin") {
            tokens.push(SemanticToken::ProcessActive);
        } else if line_lower.contains("complete") || line_lower.contains("success") {
            tokens.push(SemanticToken::ProcessComplete);
        }

        // Detect system entities
        if line_lower.contains("database") || line_lower.contains("db") {
            tokens.push(SemanticToken::EntitySystem);
        }

        // If no tokens detected, use a generic token
        if tokens.is_empty() {
            tokens.push(SemanticToken::ContextProgramming);
        }

        // Calculate intensity based on log level
        let intensity = if line_lower.contains("error") || line_lower.contains("fatal") {
            1.0
        } else if line_lower.contains("warn") {
            0.7
        } else {
            0.3
        };

        SemanticUnit {
            tokens,
            metadata: HashMap::new(),
            intensity,
        }
    }

    /// Deduplicate entries by pattern
    fn deduplicate_entries(entries: Vec<LogEntry>) -> Vec<LogEntry> {
        let mut seen = HashMap::new();
        let mut result = Vec::new();

        for entry in entries {
            if !seen.contains_key(&entry.pattern) {
                seen.insert(entry.pattern.clone(), true);
                result.push(entry);
            }
        }

        result
    }

    /// Aggregate noise patterns and count occurrences
    fn aggregate_noise_patterns(noise: Vec<(String, usize)>) -> Vec<(String, usize)> {
        let mut aggregated = HashMap::new();

        for (pattern, count) in noise {
            *aggregated.entry(pattern).or_insert(0) += count;
        }

        let mut result: Vec<_> = aggregated.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending

        result
    }
}

impl LogSummary {
    /// Format summary as human-readable text
    pub fn format(&self, config: &SummarizerConfig) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "📊 Log Summary ({} lines analyzed)\n",
            self.total_lines
        ));
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

        // Revolutionary patterns
        if !self.revolutionary.is_empty() {
            let emoji = if config.use_emojis { "💎 " } else { "" };
            output.push_str(&format!(
                "{}NOVEL PATTERNS ({})\n",
                emoji,
                self.revolutionary.len()
            ));

            for entry in self.revolutionary.iter().take(config.max_per_category) {
                output.push_str(&format!("  • {}\n", entry.line));
            }
            output.push('\n');
        }

        // Important patterns
        if !self.important.is_empty() {
            let emoji = if config.use_emojis { "🌟 " } else { "" };
            output.push_str(&format!("{}IMPORTANT ({})\n", emoji, self.important.len()));

            for entry in self.important.iter().take(config.max_per_category) {
                output.push_str(&format!("  • {}\n", entry.line));
            }
            output.push('\n');
        }

        // Familiar patterns
        if !self.familiar.is_empty() && self.familiar.len() < 20 {
            let emoji = if config.use_emojis { "📝 " } else { "" };
            output.push_str(&format!("{}FAMILIAR ({})\n", emoji, self.familiar.len()));

            for entry in self.familiar.iter().take(config.max_per_category) {
                output.push_str(&format!(
                    "  • {} (seen {} times)\n",
                    entry.line, entry.count
                ));
            }
            output.push('\n');
        }

        // Background noise summary
        if config.show_noise && self.noise_count > 0 {
            let emoji = if config.use_emojis { "💤 " } else { "" };
            output.push_str(&format!(
                "{}BACKGROUND NOISE (filtered {} lines)\n",
                emoji, self.noise_count
            ));

            // Show top noise patterns
            for (pattern, count) in self.noise.iter().take(5) {
                let truncated = if pattern.len() > 80 {
                    format!("{}...", &pattern[..77])
                } else {
                    pattern.clone()
                };
                output.push_str(&format!("  • {}: {} occurrences\n", truncated, count));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_extraction() {
        let config = SummarizerConfig::default();
        let summarizer = LogSummarizer::new(config);

        let line1 = "2024-01-15 10:30:45 ERROR Connection timeout to 192.168.1.100";
        let line2 = "2024-01-15 10:31:22 ERROR Connection timeout to 192.168.1.200";

        let pattern1 = summarizer.extract_pattern(line1);
        let pattern2 = summarizer.extract_pattern(line2);

        // Patterns should be identical (variables removed)
        assert_eq!(pattern1, pattern2);
        assert!(pattern1.contains("<TIMESTAMP>"));
        assert!(pattern1.contains("<IP>"));
    }

    #[test]
    fn test_novelty_detection() {
        let config = SummarizerConfig::default();
        let mut summarizer = LogSummarizer::new(config);

        // Add many repeated lines to trigger familiar/noise classification
        let mut lines = vec![];
        for _ in 0..10 {
            lines.push("INFO: Server started successfully".to_string());
        }
        lines.push("ERROR: Database connection failed".to_string());

        let summary = summarizer.summarize(&lines);

        // Should have novel patterns
        assert!(summary.revolutionary.len() > 0 || summary.important.len() > 0);

        // Should detect repetition with many occurrences
        // After 10 repetitions, patterns should be familiar or noise
        assert!(
            summary.noise_count > 0 || summary.familiar.len() > 0,
            "Expected repetition detection. Revolutionary: {}, Important: {}, Familiar: {}, Noise: {}",
            summary.revolutionary.len(),
            summary.important.len(),
            summary.familiar.len(),
            summary.noise_count
        );
    }
}
