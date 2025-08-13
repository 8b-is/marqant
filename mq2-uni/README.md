MQ2-UNI: Universal, AI-guided, fixed-dictionary tokenizer

Scope
- Fixed, deterministic dictionary (~T/~S) curated offline; tiny encoder/decoder.
- Language-indifferent (UTF-8 bytes/graphemes), AI-guided but AI-free at runtime.

Wire format
- Header: `MQ2~UNI~<ts_hex>~<orig_hex>~<comp_hex>~<tokc_hex>~text`\n
- `~T` token map section: `~T <tok><len:u16_be><bytes> ...` then `\n~~~~\n`
- Stream: token bytes; passthrough for non-tokenized.

Determinism
- Sort token entries by token id in `~T`.
- Timestamp can be fixed via env or test clock.

DNSSEC Token Distribution (cache-everywhere)
- Dictionary ID: `dict_id = blake3(~T||~S)[:128]` (128-bit hex).
- Name: `dict.<dict-id-hex>.<root>` (e.g., `dict.7f1a…c9.mq.8a.is`).
- Records (DNSSEC-signed):
  - SVCB/HTTPS: canonical HTTPS endpoints (CDN/IPFS/MEMNET gateway)
  - TXT: `ver`, `created`, `tokc`, `dict_id`, optional hints
  - (Optional) CAA: publisher pinning
- Chunked token maps via labels: `t0.dict.<id>…`, `t1.dict.<id>…` carry Base64url shards (≤1200B)
- Reverse PTR: `…ip6.arpa.` maps 128-bit `dict_id` → canonical name for discovery
- Token info: `tok.<tok-hex>.dict.<id>…` TXT includes len, class, and `sem128`

SEM128 (token “meaning stamp”)
Two options (both 128-bit):
1) Packed fields (interpretable)
   - [8b class][8b modality][8b domain][8b flags]
   - [16b valence][16b arousal][16b dominance]
   - [24b time_band][24b cluster_id][8b ver]
   - Fixed-point in [-1,1] for V/A/D; class/modality/domain enums
2) Quantized vector (portable)
   - 16× int8 (signed) + 1-byte scale table index (drop 1 bit via fixed table) ≈ 128b

Security & Ops
- DNSSEC required; DoQ/DoH recommended for privacy. Pin `dict_id` with TLS origin.
- Rotation: TXT `replaces=<new-id>` with grace TTL; mirrors via AXFR.

Reference impl
- `encode`/`decode` in `src/lib.rs` (no deps). Greedy longest-match, passthrough otherwise.
- Optional resolver stub lives in `mq2-uni-resolver` (feature-gated DNS).


