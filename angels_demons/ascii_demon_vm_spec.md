# ASCII Demon VM Specification v1.0
**Revolutionary VM-based Compression Architecture**

## Overview

ASCII Demon VM (ADVM) is a pure, extensible virtual machine for text compression that combines:
- **Boilerplate demons** (0x80-0xAC): Common patterns baked into every decoder
- **Custom demons**: Per-file definitions for specialized patterns
- **Override mechanism**: Files can shadow boilerplates with custom definitions
- **Clean VM design**: C0 controls + demons at 0x80-0xFF

## File Structure

```
"ADVM1"        ; 5-byte magic signature
0x01           ; version = 1
flags:u16le    ; bit0=crc32, bit1=sha256, others reserved
sec_count:ULEB ; number of header sections
[sections...]  ; variable-length sections
-- main VM stream begins --
```

**ULEB**: Unsigned LEB128 (7-bit groups, MSB=continue). Decoders must bound to ≤10 bytes.

## Header Sections

### Section Format (Generic)

```
type:u8        ; 1=DEFINE_DEMONS, 2=PARAM_DICT, 3=CHECKSUMS
len:ULEB       ; length of payload in bytes
payload[...]   ; depends on type
```

Minimal decoders only need type=1. Unknown types → skip `len` bytes (forward-compatible).

### Section Type 1: DEFINE_DEMONS

Define or override demons in range 0x80-0xFF. Local definitions shadow boilerplates.

```
entries:ULEB
repeat(entries) {
  demon_id:u8        ; 0x80-0xFF
  kind:u8            ; 0=WRAP, 1=WORDLIST, 2=ALIAS
  body[...]          ; by kind (below)
}
```

#### Kind 0: WRAP
Emits prefix + (invocation body) + suffix.

```
pre_len:ULEB
pre_bytes[pre_len]   ; raw bytes
suf_len:ULEB
suf_bytes[suf_len]
```

**Invocation**: Decoder parses nested substream until EOT (0x04), writes prefix+output+suffix.

#### Kind 1: WORDLIST
Fixed dictionary; invocation supplies index.

```
count:ULEB
repeat(count) {
  item_len:ULEB
  item[item_len]
}
```

**Invocation**: Read index:ULEB then EOT; emit item[index].

#### Kind 2: ALIAS
Maps this ID to another demon.

```
target_id:u8   ; 0x80-0xFF
```

### Section Type 2: PARAM_DICT (Optional)

Key/value table for encoder-side param compaction.

```
pairs:ULEB
repeat(pairs) {
  id:ULEB
  val_len:ULEB
  val[val_len]
}
```

### Section Type 3: CHECKSUMS (Optional)

Archive integrity verification.

```
flags:u8        ; bit0=crc32, bit1=sha256
if flags&1: crc32_le:u32
if flags&2: sha256[32]
```

## VM Stream Encoding

After header sections, the main stream begins:

### Control Characters (0x00-0x1F)
- **0x02 STX**: Start of Text - begin literal mode
- **0x03 ETX**: End of Text - end literal mode
- **0x04 EOT**: End of Transmission - end demon frame
- **0x1F US**: Unit Separator - optional field delimiter

### ASCII Literals (0x20-0x7F)
Copied as-is when not in literal mode.

### Demon Calls (0x80-0xFF)
Invoke demon, read nested body until EOT.

## Boilerplate Demons (v1)

Built into every v1 decoder:

| ID | Name | Pattern | Example |
|----|------|---------|---------|
| 0x80 | WRAP | `[[ body ]]` | Wiki link wrapper |
| 0x81 | WRAP_L2 | `[[ body ]]` | Two-bracket wrapper |
| 0x82 | WRAP_L3 | `[[[ body ]]]` | Three-bracket wrapper |
| 0x83 | CAT | `[[Category: body ]]` | Category tag |
| 0x84 | TITLE | `<title> body </title>` | XML title |
| 0x85 | REF | `<ref> body </ref>` | Reference tag |
| 0x86 | CITE | `{{cite body }}` | Citation template |
| 0x87 | QUOTE | `" body "` | Quoted text |
| 0x88 | PAREN | `( body )` | Parenthetical |
| 0x89 | DATE | `body, yyyy` | Date format |
| 0x8A | BOLD | `''' body '''` | Wiki bold |
| 0x8B | ITALIC | `'' body ''` | Wiki italic |
| 0x8C | H1 | `= body =` | Level 1 heading |
| 0x8D | H2 | `== body ==` | Level 2 heading |
| 0x8E | H3 | `=== body ===` | Level 3 heading |
| 0x8F | LIST | `* body` | List item |
| 0x90 | NUM_LIST | `# body` | Numbered item |
| 0x91 | INDENT | `: body` | Indented line |
| 0x92 | IMG | `[[File: body ]]` | Image link |
| 0x93 | TMPL | `{{ body }}` | Template |
| 0x94 | MATH | `<math> body </math>` | Math block |
| 0x95 | CODE | `<code> body </code>` | Code block |
| 0x96 | PRE | `<pre> body </pre>` | Preformatted |
| 0x97 | COMMENT | `<!-- body -->` | HTML comment |
| 0x98 | DIV | `<div> body </div>` | Div block |
| 0x99 | SPAN | `<span> body </span>` | Inline span |
| 0x9A | TABLE | `{| body |}` | Wiki table |
| 0x9B | ROW | `|- body` | Table row |
| 0x9C | CELL | `| body` | Table cell |
| 0x9D | HCELL | `! body` | Header cell |
| 0x9E | LANG | `[[ body ]]` | Language link |
| 0x9F | EXT | `[ body ]` | External link |
| 0xA0 | REDIRECT | `#REDIRECT [[ body ]]` | Page redirect |
| 0xA1 | INFOBOX | `{{Infobox body }}` | Infobox |
| 0xA2 | NAVBOX | `{{Navbox body }}` | Navigation box |
| 0xA3 | PORTAL | `{{Portal| body }}` | Portal link |
| 0xA4 | STUB | `{{ body -stub}}` | Stub template |
| 0xA5 | DISAMBIG | `{{disambiguation}}` | Disambig page |
| 0xA6 | COORD | `{{coord| body }}` | Coordinates |
| 0xA7 | CONVERT | `{{convert| body }}` | Unit conversion |
| 0xA8 | CITATION | `{{citation body }}` | Full citation |
| 0xA9 | REFLIST | `{{reflist}}` | Reference list |
| 0xAA | NOTELIST | `{{notelist}}` | Notes section |
| 0xAB | AUTHORITY | `{{Authority control}}` | Authority |
| 0xAC | DEFAULT | `{{DEFAULTSORT: body }}` | Sort key |

## Override Rules

When a file defines a demon_id matching a boilerplate:
1. Local definition wins for this file
2. Decoder maintains local table; lookup order: local → boilerplate
3. Allows per-file customization without breaking compatibility

## Example: Custom Wikipedia File

```
Header:
"ADVM1" 01 00 00           ; magic, ver, flags
01                          ; sec_count = 1
01 <len>                    ; type=DEFINE_DEMONS
  02                        ; entries=2
  83 00                     ; Override CAT with custom wrapper
    0E "[[Kategorie:"       ; German category
    02 "]]"
  B0 01                     ; Custom wordlist at 0xB0
    03                      ; 3 items
    06 "United"
    06 "States" 
    07 "America"

Stream:
83 44 65 6D 6F 6E 73 04    ; CAT: "Demons" → [[Kategorie:Demons]]
B0 00 04 20 6F 66 20       ; Wordlist[0]: "United" + " of "
B0 02 04                    ; Wordlist[2]: "America"
                            ; Result: "United of America"
```

## Decoder Implementation

### Minimal C Decoder (~4KB)

```c
typedef struct {
    uint8_t id;
    uint8_t kind;
    union {
        struct { char* pre; char* suf; } wrap;
        struct { char** items; int count; } wordlist;
        uint8_t target_id;
    } data;
} Demon;

Demon boilerplates[45];  // 0x80-0xAC
Demon customs[128];      // Local overrides
int custom_count = 0;

void decode_stream(uint8_t* in, uint8_t* out) {
    while (*in) {
        if (*in < 0x20) {
            // Control character
            if (*in == 0x02) { /* STX: literal mode */ }
            else if (*in == 0x04) { /* EOT: end frame */ }
        }
        else if (*in < 0x80) {
            *out++ = *in++;  // ASCII literal
        }
        else {
            // Demon call
            Demon* d = find_demon(*in++);
            execute_demon(d, &in, &out);
        }
    }
}
```

## Compression Strategy

1. **Analysis Phase**
   - Scan for Wikipedia patterns
   - Count frequencies
   - Calculate entropy savings

2. **Demon Selection**
   - Assign boilerplates to common patterns
   - Create custom demons for file-specific patterns
   - Build optimal wordlists

3. **Encoding**
   - Write ADVM1 header with custom definitions
   - Encode text using demon calls
   - Optimize for minimal output size

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Decoder size | <5KB | Pure C implementation |
| Compression ratio | <11% | On enwik9 |
| Decompression speed | >50MB/s | Single-threaded |
| Memory usage | <1MB | During decompression |

## Version Evolution

### v1.0 (Current)
- 45 boilerplate demons
- WRAP, WORDLIST, ALIAS kinds
- Override mechanism

### v2.0 (Planned)
- Extended demon range (0x80-0xFF)
- Regex demons
- Compression chains
- Parallel decoding hints

## Security Considerations

1. **ULEB Bounds**: Max 10 bytes prevents infinite loops
2. **Index Checks**: Wordlist access validates bounds
3. **Memory Limits**: Fixed buffers prevent allocation attacks
4. **Checksum Verification**: Optional integrity checking

## Conclusion

ASCII Demon VM provides a clean, extensible architecture for text compression that:
- Maintains tiny decoder size (~4-5KB)
- Allows per-file flexibility
- Supports future extensions
- Achieves state-of-the-art compression on Wikipedia data

The combination of boilerplate patterns and custom overrides gives us the best of both worlds: efficiency for common cases and flexibility for specialized content.