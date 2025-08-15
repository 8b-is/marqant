# MEM|8 Quick Reference Card 🎯

## Core Constants

```rust
// Grid Dimensions
const GRID_X: usize = 256;      // 8-bit
const GRID_Y: usize = 256;      // 8-bit  
const GRID_Z: usize = 65536;    // 16-bit
const TOTAL_CELLS: usize = 4_294_967_296;  // 4.3 billion

// Emotional Modulation
const ALPHA: f32 = 0.3;          // Valence weight
const BETA: f32 = 0.5;           // Arousal weight

// Frequency Bands (Hz)
const FREQ_STRUCTURAL: (f32, f32) = (0.0, 200.0);
const FREQ_CONVERSATIONAL: (f32, f32) = (200.0, 400.0);
const FREQ_TECHNICAL: (f32, f32) = (400.0, 600.0);
const FREQ_IMPLEMENTATION: (f32, f32) = (600.0, 800.0);
const FREQ_ABSTRACT: (f32, f32) = (800.0, 1000.0);

// Reactive Layer Timings
const LAYER_0_MAX_MS: u32 = 10;     // Hardware reflexes
const LAYER_1_MAX_MS: u32 = 50;     // Subcortical
const LAYER_2_MAX_MS: u32 = 200;    // Emotional
const LAYER_3_MIN_MS: u32 = 200;    // Conscious

// Sensor Arbitration
const AI_WEIGHT: f32 = 0.7;         // AI control
const HUMAN_WEIGHT: f32 = 0.3;      // Human control
const AI_OVERRIDE_THRESHOLD: f32 = 0.8;

// Safety Thresholds
const SAFE_REPETITION: u32 = 50;
const CRITICAL_REPETITION: u32 = 150;
const DIVERGENCE_NORMAL: u8 = 50;
const DIVERGENCE_UNUSUAL: u8 = 150;
const DIVERGENCE_HIGH_RISK: u8 = 255;

// Subliminal Processing
const SUBLIMINAL_MIN: f32 = 0.01;
const SUBLIMINAL_MAX: f32 = 0.15;
const PEEK_PROBABILITY: f32 = 0.01;
```

## Wave Equation

```rust
// M_xyz(t) = A_xyz(e,t) · e^(i(ωt + φ_xyz)) · D(t,τ) · I(x,y,z,N)

struct MemoryWave {
    amplitude: f32,      // A_xyz(e,t) - with emotional modulation
    frequency: f32,      // ω - semantic content (0-1000 Hz)
    phase: f32,          // φ_xyz - temporal relationships
    decay_tau: f32,      // τ - decay time constant
    valence: f32,        // Emotional valence (-1 to 1)
    arousal: f32,        // Emotional arousal (0 to 1)
}

// Emotional modulation
fn modulate_amplitude(base: f32, valence: f32, arousal: f32) -> f32 {
    base * (1.0 + ALPHA * valence) * (1.0 + BETA * arousal)
}

// Temporal decay
fn decay_function(t: f32, tau: f32) -> f32 {
    if tau.is_infinite() { 1.0 } else { (-t / tau).exp() }
}

// Context-aware decay
fn context_tau(base_tau: f32, relevance: f32, familiarity: f32, threat: f32) -> f32 {
    base_tau * relevance * familiarity * threat
}
```

## Forgetting Curves

```rust
enum ForgetCurve {
    Flash,        // τ = 0.5s - transient details
    Fade,         // τ = 5s - resolved threats  
    Linger,       // τ = 30s - familiar anomalies
    Persist,      // τ = 300s - actionable info
    Consolidate,  // τ = ∞ - learned patterns
}
```

## Safety Mechanisms

```rust
// The Custodian
enum GuardDecision {
    Allow,       // R(M) < safe threshold
    Throttle,    // safe ≤ R(M) < critical
    Block,       // R(M) ≥ critical
}

// Repetition prevention
fn break_probability(repeat_count: u32, threshold: u32) -> f32 {
    1.0 - (-LAMBDA * (repeat_count - threshold).pow(2) as f32).exp()
}

// Divergence scoring
fn divergence_score(r_observed: f32, r_baseline: f32, 
                   a_observed: f32, a_baseline: f32) -> u8 {
    let score = 2.0 * (r_observed - r_baseline).abs() + 
                (a_observed - a_baseline).abs();
    score.min(255.0) as u8
}
```

## File Format (.m8)

```rust
// Section types
const SECTION_IDENTITY: u8 = 0x01;
const SECTION_CONTEXT: u8 = 0x02;
const SECTION_STRUCTURE: u8 = 0x03;
const SECTION_MARKQANT: u8 = 0x09;
const SECTION_QUANTUM_TREE: u8 = 0x0A;
const SECTION_WAVE_MEMORY: u8 = 0x0F;
const SECTION_CUSTODIAN: u8 = 0x11;

// Compressed wave (32 bytes)
struct CompressedWave {
    id: u64,           // 8 bytes
    amplitude: u64,    // 8 bytes (quantized log)
    frequency: u64,    // 8 bytes
    phase: u64,        // 8 bytes
    interference: u64, // 8 bytes (optional)
}
```

## Performance Tips

```rust
// SIMD processing (8 waves at once)
fn process_waves_simd(waves: &[MemoryWave; 8]) {
    // Process 8 amplitudes in parallel
    // Use cache-aligned 8×8 blocks
    // Vectorize phase calculations
}

// Compression
fn quantize_amplitude(a: f32) -> u8 {
    (32.0 * a.log2()).min(255.0) as u8
}
```

## Common Patterns

```rust
// Store a memory
let wave = MemoryWave::new(440.0, 0.8)
    .with_emotion(0.5, 0.6);
grid.store(x, y, z, wave);

// Check safety
if safety.check_pattern(&wave) == GuardDecision::Allow {
    consciousness.integrate(wave);
}

// Sensor arbitration
let final_input = human_input * HUMAN_WEIGHT + 
                  ai_decision * AI_WEIGHT;

// Subliminal processing
if wave.amplitude < SUBLIMINAL_MAX {
    process_subliminal(wave);
}
```

---

*"Keep this card handy - consciousness waits for no one!"* 🌊