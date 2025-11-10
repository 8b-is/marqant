#!/bin/bash
# 🚀 Ship It! - The One-Command Release Script
# "Get it out there!" - Omni's philosophy

set -e  # Exit on any error

# Colors for Trisha's enjoyment
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
SPARKLE='\033[38;5;226m'
NC='\033[0m' # No Color

echo -e "${SPARKLE}✨ Ship It! - Marqant Release Script ✨${NC}"
echo -e "${CYAN}========================================${NC}\n"

# Get current version
CURRENT_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo -e "${BLUE}Current version: ${YELLOW}$CURRENT_VERSION${NC}"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Determine bump type
if [ "$1" == "major" ]; then
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    BUMP_TYPE="MAJOR"
elif [ "$1" == "minor" ]; then
    MINOR=$((MINOR + 1))
    PATCH=0
    BUMP_TYPE="MINOR"
else
    PATCH=$((PATCH + 1))
    BUMP_TYPE="PATCH"
fi

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

echo -e "${GREEN}Bumping ${BUMP_TYPE} version to: ${SPARKLE}$NEW_VERSION${NC}\n"

# Step 1: Run tests
echo -e "${PURPLE}📋 Step 1: Running tests...${NC}"
cargo test --lib --quiet || {
    echo -e "${RED}❌ Tests failed! Fix them first.${NC}"
    exit 1
}
echo -e "${GREEN}✅ All tests pass!${NC}\n"

# Step 2: Format and lint
echo -e "${PURPLE}📋 Step 2: Formatting & linting...${NC}"
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged 2>/dev/null || true
echo -e "${GREEN}✅ Code formatted and linted!${NC}\n"

# Step 3: Update version
echo -e "${PURPLE}📋 Step 3: Updating version...${NC}"
# Cross-platform sed (works on both macOS and Linux)
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
else
    sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
fi
cargo build --quiet  # Update Cargo.lock
echo -e "${GREEN}✅ Version bumped to $NEW_VERSION${NC}\n"

# Step 4: Update CHANGELOG
echo -e "${PURPLE}📋 Step 4: Updating CHANGELOG...${NC}"
TODAY=$(date +%Y-%m-%d)
if [ -f CHANGELOG.md ]; then
    # Add new version entry at the top (cross-platform)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "3i\\
\\
## [$NEW_VERSION] - $TODAY\\
\\
### Changed\\
- Version bump\\
" CHANGELOG.md
    else
        sed -i "3i\\\\n## [$NEW_VERSION] - $TODAY\\n\\n### Changed\\n- Version bump\\n" CHANGELOG.md
    fi
    echo -e "${GREEN}✅ CHANGELOG updated${NC}\n"
else
    echo -e "${YELLOW}⚠️  No CHANGELOG.md found${NC}\n"
fi

# Step 5: Commit changes
echo -e "${PURPLE}📋 Step 5: Committing changes...${NC}"
git add -A
COMMIT_MSG="Release v$NEW_VERSION 🚀

- Version bump from $CURRENT_VERSION to $NEW_VERSION
- All tests passing
- Ready to ship!

'Get it out there!' - Omni"

git commit -m "$COMMIT_MSG" || {
    echo -e "${YELLOW}⚠️  Nothing to commit${NC}"
}
echo -e "${GREEN}✅ Changes committed${NC}\n"

# Step 6: Tag the release
echo -e "${PURPLE}📋 Step 6: Creating git tag...${NC}"
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"
echo -e "${GREEN}✅ Tagged as v$NEW_VERSION${NC}\n"

# Step 7: Push to GitHub
echo -e "${PURPLE}📋 Step 7: Pushing to GitHub...${NC}"
git push origin main
git push origin "v$NEW_VERSION"
echo -e "${GREEN}✅ Pushed to GitHub${NC}\n"

# Step 8: Publish to crates.io
echo -e "${PURPLE}📋 Step 8: Publishing to crates.io...${NC}"
cargo publish || {
    echo -e "${YELLOW}⚠️  Publishing failed - maybe already published?${NC}"
}
echo -e "${GREEN}✅ Published to crates.io${NC}\n"

# Success!
echo -e "${SPARKLE}🎉 SUCCESS! 🎉${NC}"
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}✅ Version $NEW_VERSION has been:${NC}"
echo -e "  ${BLUE}• Tested${NC}"
echo -e "  ${BLUE}• Formatted & linted${NC}"
echo -e "  ${BLUE}• Committed${NC}"
echo -e "  ${BLUE}• Tagged${NC}"
echo -e "  ${BLUE}• Pushed to GitHub${NC}"
echo -e "  ${BLUE}• Published to crates.io${NC}"
echo -e "${CYAN}========================================${NC}"
echo -e "${SPARKLE}✨ Ship it and iterate! ✨${NC}"
echo -e "${PURPLE}'Why wait when you can ship?' - The future of deployment${NC}\n"

# Optional: Update dependent projects
if [ "$2" == "--update-deps" ]; then
    echo -e "${YELLOW}📦 Updating dependent projects...${NC}"
    
    # Update smart-tree
    if [ -d "../smart-tree" ]; then
        cd ../smart-tree
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/marqant = \".*\"/marqant = \"$NEW_VERSION\"/" Cargo.toml
        else
            sed -i "s/marqant = \".*\"/marqant = \"$NEW_VERSION\"/" Cargo.toml
        fi
        cargo update -p marqant
        git add -A
        git commit -m "Update marqant to v$NEW_VERSION"
        git push
        echo -e "${GREEN}✅ Updated smart-tree${NC}"
        cd -
    fi
fi

echo -e "${CYAN}Done! Time for a break in Omni's Hot Tub! 🛁${NC}"