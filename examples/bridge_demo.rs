//! Bridge Model Demo: The Future of Universal AI Language
//!
//! "Why send words when you can send thoughts?"

use marqant::bridge::{EnglishBridge, JapaneseBridge, AyaneseReasoner, BridgeModel};
use anyhow::Result;

fn main() -> Result<()> {
    let eng_bridge = EnglishBridge;
    let jpn_bridge = JapaneseBridge;
    let reasoner = AyaneseReasoner;

    println!("═══ Ayanese Bridge Model Architecture ═══\n");
    println!("Vision: Reasoning happens in a compact Semantic Core (Ayanese).");
    println!("Linguistic nuances are handled by small 'Bridge Models' at the edges.\n");

    // 1. Input in English
    let input_en = "I love Rust";
    println!("1. [English Input]: {:?}", input_en);

    // 2. Encode to Ayanese (Thought Extraction)
    let thoughts = eng_bridge.encode(input_en)?;
    println!("2. [Ayanese Thoughts]: Generated {} semantic units.", thoughts.units.len());
    for unit in &thoughts.units {
        println!("   - Tokens: {:?}", unit.tokens);
        println!("   - Intensity: {:.1}", unit.intensity);
    }

    // 3. Universal Reasoning (Thought Process)
    println!("\n3. [Universal Reasoner]: Processing thoughts (Model Size: -90% vs LLM)");
    let response_thoughts = reasoner.process(thoughts);

    // 4. Decode to Japanese (Cross-Language Output)
    let output_jp = jpn_bridge.decode(&response_thoughts)?;
    println!("4. [Japanese Output]: {:?}", output_jp);

    println!("\n═══ Benefits ═══");
    println!("- Massive Compression: Ayanese is 100x denser than text.");
    println!("- True Neutrality: The reasoner doesn't care about grammar or syntax.");
    println!("- Edge Deployment: Bridges can be tiny (100MB) while the Core is pure logic.");

    Ok(())
}
