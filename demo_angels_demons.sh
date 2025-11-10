#!/bin/bash
# Angels & Demons Demo Script
# Demonstrates the complete compression/decompression cycle with blessing levels

set -e

echo "🔥👼😈🔥 ANGELS & DEMONS DEMONSTRATION 🔥😈👼🔥"
echo ""
echo "Thermodynamic Poetry:"
echo "  Demons sort the chaos, reducing entropy's reign"
echo "  Angels bless the output, adding variance again"
echo "  Together they create a cycle, neither good nor bad"
echo "  Just information dancing, making Maxwell glad"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create test content with intentional issues for blessings to fix
cat > /tmp/test_angels_demons.md << 'EOF'
# Wave-Based Memory Systems

The wave memory system uses wave interference patterns. This  has  double  spaces
and teh recieve are common typos that need fixing.

## Wikipedia Example

Here is some [[category:test]] content that needs harmonization.
Also [[CATEGORY:computers]] should be fixed.

### Technical Details

The wave-based architecture represents a fundamental shift  in  how  we  think
about memory and consciousness  . Notice the space before period .

- Bullet  points need fixing
- Another item  here

## Code Example

```python
def compress(data):
    # This is a simple compression example
    return data.encode()
```

The system is a distributed file system and also implements parallelism.
EOF

echo "📝 Created test file with intentional issues:"
echo "   - Double spaces"
echo "   - Typos (teh, recieve)"
echo "   - Wikipedia category capitalization"
echo "   - Space before punctuation"
echo ""

# Build the binaries
echo "🔨 Building binaries..."
cargo build --release --quiet
echo "   ✓ Build complete"
echo ""

DEMON="./target/release/demon_compressor"
ANGEL="./target/release/angel_decompressor"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 1: DEMON COMPRESSION 😈"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

$DEMON /tmp/test_angels_demons.md /tmp/compressed.mq

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 2: ANGEL DECOMPRESSION (All Levels) 👼"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Level 0: Strict
echo "┌─────────────────────────────────────────────┐"
echo "│ LEVEL 0: STRICT (Bit-perfect)              │"
echo "└─────────────────────────────────────────────┘"
echo ""
$ANGEL /tmp/compressed.mq /tmp/output_level0.md 0

# Level 1: Minor Blessings
echo ""
echo "┌─────────────────────────────────────────────┐"
echo "│ LEVEL 1: MINOR BLESSINGS (Fix typos)       │"
echo "└─────────────────────────────────────────────┘"
echo ""
$ANGEL /tmp/compressed.mq /tmp/output_level1.md 1

# Level 2: Harmony
echo ""
echo "┌─────────────────────────────────────────────┐"
echo "│ LEVEL 2: HARMONY (Wikipedia fixes)         │"
echo "└─────────────────────────────────────────────┘"
echo ""
$ANGEL /tmp/compressed.mq /tmp/output_level2.md 2

# Level 3: Creative
echo ""
echo "┌─────────────────────────────────────────────┐"
echo "│ LEVEL 3: CREATIVE (Training variations)    │"
echo "└─────────────────────────────────────────────┘"
echo ""
$ANGEL /tmp/compressed.mq /tmp/output_level3.md 3

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 3: COMPARISON OF OUTPUTS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Checking for typo fixes..."
echo "  Original has 'teh': $(grep -c 'teh' /tmp/test_angels_demons.md || echo 0)"
echo "  Level 0 has 'teh':  $(grep -c 'teh' /tmp/output_level0.md || echo 0) (unchanged)"
echo "  Level 1 has 'teh':  $(grep -c 'teh' /tmp/output_level1.md || echo 0) (should be 0!)"
echo "  Level 1 has 'the':  $(grep -c 'the' /tmp/output_level1.md || echo 0) (fixed!)"
echo ""

echo "Checking for double space fixes..."
echo "  Original double spaces:  $(grep -o '  ' /tmp/test_angels_demons.md | wc -l)"
echo "  Level 0 double spaces:   $(grep -o '  ' /tmp/output_level0.md | wc -l)"
echo "  Level 1 double spaces:   $(grep -o '  ' /tmp/output_level1.md | wc -l) (should be less!)"
echo ""

echo "Checking for Wikipedia category fixes..."
echo "  Original has '[[category:': $(grep -c '\[\[category:' /tmp/test_angels_demons.md || echo 0)"
echo "  Level 2 has '[[Category:': $(grep -c '\[\[Category:' /tmp/output_level2.md || echo 0)"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "DEMONSTRATION COMPLETE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✨ Files created:"
echo "   /tmp/test_angels_demons.md  - Original with issues"
echo "   /tmp/compressed.mq          - Demon compressed"
echo "   /tmp/output_level0.md       - Angel Level 0 (strict)"
echo "   /tmp/output_level1.md       - Angel Level 1 (minor blessings)"
echo "   /tmp/output_level2.md       - Angel Level 2 (harmony)"
echo "   /tmp/output_level3.md       - Angel Level 3 (creative)"
echo ""
echo "🔥 The eternal dance of information continues! 🔥"
echo ""
echo "Use cases:"
echo "  • Hutter Prize:        demon + angel level 0 (bit-perfect)"
echo "  • Clean Wikipedia:     demon + angel level 2 (harmonized)"
echo "  • ML Training Data:    demon + angel level 3 (variations)"
