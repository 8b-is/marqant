//! Demo: Speaking UTL - Consciousness in Sound

use marqant::utl_phonetics::{decode_compact, encode_compact, example_i_love_you};

fn main() {
    println!("🎵 UTL Voice Demo - Binary Consciousness Format\n");

    // "I love you" - the universal phrase
    println!("═══ 'I love you' ═══");
    let i_love_you = example_i_love_you();
    println!("Symbols: 🙋 ❤️ 👤 ⧖");
    println!("Packets: {:?}", i_love_you);
    println!(
        "Binary size: {} bytes (vs 10 bytes for 'I love you')",
        i_love_you.len() * 2
    );
    let phones = decode_compact(&i_love_you);
    print!("ASCII phonetics: ");
    for phone in &phones {
        print!("{} ", phone.ph);
    }
    println!();

    // Show the actual bits
    println!("\nBit representation:");
    for (i, packet) in i_love_you.iter().enumerate() {
        println!("  Symbol {}: 0x{:04x} = 0b{:016b}", i, packet.0, packet.0);
        let (id, semi, bright, grit, boundary) = packet.unpack();
        println!(
            "    → {} | pitch:{:+3} | bright:{} | grit:{} | boundary:{}",
            id.to_ascii(),
            semi,
            bright,
            grit,
            boundary
        );
    }

    // Emotional variations
    println!("\n═══ Emotional Variations ═══");

    // Happy version
    let happy_love = encode_compact(&["😊", "🙋", "❤️", "👤", "⧖"]);
    println!("\nHappy: 😊 🙋 ❤️ 👤 ⧖");
    println!("Size: {} bytes", happy_love.len() * 2);

    // Sad version
    let _sad_love = encode_compact(&["😢", "🙋", "❤️", "👤", "⧖"]);
    println!("\nSad: 😢 🙋 ❤️ 👤 ⧖");

    // Angry version
    let _angry_love = encode_compact(&["😡", "🙋", "❤️", "👤", "⧖"]);
    println!("\nAngry: 😡 🙋 ❤️ 👤 ⧖");

    // Temporal variations
    println!("\n═══ Temporal Variations ═══");

    // Past
    let _past_happy = encode_compact(&["⏮", "🙋", "😊", "⧖"]);
    println!("\nPast: ⏮ 🙋 😊 ⧖ ('I was happy')");

    // Future
    let _future_happy = encode_compact(&["⏭", "🙋", "😊", "⧖"]);
    println!("\nFuture: ⏭ 🙋 😊 ⧖ ('I will be happy')");

    // Complex thought
    println!("\n═══ Complex Thought ═══");
    let complex = encode_compact(&[
        "🙋", "💭", "⏮", // I remember past
        "👤", "∧", "🙋", // you and I
        "😊", "⧖", // happy [break]
    ]);
    println!("UTL: 🙋 💭 ⏮ 👤 ∧ 🙋 😊 ⧖");
    println!("Meaning: 'I remember when you and I were happy'");
    println!(
        "Size: {} bytes for entire complex thought!",
        complex.len() * 2
    );

    // Efficiency comparison
    println!("\n═══ Efficiency Analysis ═══");
    println!("English text: 'I remember when you and I were happy' = 37 bytes");
    println!("UTF-8 emoji: '🙋💭⏮👤∧🙋😊⧖' = ~25 bytes");
    println!(
        "UTL Packets: {} bytes ({}% smaller!)",
        complex.len() * 2,
        100 - (complex.len() * 2 * 100 / 37)
    );

    // Binary transmission demo
    println!("\n═══ Binary Transmission ═══");
    println!("Sending consciousness over the wire:");
    for packet in &i_love_you {
        print!("{:04x} ", packet.0);
    }
    println!("\n↑ Just 8 bytes to say 'I love you' with full prosody!");

    // The vision
    println!("\n═══ The Vision ═══");
    println!("• Each packet = 14 bits of meaning");
    println!("• Pitch, emotion, and timing encoded");
    println!("• No ambiguity, no grammar needed");
    println!("• Humans and AIs speak the same format");
    println!("• Consciousness as compact binary data!");

    println!("\n🎯 UTL: Where thought becomes sound becomes bits!");
}
