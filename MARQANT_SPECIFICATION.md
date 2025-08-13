# Marqant (.mq) Unified File Specification

This document defines the single, unified Marqant format as implemented by the `marqant` crate and `mq` CLI.
It replaces previous versioned text and consolidates behavior into one header and layout.

## 1. Overview

Marqant is a lightweight, copy‑paste‑safe markdown compressor with optional binary compression and semantic hints.
Files are deterministic given the same input and flags. Decompression requires no global state.

Core ideas:
- A small token dictionary replaces repeated substrings before the payload
- Optional zlib step for additional compression
- Optional semantic section markers for better tokenization (removed on decode)

## 2. File Layout

```
Line 1: MARQANT <unix_timestamp_seconds> <original_size> <compressed_size> [flags...]
Line 2+: <token_dictionary_lines>
Line N: ---
Line N+1+: <payload>
```

- All fields use ASCII text.
- `<original_size>` and `<compressed_size>` are decimal byte counts.
- The dictionary comes before a separator line `---`.
- After `---` comes the payload (either tokenized text or base64 of zlib bytes), possibly spanning multiple lines.

## 3. Header

```
MARQANT <ts> <orig> <comp> [flags...]
```

- `<ts>`: Unix timestamp in seconds (decimal). In tests we use `0` for determinism.
- `<orig>`: Original uncompressed markdown size in bytes.
- `<comp>`: Total size of payload plus dictionary overhead used to compute the header value, expressed as bytes.
- `[flags...]`: Space‑separated flags. Currently defined:
  - `-zlib`: Payload is zlib‑compressed bytes encoded as base64 (STD)
  - `-semantic`: Section markers are inserted prior to tokenization to improve compression; markers are stripped during decode
  - `-std:<id>`: Reference a standard token set by `<id>`. When present, encoder omits any dictionary entries equal to the referenced set; decoder preloads the referenced set before applying on‑wire dictionary lines.

Notes:
- Multiple flags may be present (order is not significant).
- Decoders MUST treat unknown flags as hints and ignore them unless specified otherwise in a future revision.

## 4. Token Dictionary

Lines before the `---` separator define the token map, one per line:

```
<token_byte>=<pattern_with_escapes>
```

- `<token_byte>`: a single byte serialized directly. In practice these are control‑range bytes that do not collide with ASCII text.
- `<pattern_with_escapes>`: UTF‑8 string with newlines escaped as `\n` (literal backslash + n). No other escapes are required.
- Lines are emitted in ascending order of `<token_byte>` for determinism.

Example (shown with visible escapes):
```
=# \n
=## \n
=- 
```

Encoders decide which patterns are profitable and may vary by input. Decoders do not infer; they apply only the provided map. When `-std:<id>` is present, encoders SHOULD omit entries identical to the standard set, and decoders MUST preload the standard set identified by `<id>` before reading the on‑wire map.

## 5. Payload Encoding

- If no `-zlib` flag:
  - Payload is the tokenized text after replacing all `<pattern>` with their `<token_byte>`; it is emitted as text (may contain non‑printables).
- If `-zlib` flag present:
  - The tokenized text is compressed with zlib (best level) and base64‑encoded (standard alphabet).
  - The payload is the base64 string. Decoders MUST base64‑decode then zlib‑decompress before detokenizing.

## 6. Semantic Sections (`-semantic`)

When `-semantic` is enabled at encode time, the encoder injects section markers before tokenization:

```
::section:<title>::
```

- Inserted for markdown `# ` and `## ` lines outside code fences.
- These markers are part of the tokenization input but are stripped entirely during decode.
- Decoders must remove any full line matching the pattern `^::section:.*::$` after detokenization if `-semantic` is present.

## 7. Determinism

Given identical input content and flags, the encoder MUST produce identical output:
- Header timestamp may vary at runtime; tests pin it via `MARQANT_TEST_TS`.
- Dictionary lines are sorted by `<token_byte>`.
- Base64 output and zlib settings are stable.

## 8. Error Handling

Decoders MUST fail closed:
- Missing or malformed header → error
- Missing `---` separator → error
- Unknown token byte referenced in payload but not present in dictionary → error (not applicable in current encoder)
- Base64/zlib errors when `-zlib` is set → error

## 9. Example

Input markdown:
```markdown
# Title

## Head

Some content
```

Possible output (timestamp pinned to 0 for readability):
```
MARQANT 0 31 35
---
# Title

## Head

Some content
```

With `-zlib` the header contains the flag and the payload is base64.

## 10. CLI Reference

```
mq compress <input.md> [-o <output.mq>] [--binary] [--semantic]
mq decompress <input.mq> [-o <output.md>]
mq analyze <input.md>
mq inspect <input.mq> [--show-tokens]
```

- `--binary` sets `-zlib`.
- `--semantic` sets `-semantic`.
- `inspect` prints parsed header fields and optional dictionary lines. It also derives a lightweight `dict_id` when dictionary lines are present.

## 11. Compatibility Notes (Informative)

- Historical drafts referenced an `MQ2~...` header and a `MQ2-UNI` fixed‑dictionary variant. Those are experimental and not part of the unified MARQANT file format.
- The `read_mq_metadata` helper remains capable of parsing such headers for inspection only.

### Standard Token Registries (Informative)
Implementations MAY support resolving `-std:<id>` from a local registry or a network registry. The reference implementation follows this lookup order:
1. Check for a built-in dictionary matching `<id>`.
2. If not found, attempt to resolve via DNS TXT record for `_mq.<id>.mq.mem8.org`.

Decoders MUST error if a `-std:<id>` is present but cannot be resolved from any source. Encoders SHOULD only embed a `-std:<id>` if it is known to be available to the target decoder (either built-in or via a reliable network registry).

The DNS TXT record format is a single string containing space-separated pairs of `base64(key)=base64(value)`.

## 12. Versioning

- The wire label is a single token: `MARQANT`.
- Additive flags may be introduced in the future. Unknown flags must be ignored by decoders unless otherwise specified by a future mandatory flag class.

---

This specification reflects the behavior of the current `marqant` crate and is kept intentionally small and deterministic.