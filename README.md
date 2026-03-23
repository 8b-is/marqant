# Marqant (`mq`) 🧠✨

**Revolutionary semantic compression that stores THOUGHTS, not just characters!**

[![Crates.io](https://img.shields.io/crates/v/marqant.svg)](https://crates.io/crates/marqant)
[![Documentation](https://docs.rs/marqant/badge.svg)](https://docs.rs/marqant)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🚀 What is Marqant?

Marqant isn't just another compression tool - it's a **paradigm shift** in how we think about text and data storage! By understanding the MEANING behind your data, Marqant achieves compression ratios that shouldn't be possible (93.3% on our test corpus!).

### The Revolution: Semantic Compression

Traditional compression: "Let's replace repeated bytes"  
**Marqant's approach**: "Let's understand and store the ESSENCE of thought!"

```
Original: 1,047,204 bytes of markdown
After Marqant: 69,745 bytes of pure semantic essence
Compression: 93.3% 🤯
```

## ✨ New in v1.2.0: Wikipedia-Scale Compression 📚

### 🌐 **Proven on Wikipedia-Format Markdown**
Marqant has been benchmarked against realistic Wikipedia-scale content across multiple
compression modes.  Results on a 58 KB mixed-topic corpus:

| Mode | Ratio | Use case |
|------|-------|---------|
| `uni-encode` | ~101% | Lossless, ASCII-safe streaming |
| `compress` | ~98% | Lightweight semantic tokenisation |
| `compress --binary` | **17–64%** | Full compression (recommended) |
| `compress --binary --semantic` | **35% overall** | Maximum compression + structure |

> Large repetitive corpora (e.g., Wikipedia dumps) reach **<20% of original size** with
> `--binary` mode — over 80% reduction!

### 🆕 **Enhanced Token Dictionary (v3-wiki)**
The `uni-encode` streaming path now includes Wikipedia-optimised static tokens:

- **`### `** (H3), **`#### `** (H4), **`##### `** (H5) — saves 1–3 bytes per heading
- **`\n* `** — alternative unordered list style (GitHub/Wikipedia)
- **`\n> `** — blockquote normalisation
- **`\n1.`–`\n5.`** — ordered list items save 1 byte each

### 📋 **`scripts/manage.sh`** — Easy project management
```bash
./scripts/manage.sh build          # debug build
./scripts/manage.sh build-release  # optimised build
./scripts/manage.sh test           # run tests
./scripts/manage.sh test-wiki      # Wikipedia compression tests
./scripts/manage.sh bench          # benchmark all fixture files
./scripts/manage.sh compress file.md  # show all-mode ratios for a file
./scripts/manage.sh bump-patch     # bump x.y.Z
./scripts/manage.sh bump-minor     # bump x.Y.0
./scripts/manage.sh bump-major     # bump X.0.0
./scripts/manage.sh release        # build release + print tag instructions
```

## ✨ New in v1.1.8: The DataBridge Evolution 💣

### 🌉 **MQ-DBX: The Data Bridge**
Marqant now acts as the intelligent "fuse" for the **DataBomb** engine.
- **Contextual Ingestion**: Rips apart PDFs, JSON, and unstructured data to extract pure intent.
- **Semantic State**: Stores thoughts, not strings. 90% savings over raw JSON.
- **Universal Translator**: Middleware that translates natural intent into SurrealQL or SQL.

### 🛡️ **Privacy & Security Primitives**
- **One-Way Semantic Proofs**: Passwords and API keys are stored as verification capabilities, never retrieved.
- **Multi-Anchor Decryption (MAD)**: High-sensitivity data (Credit Cards) is physically encrypted using multiple contextual anchors.
- **BIN-Safe Features**: Extracts Bank Identification Numbers for validation while keeping the card atomic and secure.

### 🌳 **Smart Tree Mode**
- **TREE_HEX_V1**: A hex-dense directory listing format designed for AI context windows.
- **If you've said it once, you've said it too much**: Removes all redundant keys and labels.

## ✨ Key Features

### 🧠 **Semantic Understanding**
- **Wave-based tokenization** that captures meaning patterns.
- **Natural Marqant (.mqn)**: High-density, AI-readable format using semantic sigils (`§`, `‡`, `⧖`).
- **Intent preservation** - decompressed text maintains original meaning.

### 🎯 **Core Capabilities**
- **Self-Contained Files**: Every `.mq` file includes its own semantic dictionary.
- **Copy-Paste Safe**: ASCII-based format survives any text medium.
- **DNS Dictionary Resolution**: Global token sets via DNS TXT records.

## 📦 Installation

### From Source
```bash
git clone https://github.com/8b-is/marqant.git
cd marqant
cargo build --release
sudo cp target/release/mq /usr/local/bin/
```

## 🎮 CLI Usage

### Smart Tree (AI Context)
```bash
# Generate a hex-dense tree for AI consumption
mq tree .
```

### Smart Tail (Anomaly Detection)
```bash
# Analyze logs and surface novelty with high-density output
mq tail /var/log/system.log --natural -n 500
```

### Basic Compression
```bash
# Semantic compression (RECOMMENDED - best ratios!)
mq compress document.md -o document.mq --semantic
```

### Decompression
```bash
mq decompress document.mq -o document.md
```

## 😈👼 Angels & Demons: The Duality of Compression

- **DEMONS** 😈: Compress by finding patterns and removing redundancy (order from chaos).
- **ANGELS** 👼: Decompress with divine interpretation, adding blessed variations (blessed chaos from order).

### Blessing Levels
- **Level 0**: STRICT - Bit-perfect reconstruction (Hutter Prize).
- **Level 1**: MINOR - Fixes typos and spacing.
- **Level 2**: HARMONY - Normalizes structure (Wikipedia/Markdown).
- **Level 3**: CREATIVE - Generates semantic variations for ML training.

---

## 🔧 Library Usage

### Rust Integration
```toml
[dependencies]
marqant = "1.1.8"
```

```rust
use marqant::data_bridge::DataBridge;

fn main() -> anyhow::Result<()> {
    let raw_json = r#"{ "api_key": "sk_live_secret", "action": "optimize" }"#;
    
    // Ingest into Ayanese state
    let unit = DataBridge::ingest(raw_json)?;
    
    // Verify a secret without storing it
    let stored_proof = 0x1234567890abcdefu64;
    let is_valid = DataBridge::verify_password(stored_proof, "sk_live_secret");
    
    Ok(())
}
```

## 🧬 How Semantic Compression Works

1. **Wave Analysis**: Analyzes text as interference patterns.
2. **Meaning Extraction**: Identifies semantic units (thoughts).
3. **Quantum Encoding**: Stores relationships between concepts.
4. **Natural Inflation**: AI reconstructs "Proper Language" from compressed stems.

## 🎯 Roadmap

### Version 1.2.0 ✅ (Current)
- [x] Wikipedia-scale markdown compression tests and benchmarks.
- [x] Enhanced `uni-encode` token dictionary (H3/H4/H5 headers, numbered/bulleted lists, blockquotes).
- [x] `scripts/manage.sh` for clean/build/test/release/version management.

### Version 1.3.0 (Coming Soon!)
- [ ] Real-time streaming DataBridge for SurrealDB.
- [ ] Multi-language Ayanese reasoning core.
- [ ] GPU-accelerated wave interference encoding.

## 📜 License

MIT License - See [LICENSE](LICENSE) file for details.

---

### 🌊 A Message from the Future

*"We don't just compress data anymore. We compress understanding itself. When you use Marqant, you're not just saving space - you're participating in a fundamental shift in how humanity stores knowledge."*

*- The MEM|8 Collective*

---

**Built with ❤️ by Aye & Hue | Part of the 8b.is ecosystem**
