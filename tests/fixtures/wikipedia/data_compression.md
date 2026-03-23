# Compression (data compression)

**Data compression** is a process in which the encoding of information uses fewer bits than the
original representation. Compression can be either **lossy** or **lossless**. Lossless
compression reduces bits by identifying and eliminating statistical redundancy; no information
is lost in lossless compression. Lossy compression reduces bits by removing unnecessary or
less important information.

## Overview

Data compression is useful because it reduces resources required to store and transmit data.
The process of data compression involves transforming data from one format to another.
Compression algorithms are evaluated on:

- **Compression ratio**: How much smaller the compressed data is compared to the original
- **Compression speed**: How quickly data can be compressed
- **Decompression speed**: How quickly compressed data can be restored
- **Memory requirements**: RAM needed during compression and decompression

## Lossless Compression

### Entropy Coding

Entropy coding exploits the statistical structure of data to achieve compression without
any information loss. The most common entropy coding methods are:

#### Huffman Coding

Huffman coding assigns shorter codewords to more frequent symbols and longer codewords to
less frequent ones. The optimal prefix code for a given set of symbol probabilities can be
found in O(n log n) time.

```
Symbol  Frequency  Huffman Code
A       45%        0
B       13%        101
C       12%        100
D       16%        111
E       9%         1101
F       5%         1100
```

#### Arithmetic Coding

Arithmetic coding encodes an entire message into a single number between 0 and 1.
It can achieve compression ratios close to the theoretical entropy limit, making it
more efficient than Huffman coding for certain distributions.

#### Asymmetric Numeral Systems (ANS)

ANS is a modern entropy coding method that combines the simplicity of Huffman coding
with the efficiency of arithmetic coding. It is used in Facebook's Zstandard (zstd)
and Apple's LZFSE compressors.

### Dictionary Compression

Dictionary compression works by replacing repeated occurrences of patterns with references
to a shared dictionary.

#### LZ77 and LZ78

The LZ family of algorithms forms the foundation of most modern lossless compressors:

1. **LZ77**: Uses a sliding window to find matches in previously seen data
2. **LZ78**: Builds an explicit dictionary as it processes data
3. **LZW**: A variant of LZ78 used in GIF and TIFF formats
4. **LZMA**: Used in 7-Zip and XZ utilities, achieving excellent compression ratios
5. **LZ4**: Optimized for extremely fast compression and decompression

#### Deflate

Deflate combines LZ77 with Huffman coding and is the compression algorithm used in:

- gzip (`.gz` files)
- zlib library
- PNG image format
- ZIP archives

### Run-Length Encoding

Run-length encoding (RLE) replaces sequences of the same value with a count and the value:

> Original: AAAABBBCCCDDDDDD  
> RLE: 4A3B3C6D

RLE is highly effective for data with many consecutive repeated values, such as simple
bitmap images with large areas of solid color.

### Prediction and Context Modelling

Advanced lossless compressors use predictive modelling to achieve compression ratios beyond
simple dictionary methods:

- **PPM (Prediction by Partial Matching)**: Uses a variable-order Markov model
- **PAQ**: A family of compressors using neural networks for context mixing
- **BZIP2**: Uses the Burrows-Wheeler Transform followed by Huffman coding

## Lossy Compression

### Image Compression

#### JPEG

JPEG compression operates in several stages:

1. Color space conversion (RGB to YCbCr)
2. Downsampling of chrominance channels
3. Division into 8×8 pixel blocks
4. Discrete Cosine Transform (DCT) of each block
5. Quantization of DCT coefficients
6. Entropy coding (Huffman or arithmetic)

The quantization step introduces information loss, and the quality factor controls the
trade-off between file size and image quality.

#### WebP

WebP is a modern image format developed by Google that supports both lossy and lossless
compression. For lossy compression, WebP uses:

- Predictive coding for image blocks
- Discrete cosine transform for residuals
- Arithmetic entropy coding

WebP achieves 25-34% better compression than JPEG at equivalent quality.

#### AVIF

AVIF (AV1 Image File Format) is based on the AV1 video codec and offers superior
compression to both JPEG and WebP for many types of images.

### Audio Compression

#### MP3

MP3 (MPEG-1 Audio Layer III) uses psychoacoustic models to identify and remove audio
information that the human ear cannot perceive. The algorithm:

1. Splits audio into frequency sub-bands using polyphase filter banks
2. Applies modified discrete cosine transform (MDCT)
3. Quantizes coefficients based on psychoacoustic masking
4. Encodes with Huffman coding

#### AAC

Advanced Audio Coding (AAC) is the successor to MP3 and is the default audio format for
YouTube, iPhone, and many streaming services. AAC achieves better audio quality than MP3
at the same bit rate.

#### Opus

Opus is a modern codec developed by the IETF that supports a wide range of bitrates and
is used in WebRTC, Discord, and many other applications.

### Video Compression

Video compression exploits both spatial redundancy (within a frame) and temporal redundancy
(between frames):

| Codec | Standard | Typical Use |
|-------|----------|-------------|
| H.264/AVC | MPEG-4 Part 10 | Streaming, Blu-ray, broadcast |
| H.265/HEVC | MPEG-H Part 2 | 4K streaming, broadcast |
| VP9 | Google/WebM | YouTube, Chrome |
| AV1 | Alliance for Open Media | Streaming, Netflix |
| H.266/VVC | MPEG-I Part 3 | Next-gen streaming |

Key video compression concepts include:

- **I-frames**: Complete intra-coded frames (no reference to other frames)
- **P-frames**: Predicted frames using motion vectors from previous frames
- **B-frames**: Bidirectional predicted frames using both past and future frames
- **Motion compensation**: Describing how blocks of pixels move between frames

## Semantic Compression

Semantic compression goes beyond traditional bit-level approaches by understanding the
*meaning* of data rather than just its statistical structure. This approach is particularly
relevant for:

### Text and Document Compression

Traditional text compressors treat documents as byte streams. Semantic compressors can:

- Extract key concepts and discard redundant elaboration
- Represent relationships between ideas rather than verbatim text
- Use domain-specific knowledge to achieve higher compression ratios
- Reconstruct "equivalent" text rather than exact reproductions

### AI-Driven Compression

Large language models and other AI systems enable new forms of compression:

1. **Neural image compression**: Models learn optimal transforms and entropy codes
2. **Semantic video compression**: Represents scenes as parametric models
3. **Text tokenization**: Subword tokenization achieves efficient representation
4. **Knowledge distillation**: Compresses model knowledge into smaller representations

## Compression Benchmarks

Compression algorithms are typically evaluated on standard benchmarks:

### Canterbury Corpus

The Canterbury Corpus is a standard benchmark for lossless data compression, consisting of
11 files representing different types of data:

1. alice29.txt — English text from Alice in Wonderland
2. asyoulik.txt — Shakespeare play
3. cp.html — HTML source
4. fields.c — C source code
5. grammar.lsp — Lisp source
6. kennedy.xls — Excel spreadsheet
7. lcet10.txt — Technical writing
8. plrabn12.txt — Poetry
9. ptt5 — Fax transmission data
10. sum — SPARC executable
11. xargs.1 — Unix man page

### Silesia Corpus

The Silesia Corpus is a larger benchmark consisting of 12 files totaling about 200 MB.
It tests compressors on a diverse range of real-world data types.

## See Also

- Information theory
- Data deduplication
- Delta encoding
- Error correction codes

## References

1. Ziv, J., & Lempel, A. (1977). A universal algorithm for sequential data compression.
   *IEEE Transactions on Information Theory*, 23(3), 337-343.
2. Huffman, D. A. (1952). A method for the construction of minimum-redundancy codes.
   *Proceedings of the IRE*, 40(9), 1098-1101.
3. Salomon, D. (2007). *Data Compression: The Complete Reference* (4th ed.). Springer.
4. Blelloch, G. E. (2001). Introduction to data compression. Computer Science Department,
   Carnegie Mellon University.
