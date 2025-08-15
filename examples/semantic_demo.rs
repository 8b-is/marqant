use marqant::semantic::{SemanticEncoder, SemanticWaveEncoder, UniversalRenderer};

fn main() {
    println!("🎯 Marqant Semantic Compression Demo");
    println!("=====================================\n");

    let test_texts = vec![
        "Alexandra is learning Rust with Claude!",
        "The team is excited about the optimization results!",
        "Claude is teaching programming concepts with patience.",
    ];

    for text in test_texts {
        println!("📝 Original Text:");
        println!("   {}\n", text);

        // Encode to semantic units
        let units = SemanticEncoder::encode(text);
        println!("🧠 Semantic Units Detected:");
        for unit in &units {
            println!("   Tokens: {:?}", unit.tokens);
            println!("   Metadata: {:?}", unit.metadata);
            println!("   Intensity: {:.1}\n", unit.intensity);
        }

        // Convert to binary
        let bytes = SemanticEncoder::to_bytes(&units);
        println!("💾 Binary Size:");
        println!("   Original: {} bytes", text.len());
        println!("   Semantic: {} bytes", bytes.len());
        println!(
            "   Ratio: {:.1}%\n",
            (bytes.len() as f32 / text.len() as f32) * 100.0
        );

        // Render to different formats
        println!("🌐 Universal Rendering:");

        let english = UniversalRenderer::to_english(&units);
        println!("   English: {}", english);

        let emoji = UniversalRenderer::to_emoji(&units);
        println!("   Emoji: {}", emoji);

        let dns = UniversalRenderer::to_dns_fingerprint(&units);
        println!("   DNS: {}", dns);

        // Convert to wave patterns
        let waves = SemanticWaveEncoder::to_wave_pattern(&units);
        println!("\n🌊 Wave Patterns (for MEM|8):");
        for (i, (freq, amp, phase)) in waves.iter().take(3).enumerate() {
            println!(
                "   Wave {}: {:.1}Hz @ {:.1} amplitude, {:.1}° phase",
                i + 1,
                freq,
                amp,
                phase
            );
        }

        println!("\n{}", "=".repeat(50));
        println!();
    }

    println!("✨ The Future of Compression:");
    println!("   - No Unicode needed");
    println!("   - No ASCII needed");
    println!("   - Just pure meaning");
    println!("   - Universal understanding across all languages");
    println!("\n💭 \"Why send words when you can send thoughts?\"");
}
