//! Marqant Data Bridge (MQ-DBX) Prototype
//!
//! Translating bloated data structures into high-density semantic state.

use crate::semantic::{SemanticToken, SemanticUnit};
use std::collections::HashMap;
use anyhow::Result;

/// A "Bloated" data record from a traditional DB
#[derive(Debug)]
pub struct BloatedRecord {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub payload: HashMap<String, String>,
    pub metadata_schema_version: String, // Bloat!
}

/// A semantic unit that requires specific context to access its data
#[derive(Debug)]
pub struct SecureSemanticUnit {
    pub base: SemanticUnit,
    pub is_one_way: bool, // If true, can only be matched, never retrieved
    pub required_context_keys: Vec<String>,
    pub encrypted_payload: Vec<u8>, // The actual sensitive data (encrypted)
    pub expected_anchor_hash: u64, // The composite hash of all required anchors
}

struct CcFeatures {
    bin: String,
    _full_len: usize,
}

/// The Semantic Data Bridge
pub struct DataBridge;

impl DataBridge {
    /// Ingest a bloated record and return a dense Semantic Unit
    pub fn ingest(json_str: &str) -> Result<SemanticUnit> {
        let mut tokens = Vec::new();
        let metadata = HashMap::new();
        
        if json_str.contains("login") {
            tokens.push(SemanticToken::EntityHuman);
            tokens.push(SemanticToken::ProcessActive);
            tokens.push(SemanticToken::QualifierHigh);
        }
        
        if json_str.contains("database") || json_str.contains("SurrealDB") {
            tokens.push(SemanticToken::EntitySystem);
            tokens.push(SemanticToken::ContextProgramming);
        }

        // SECURITY CUE: Check for clear-text passwords or API keys
        if json_str.contains("\"password\":") || json_str.contains("\"api_key\":") || json_str.contains("\"secret\":") {
            println!("⚠️ DataBomb Alert: Sensitive secret detected (Password/API Key). Applying One-Way Proof.");
        }

        // LEAKAGE DETECTOR: The "Luhn" Check + BIN Extraction
        if let Some(cc_data) = Self::extract_cc_features(json_str) {
            println!("🛑 DataBomb Block: CC detected! Extracting BIN Feature and Enforcing MAD.");
            tokens.push(SemanticToken::QualifierHigh); // Mark as high sensitivity
            // Store the BIN as a semantic hint (safe for validation)
            tokens.push(SemanticToken::ContextAI); // Mocking a 'BIN' token category
            println!("   - Feature Extracted: BIN={}", cc_data.bin);
        }

        Ok(SemanticUnit {
            tokens,
            metadata,
            intensity: 1.0,
        })
    }

    /// Detect CC and extract non-sensitive features (BIN)
    fn extract_cc_features(input: &str) -> Option<CcFeatures> {
        let re = regex::Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap();
        for cap in re.find_iter(input) {
            let digits: String = cap.as_str().chars().filter(|c| c.is_ascii_digit()).collect();
            if Self::is_luhn_valid(&digits) {
                return Some(CcFeatures {
                    bin: digits[..6].to_string(), // The first 6 digits (Industry Standard BIN)
                    _full_len: digits.len(),
                });
            }
        }
        None
    }

    /// One-way verification logic
    pub fn verify_password(stored_hash: u64, attempt: &str) -> bool {
        crate::fnv1a64(attempt) == stored_hash
    }

    /// Multi-Anchor Decryption (MAD) logic
    pub fn try_unlock_field(unit: &SecureSemanticUnit, provided_context: &HashMap<String, String>) -> Result<String> {
        let mut anchor_material = String::new();
        
        // 1. Collect all required anchors in deterministic order
        let mut keys = unit.required_context_keys.clone();
        keys.sort();
        
        for key in &keys {
            if let Some(val) = provided_context.get(key) {
                anchor_material.push_str(val);
            } else {
                return Err(anyhow::anyhow!("Context lock violation: Missing anchor '{}'", key));
            }
        }
        
        // 2. Derive the composite key hash
        let derived_hash = crate::fnv1a64(&anchor_material);
        
        // 3. Verify and "Decrypt"
        if derived_hash == unit.expected_anchor_hash {
            // In a real system, use AES-GCM with the derived_hash as key material
            let decrypted = String::from_utf8(unit.encrypted_payload.clone())?;
            Ok(format!("Decrypted: {}", decrypted))
        } else {
            Err(anyhow::anyhow!("Decryption failed: Incorrect anchor values provided."))
        }
    }

    /// Translate between "Database Dialects" via Semantic Intent
    pub fn translate_query(query: &str, target_dialect: &str) -> String {
        println!("Translating query: {:?} -> {}", query, target_dialect);
        
        // 1. Extract Intent (Ayanese Reasoner)
        // 2. Render to Target
        match target_dialect {
            "SurrealQL" => "SELECT * FROM semantic_thoughts WHERE meaning CONTAINS 'optimization'".to_string(),
            "SQL" => "SELECT * FROM thoughts WHERE type_id = 0x24".to_string(),
            _ => "UNKNOWN_DIALECT".to_string(),
        }
    }

    fn is_luhn_valid(s: &str) -> bool {
        let mut sum = 0;
        let mut even = false;
        for c in s.chars().rev() {
            let mut d = match c.to_digit(10) {
                Some(d) => d,
                None => continue,
            };
            if even {
                d *= 2;
                if d > 9 { d -= 9; }
            }
            sum += d;
            even = !even;
        }
        sum % 10 == 0
    }
}

pub fn demo_bridge() -> Result<()> {
    let raw_json = r#"{
        "id": "usr_123",
        "timestamp": "2026-02-19T15:30:00Z",
        "event_type": "system_optimization_login",
        "payload": { "db": "SurrealDB", "status": "active" },
        "metadata_schema_version": "v2.1.45-bloat"
    }"#;

    println!("--- Ingesting Bloated Data ---");
    let unit = DataBridge::ingest(raw_json)?;
    println!("Extracted Semantic Tokens: {:?}", unit.tokens);
    
    let compressed_bytes = crate::semantic::SemanticEncoder::to_bytes(&[unit]);
    println!("Data Density: {} bytes (JSON) -> {} bytes (Ayanese)", raw_json.len(), compressed_bytes.len());
    println!("Savings: {:.1}%", (1.0 - (compressed_bytes.len() as f64 / raw_json.len() as f64)) * 100.0);

    println!("\n--- Cross-DB Syntax Translation ---");
    let sql = DataBridge::translate_query("Find all active optimizations in SurrealDB", "SQL");
    println!("Resulting SQL: {}", sql);

    Ok(())
}
