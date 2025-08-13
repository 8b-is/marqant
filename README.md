# Marqant (`mq`)

**Quantum-compressed markdown with standard & custom token sets.**

Marqant is a lightweight, copy-paste-safe markdown compressor that uses a combination of dynamic and standard token sets to achieve high compression ratios while maintaining readability and performance. It's designed to be simple, fast, and extensible.

## What is Marqant?

At its core, Marqant replaces frequently-occurring substrings in a document with single-byte tokens. This dictionary is prepended to the compressed file, making each file self-contained and decodable without any external context.

Key concepts:
- **Self-Contained**: Every `.mq` file includes its own dictionary.
- **Text-Safe**: The default format uses only printable ASCII characters.
- **Extensible**: Supports standard token sets via local or network-based registries.
- **Performant**: Written in Rust for speed and safety.

## Features

- **Dynamic Tokenization**: Automatically identifies and tokenizes the most frequent phrases in a document.
- **Standard Dictionaries**: Use shared, standard token sets for common patterns (e.g., markdown syntax) to reduce file size. Reference them with a simple `-std:<id>` flag.
- **DNS-based Dictionary Resolution**: For globally shared dictionaries, Marqant can resolve token sets from DNS TXT records, allowing for centralized management without embedding them in the client.
- **Optional Zlib Compression**: For maximum compression, enable the `-zlib` flag to compress the tokenized payload.
- **Semantic Hints**: The `-semantic` flag injects section markers based on markdown headers (`#`, `##`) to improve tokenization locality, which are then stripped upon decompression.

## CLI Usage

The `mq` command-line tool provides a simple interface for all Marqant operations.

### Compress a file

```bash
# Basic compression
mq compress document.md -o document.mq

# With zlib and a standard dictionary
mq compress document.md -o document.mq --binary --std std-static-v1
```

### Decompress a file

Decompression is simple and automatically handles all flags from the file header.

```bash
mq decompress document.mq -o document.md
```

### Inspect a file

You can view a `.mq` file's metadata without decompressing it.

```bash
mq inspect document.mq

# Show the token dictionary as well
mq inspect document.mq --show-tokens
```

For more commands, run `mq --help`.

## Library Usage

You can use `marqant` as a library in your own Rust projects.

Add it to your `Cargo.toml`:
```toml
[dependencies]
marqant = { path = "path/to/marqant" } # Or from a registry when published
```

### Example

```rust
use marqant::Marqant;

fn main() -> anyhow::Result<()> {
    let markdown = "# My Document\n\nThis is a test of the Marqant compression system.\n";

    // Compress with a standard dictionary
    let flags = Some("-std:std-static-v1");
    let compressed = Marqant::compress_markdown_with_flags(markdown, flags)?;

    println!("Compressed:\n{}", compressed);

    // Decompress
    let decompressed = Marqant::decompress_marqant(&compressed)?;

    println!("Decompressed:\n{}", decompressed);

    assert_eq!(markdown.trim(), decompressed.trim());

    Ok(())
}
```

## Building from Source

1.  Clone the repository.
2.  Install the Rust toolchain (if you haven't already).
3.  Build the project:
    ```bash
    cargo build --release
    ```
4.  The binary will be at `target/release/mq`.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
