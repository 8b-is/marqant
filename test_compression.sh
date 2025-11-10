#!/bin/bash
# Test compression improvement with high-frequency words

cat > /tmp/test_wave.md << 'EOF'
# Wave-Based Memory Systems

The wave memory system uses wave interference patterns to store information. The wave approach differs from traditional memory. Wave-based systems rely on wave patterns and wave interactions.

The key advantage of wave memory is that the wave can encode multiple pieces of information simultaneously. The wave patterns naturally implement parallelism through wave interference.

Wave-based computation offers several benefits:
- The wave memory is fault-tolerant
- The wave system scales naturally
- The wave approach matches biological systems

The wave interference creates holographic storage where each part contains information about the whole. The wave-based architecture represents a fundamental shift in how we think about memory and consciousness in artificial intelligence systems.

EOF

echo "=== Testing compression on wave-heavy content ==="
echo ""
echo "Original size: $(wc -c < /tmp/test_wave.md) bytes"
echo ""

cargo run --bin mq -- compress /tmp/test_wave.md /tmp/test_wave.mq 2>&1 | grep -E "(Original|Compressed|Ratio)"

if [ -f /tmp/test_wave.mq ]; then
    echo ""
    echo "=== Token assignments (showing high-frequency words) ==="
    grep "^=" /tmp/test_wave.mq | head -20
    echo ""
    echo "Compressed size: $(wc -c < /tmp/test_wave.mq) bytes"
fi
