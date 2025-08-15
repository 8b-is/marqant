use marqant::semantic::{SemanticEncoder, SemanticUnit};
use marqant::novelty::{NoveltyTracker, NoveltyClass};

fn main() {
    println!("💎 Semantic Novelty Tracking Demo");
    println!("==================================");
    println!("'The most interesting marqant is the one never seen before'\n");
    
    let mut tracker = NoveltyTracker::new();
    
    // Simulate a stream of thoughts/documents
    let thought_stream = vec![
        ("Alexandra is learning Rust with Claude!", "alexandra"),
        ("Hello World!", "beginner"),
        ("Alexandra is learning Rust with Claude!", "alexandra"), // Repeat
        ("The quantum waves interfere to create memory patterns", "quantum"),
        ("Hello World!", "beginner"), // Another Hello World
        ("Hello World!", "beginner"), // And another...
        ("Consciousness emerges from wave interference patterns", "consciousness"),
        ("Alexandra is learning Rust with Claude!", "alexandra"), // Third time
        ("Hello World!", "beginner"), // Getting stale...
    ];
    
    println!("📊 Processing thought stream...\n");
    
    for (thought, author) in thought_stream {
        let units = SemanticEncoder::encode(thought);
        let novelty = tracker.calculate_novelty(&units);
        
        println!("💭 Thought: \"{}\"", thought);
        println!("   Author: {}", author);
        println!("   {} Novelty: {:.2} - {:?}", 
                 novelty.classification.emoji(),
                 novelty.value,
                 novelty.classification);
        
        if novelty.is_novel {
            println!("   ✨ FIRST TIME SEEN - Maximum value!");
        } else {
            println!("   📈 Occurrence #{}, Decay factor: {:.2}", 
                     novelty.occurrence, 
                     novelty.decay_factor);
        }
        
        // Show DNS routing based on novelty
        let dns = marqant::novelty::generate_novelty_dns(&novelty, author);
        println!("   🌐 DNS: {}", dns);
        
        println!();
    }
    
    println!("📊 Top Novel Patterns:");
    println!("=======================");
    let top_patterns = tracker.get_top_novel(5);
    for (i, (pattern, value)) in top_patterns.iter().enumerate() {
        println!("{}. Pattern: {} (value: {:.3})", i+1, pattern, value);
    }
    
    println!("\n🧠 Insights:");
    println!("============");
    println!("• 'Hello World!' quickly became {} Background Noise", 
             NoveltyClass::BackgroundNoise.emoji());
    println!("• Quantum/consciousness thoughts remained {} Fresh", 
             NoveltyClass::Fresh.emoji());
    println!("• Repetition causes rapid value decay");
    println!("• Novel thoughts get priority in storage and retrieval");
    
    println!("\n💡 The 'New Car Smell' Algorithm:");
    println!("=================================");
    println!("value = novelty × relevance ÷ repetition_count");
    println!("\nFirst time:  value = 1.0 💎 (Revolutionary)");
    println!("Second time: value ≈ 0.7 🌟 (Fresh)");
    println!("Third time:  value ≈ 0.4 💡 (Interesting)");
    println!("Tenth time:  value ≈ 0.1 📰 (Stale)");
    println!("100th time:  value ≈ 0.0 💤 (Background Noise)");
    
    println!("\n🚀 Applications:");
    println!("================");
    println!("• Priority storage: Novel thoughts get more storage space");
    println!("• Smart caching: Keep novel content, discard repetitive");
    println!("• DNS ranking: revolutionary.topic.q7.is vs noise.topic.q7.is");
    println!("• AI training: Focus on novel patterns, ignore repetition");
    println!("• MEM|8 waves: Novel = high amplitude, Stale = decaying waves");
}