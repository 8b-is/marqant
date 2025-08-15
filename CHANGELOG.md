# Changelog


## [0.2.0] - 2025-08-15

### Changed
- Version bump
## [0.1.2] - 2024-08-14

### Fixed
- **CRITICAL**: Fixed UTF-8/emoji corruption by using ASCII escape sequences instead of high bytes
- Removed all byte substitutions >= 0x80 that conflicted with UTF-8 continuation bytes
- Added UTF-8 boundary detection to never split multi-byte characters
- All emojis and Unicode now preserved perfectly during encode/decode

### Added
- Comprehensive roundtrip tests for all markdown files
- Explicit byte-level emoji preservation tests
- UTF-8 boundary skipper for safe character handling

### Changed
- Token format now uses ASCII escape sequences (e.g., `~H1` for headers)
- Dictionary ID updated to "mq2-uni-v2-utf8safe"

## [0.1.1] - 2024-08-14

### Added
- Module documentation for docs.rs
- Semantic compression prototype (experimental)
- Novelty tracking system

### Fixed
- Categories updated to valid crates.io slugs

## [0.1.0] - 2024-08-14

### Initial Release
- Basic marqant compression for markdown
- CLI tool for encoding/decoding
- DNS integration for token maps