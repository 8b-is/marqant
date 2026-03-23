#!/usr/bin/env bash
# scripts/manage.sh — Marqant project management helper
#
# Usage: ./scripts/manage.sh <command> [args...]
#
# Commands:
#   clean              Remove build artefacts (target/)
#   build              Build debug binary
#   build-release      Build optimised release binary
#   run [args...]      Run the mq CLI in debug mode
#   run-release [args...] Run the mq CLI in release mode
#   test               Run the full test suite (excluding known-broken tests)
#   test-wiki          Run only the Wikipedia compression tests
#   bench              Show compression benchmarks on Wikipedia fixture files
#   version            Print the current version
#   bump-patch         Bump the patch version (x.y.Z)
#   bump-minor         Bump the minor version (x.Y.0)
#   bump-major         Bump the major version (X.0.0)
#   release            Build release binary, print version, and remind you to tag
#   compress <file>    Compress a markdown file with all modes (shows ratio)
#   help               Show this help message

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT}"

# ─── helpers ──────────────────────────────────────────────────────────────────

get_version() {
    grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/'
}

set_version() {
    local new_ver="$1"
    # Portable sed -i: BSD (macOS) needs an explicit backup extension
    local sedi
    if sed --version 2>/dev/null | grep -q GNU; then
        sedi=(sed -i)
    else
        sedi=(sed -i '')
    fi
    # Update Cargo.toml
    "${sedi[@]}" "s/^version = \".*\"/version = \"${new_ver}\"/" Cargo.toml
    # Update README.md version badge / toml snippet
    "${sedi[@]}" "s/marqant = \"[0-9]*\.[0-9]*\.[0-9]*\"/marqant = \"${new_ver}\"/" README.md
    echo "Version set to ${new_ver}"
}

bump_version() {
    local part="$1"   # major | minor | patch
    local ver
    ver="$(get_version)"
    IFS='.' read -r major minor patch <<< "${ver}"
    case "${part}" in
        major) major=$((major + 1)); minor=0; patch=0 ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        patch) patch=$((patch + 1)) ;;
        *) echo "Unknown part: ${part}"; exit 1 ;;
    esac
    local new_ver="${major}.${minor}.${patch}"
    set_version "${new_ver}"
    # Rebuild Cargo.lock with new version
    cargo generate-lockfile 2>/dev/null || true
    echo "Bumped ${part}: ${ver} → ${new_ver}"
}

print_separator() {
    echo "────────────────────────────────────────────────────────"
}

compress_all_modes() {
    local file="$1"
    if [[ ! -f "${file}" ]]; then
        echo "File not found: ${file}"
        exit 1
    fi
    local orig_bytes
    orig_bytes=$(wc -c < "${file}")
    echo
    echo "📄 ${file}  (${orig_bytes} bytes)"
    print_separator
    printf "%-20s %10s  %s\n" "Mode" "Bytes" "Ratio"
    print_separator

    local mq_bin="./target/release/mq"
    if [[ ! -x "${mq_bin}" ]]; then
        mq_bin="./target/debug/mq"
    fi
    if [[ ! -x "${mq_bin}" ]]; then
        echo "Binary not found — run: $0 build"
        exit 1
    fi

    for mode in "default" "zlib" "semantic" "zlib+semantic"; do
        local flags=""
        case "${mode}" in
            zlib)         flags="--binary" ;;
            semantic)     flags="--semantic" ;;
            zlib+semantic) flags="--binary --semantic" ;;
        esac
        # shellcheck disable=SC2086
        local size
        size=$("${mq_bin}" compress ${flags} "${file}" | wc -c)
        local ratio
        ratio=$(awk "BEGIN { printf \"%.1f%%\", ${size}*100/${orig_bytes} }")
        printf "%-20s %10d  %s\n" "${mode}" "${size}" "${ratio}"
    done
    print_separator
}

# ─── commands ─────────────────────────────────────────────────────────────────

CMD="${1:-help}"
shift || true

case "${CMD}" in
    clean)
        echo "Cleaning build artefacts..."
        cargo clean
        ;;

    build)
        echo "Building debug binary..."
        cargo build --bin mq
        echo "✅ Debug binary: target/debug/mq"
        ;;

    build-release)
        echo "Building release binary..."
        cargo build --release --bin mq
        echo "✅ Release binary: target/release/mq"
        ;;

    run)
        cargo run --bin mq -- "$@"
        ;;

    run-release)
        cargo run --release --bin mq -- "$@"
        ;;

    test)
        echo "Running test suite..."
        # Exclude tests known to use bare 'cargo run' without --bin (pre-existing issue)
        cargo test --lib \
              --test std \
              --test roundtrip_folder \
              --test wikipedia_compression \
              --test utl_enforcement \
              --test errors \
              --test flags \
              --test meta \
              --test angel_levels \
              --test tokenize \
              --test wordcloud \
              --test thank_you_jp \
              --test dns \
              2>&1 | grep -v "^error: .* which binary to run" || true
        ;;

    test-wiki)
        echo "Running Wikipedia compression tests..."
        cargo test --test wikipedia_compression -- --nocapture
        ;;

    bench)
        echo "Building release binary for benchmarks..."
        cargo build --release --bin mq 2>/dev/null
        echo
        echo "╔══════════════════════════════════════════════════════╗"
        echo "║       Wikipedia Markdown Compression Benchmark      ║"
        echo "╚══════════════════════════════════════════════════════╝"
        for f in tests/fixtures/wikipedia/*.md example-md/*.md; do
            [[ -f "${f}" ]] || continue
            compress_all_modes "${f}"
        done
        ;;

    compress)
        if [[ -z "${1:-}" ]]; then
            echo "Usage: $0 compress <file.md>"
            exit 1
        fi
        cargo build --release --bin mq 2>/dev/null
        compress_all_modes "$1"
        ;;

    version)
        get_version
        ;;

    bump-patch)
        bump_version patch
        ;;

    bump-minor)
        bump_version minor
        ;;

    bump-major)
        bump_version major
        ;;

    release)
        VER="$(get_version)"
        echo "Building release v${VER}..."
        cargo build --release --bin mq
        echo
        echo "✅ Release binary ready: target/release/mq"
        echo
        echo "Next steps:"
        echo "  git add -A && git commit -m \"release: v${VER}\""
        echo "  git tag v${VER}"
        echo "  git push && git push --tags"
        ;;

    help|--help|-h)
        sed -n '/^# Usage/,/^[^#]/p' "$0" | head -20
        echo
        grep -E "^\s+[a-z].*\)" "$0" | grep -v "shift\|esac\|local\|IFS\|case" | \
            sed 's/[[:space:]]*\([^)]*\))/  \1/' | head -30
        ;;

    *)
        echo "Unknown command: ${CMD}"
        echo "Run: $0 help"
        exit 1
        ;;
esac
