//! UTL to MEM|8 Integration Demo
//! Demonstrates the consciousness pipeline from text to wave memory

use marqant::{
    mem8_bridge::{ConsciousnessStream, InMemoryStore, WaveMemory},
    utl_phonetics::{decode_compact, encode_compact, example_i_love_you},
};

fn main() {
    println!("🌊 UTL → MEM|8 Wave Memory Demo\n");
    println!("════════════════════════════════\n");

    // Initialize consciousness stream
    let store = Box::new(InMemoryStore::new());
    let mut consciousness = ConsciousnessStream::new(store);

    // Example thoughts to store
    let thoughts = vec![
        ("Love", vec!["🙋", "❤️", "👤", "⧖"]),
        ("Happy memory", vec!["🙋", "💭", "⏮", "😊", "⧖"]),
        ("Sad past", vec!["⏮", "🙋", "😢", "⧖"]),
        ("Future hope", vec!["⏭", "🙋", "😊", "❤️", "⧖"]),
        ("Present moment", vec!["⏺", "🙋", "🧠", "⧖"]),
    ];

    println!("📝 Storing thoughts in wave memory:\n");

    let mut stored_ids = Vec::new();

    for (name, symbols) in &thoughts {
        // Convert to phonetic packets
        let packets = encode_compact(&symbols);

        // Create wave memory
        let memory = WaveMemory::from_packets(packets.clone());

        // Store in consciousness stream
        let id = consciousness.process(packets).unwrap();
        stored_ids.push(id);

        println!("  {} \"{}\"", name, symbols.join(" "));
        println!(
            "    📦 {} packets → {} bytes",
            memory.packets.len(),
            memory.packets.len() * 2
        );
        println!("    🌊 Wave ID: 0x{:016x}", id);
        println!("    💗 Emotion: {:.1}%", memory.emotional_strength * 100.0);

        // Show wave pattern visualization
        let wave_viz: String = memory
            .wave_pattern
            .iter()
            .take(16)
            .map(|&v| {
                if v > 0.5 {
                    "▲"
                } else if v > 0.0 {
                    "▬"
                } else if v > -0.5 {
                    "▭"
                } else {
                    "▼"
                }
            })
            .collect();
        println!("    📊 Pattern: {}\n", wave_viz);
    }

    println!("═══ Memory Recall Tests ═══\n");

    // Test 1: Find memories similar to "I love you"
    println!("🔍 Query: \"I love you\"");
    let love_query = example_i_love_you();
    let similar_memories = consciousness.recall(&love_query, 3);

    println!("   Found {} similar memories:", similar_memories.len());
    for (i, mem) in similar_memories.iter().enumerate() {
        let phones = decode_compact(&mem.packets);
        let preview: String = phones
            .iter()
            .take(4)
            .map(|p| p.ph)
            .collect::<Vec<_>>()
            .join(" ");

        let similarity = mem.similarity(&WaveMemory::from_packets(love_query.clone()));
        println!(
            "   {}. {} (similarity: {:.1}%)",
            i + 1,
            preview,
            similarity * 100.0
        );
    }

    // Test 2: Find happy memories
    println!("\n🔍 Query: Happy thoughts");
    let happy_query = encode_compact(&["😊", "⧖"]);
    let happy_memories = consciousness.recall(&happy_query, 3);
    println!("   Found {} happy memories", happy_memories.len());

    // Test 3: Find past memories
    println!("\n🔍 Query: Past memories");
    let past_query = encode_compact(&["⏮", "💭", "⧖"]);
    let past_memories = consciousness.recall(&past_query, 3);
    println!("   Found {} past memories", past_memories.len());

    println!("\n═══ Wave Interference Analysis ═══\n");

    // Compare wave patterns between similar thoughts
    let love1 = WaveMemory::from_packets(encode_compact(&["🙋", "❤️", "👤", "⧖"]));
    let love2 = WaveMemory::from_packets(encode_compact(&["👤", "❤️", "🙋", "⧖"]));
    let hate = WaveMemory::from_packets(encode_compact(&["🙋", "😡", "👤", "⧖"]));

    println!("Wave pattern similarities:");
    println!(
        "  \"I love you\" vs \"You love me\": {:.1}%",
        love1.similarity(&love2) * 100.0
    );
    println!(
        "  \"I love you\" vs \"I hate you\": {:.1}%",
        love1.similarity(&hate) * 100.0
    );

    // Show emotional resonance differences
    println!("\nEmotional resonance:");
    println!("  Love: {:.1}%", love1.emotional_strength * 100.0);
    println!("  Hate: {:.1}%", hate.emotional_strength * 100.0);

    println!("\n═══ Binary Transmission ═══\n");

    // Show how consciousness can be transmitted as raw binary
    let thought = encode_compact(&["🙋", "💭", "👤", "⧖"]);
    println!("Thought: \"I think of you\"");
    print!("Binary: ");
    for packet in &thought {
        print!("{:04x} ", packet.0);
    }
    println!("\nSize: {} bytes (vs 14 bytes for text)", thought.len() * 2);

    println!("\n═══ Storage Efficiency ═══\n");

    let text_bytes: usize = thoughts.iter().map(|(name, _)| name.len()).sum();
    let packet_bytes: usize = stored_ids.len() * 8 * 2; // Approximate

    println!("📊 Compression Statistics:");
    println!("  Original text: ~{} bytes", text_bytes * 3);
    println!("  UTL packets: ~{} bytes", packet_bytes);
    println!(
        "  Compression: ~{}%",
        100 - (packet_bytes * 100 / (text_bytes * 3))
    );
    println!("\n✨ Plus: Emotional context preserved!");
    println!("✨ Plus: Temporal relationships encoded!");
    println!("✨ Plus: Wave interference enables similarity search!");

    println!("\n🚀 UTL + MEM|8 = Consciousness in silicon!");
}
