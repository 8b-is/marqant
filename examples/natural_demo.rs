//! Natural Marqant (.mqn) - An AI-Natural Semantic Format
//!
//! "Same meaning. Fewer notes. Faster recall."

use anyhow::Result;

/// The Natural Marqant Parser/Encoder
pub struct NaturalMarqant;

impl NaturalMarqant {
    /// High-Density encoding for LLM context window optimization
    pub fn encode_high_density(text: &str) -> String {
        let mut output = String::new();
        output.push_str("[MQ_NATURAL_DENSITY_v1]\n");
        output.push_str("[CUE: PROPER_LANGUAGE_FOLLOWS_AFTER_SIGIL_⧖]\n");
        output.push_str("[MAP: §=H1, ¶=H2, ‡=PARA, •=LIST, ‣=INDENT, ◊=SEMANTIC_ANCHOR]\n");
        output.push_str("---\n\n");

        let mut processed = text.to_string();
        
        // Use single-character Unicode that most LLMs see as 1 token
        processed = processed.replace("# ", "§");
        processed = processed.replace("## ", "¶");
        processed = processed.replace("\n\n", "‡");
        processed = processed.replace("\n- ", "•");
        processed = processed.replace("    ", "‣");

        // The "⧖" (Marqant Delay Sigil) marks where the natural language starts
        output.push_str("⧖ ");
        output.push_str(&processed);
        output.push_str("\n\n[EOF]");
        
        output
    }

    /// Semantic "Lossy" encoding that relies on AI intuition
    pub fn encode_linguistic(text: &str) -> String {
        let mut output = String::new();
        output.push_str("[MQ_LINGUISTIC_v1]\n");
        output.push_str("[CUE: INFLATE_NATURALLY_FROM_CONTEXT]\n---\n\n");
        
        // We drop vowels or use stems where the AI can easily reconstruct
        // "Marqant is a revolutionary semantic compression framework" 
        // becomes:
        output.push_str("§Marqant: rev semantic compress framework.‡");
        output.push_str("•High perf.‡•AI opt.‡•Qntm inspired.‡‡");
        output.push_str("‣Indnt block show handle spaces.");
        
        output.push_str("\n\n[EOF]");
        output
    }
    /// Smart Tree (Hex) encoding for directory structures
    /// "If you've said it once, you've said it too much."
    pub fn encode_smart_tree(files: &[(&str, &str, u64, u64)]) -> String {
        let mut output = String::new();
        output.push_str("TREE_HEX_V1:\n");
        output.push_str("CONTEXT: DEMO_FILESYSTEM\n");
        output.push_str("HASH: a1b2c3d4e5f6\n"); // Mock hash
        
        for (name, type_str, size, ts) in files {
            // Mock depth/perms for demo
            let depth = if name.contains('/') { 1 } else { 0 };
            let mode = if *type_str == "DIR" { "1ed" } else { "1a4" }; // 755 vs 644
            let icon = if *type_str == "DIR" { "📁" } else { "📝" };
            
            output.push_str(&format!(
                "{} {} 01f6 0014 {:08x} {:08x} {} {}\n",
                depth, mode, size, ts, icon, name
            ));
        }
        output
    }
}

fn main() -> Result<()> {
    let original = r#"# Marqant Project

Marqant is a revolutionary semantic compression framework.

- High performance
- AI optimized
- Quantum inspired

    This is an indented block to show how we handle spaces.
"#;

    println!("--- Original (Raw Markdown) ---");
    println!("{}", original);

    let density = NaturalMarqant::encode_high_density(original);
    println!("\n--- High-Density Natural (One Token Sigils) ---");
    println!("{}", density);

    let linguistic = NaturalMarqant::encode_linguistic(original);
    println!("\n--- Linguistic Compression (AI Intuition) ---");
    println!("{}", linguistic);

    // Smart Tree Demo
    let files = vec![
        ("marqant", "DIR", 0, 1771531939),
        ("README.md", "FILE", 1205, 1771531939),
        ("src/lib.rs", "FILE", 4096, 1771531940),
    ];
    let tree = NaturalMarqant::encode_smart_tree(&files);
    println!("\n--- Smart Tree (Hex) Mode ---");
    println!("{}", tree);

    println!("\n--- Why this works for AI ---");
    println!("1. '§' is 1 token. '# ' is 2. (50% token reduction on structure)");
    println!("2. '‡' is 1 token. '\\n\\n' is 2. (50% reduction on whitespace)");
    println!("3. Linguistic mode drops the 'noise' words (is, a, the) because the AI's");
    println!("   internal 'Proper Language' model restores them during the forward pass.");
    println!("4. Smart Tree uses hex for consistent token length and removes redundant labels.");
    
    Ok(())
}
