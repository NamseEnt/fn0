#!/bin/bash
set -e

# OCI SDK Generation Pipeline
# Ports TypeScript SDK models to Rust SDK

echo "=================================="
echo "OCI SDK Generation Pipeline"
echo "=================================="
echo ""

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TS_SDK_PATH="$SCRIPT_DIR/oci-typescript-sdk"
DATA_DIR="$SCRIPT_DIR/data"
PARSED_DIR="$DATA_DIR/parsed"
OUTPUT_DIR="$SCRIPT_DIR/src"

# Priority services to generate (can be overridden by arguments)
SERVICES="${1:-core}"

# Check if TypeScript SDK exists
if [ ! -d "$TS_SDK_PATH" ]; then
    echo "Error: TypeScript SDK not found at $TS_SDK_PATH"
    echo "Please clone it first:"
    echo "  cd $SCRIPT_DIR && git clone https://github.com/oracle/oci-typescript-sdk.git"
    exit 1
fi

# Create necessary directories
mkdir -p "$PARSED_DIR"

echo "[1/4] Service Discovery"
echo "========================"
echo "Discovering services from TypeScript SDK..."
cd "$SCRIPT_DIR/tools/discovery"
npm run discover > "$DATA_DIR/services-metadata.json" 2>&1
SERVICES_COUNT=$(grep -c '"name"' "$DATA_DIR/services-metadata.json" || echo "0")
echo "✓ Discovered $SERVICES_COUNT services"
echo ""

echo "[2/4] TypeScript Parsing"
echo "========================"
for SERVICE in ${SERVICES//,/ }; do
    echo "Parsing $SERVICE..."
    cd "$SCRIPT_DIR/tools/parser"
    npx ts-node parse-models.ts --service="$SERVICE" 2>/dev/null > "$PARSED_DIR/${SERVICE}-models.json"

    # Count models
    INTERFACES=$(grep -c '"kind": "interface"' "$PARSED_DIR/${SERVICE}-models.json" || echo "0")
    ENUMS=$(grep -c '"kind": "enum"' "$PARSED_DIR/${SERVICE}-models.json" || echo "0")
    echo "  ✓ Parsed $INTERFACES interfaces, $ENUMS enums"
done
echo ""

echo "[3/4] Rust Code Generation"
echo "==========================="
for SERVICE in ${SERVICES//,/ }; do
    echo "Generating Rust code for $SERVICE..."

    # Create output directory for this service
    SERVICE_OUTPUT="$OUTPUT_DIR/$SERVICE/models"
    mkdir -p "$SERVICE_OUTPUT"

    # Generate models
    cd "$SCRIPT_DIR"
    cargo run --manifest-path tools/generator/Cargo.toml --release -- \
        --input "$PARSED_DIR/${SERVICE}-models.json" \
        --output "$SERVICE_OUTPUT" \
        2>&1 | grep -E "(Generating|Generated|complete|Interfaces|Enums)"

    # Generate mod.rs for the service
    echo "  Generating mod.rs..."
    if [ -d "$SERVICE_OUTPUT" ] && [ "$(ls -A $SERVICE_OUTPUT/*.rs 2>/dev/null | grep -v mod.rs)" ]; then
        ls "$SERVICE_OUTPUT"/*.rs | grep -v mod.rs | xargs -I {} basename {} .rs | \
            awk '{print "pub mod "$1";\npub use "$1"::*;"}' > "$SERVICE_OUTPUT/mod.rs"
        MODEL_COUNT=$(grep -c "^pub mod" "$SERVICE_OUTPUT/mod.rs" || echo "0")
        echo "  ✓ Generated mod.rs with $MODEL_COUNT modules"
    fi
done
echo ""

echo "[4/4] Validation & Code Quality"
echo "================================"
cd "$SCRIPT_DIR"

# Step 1: Compile check
echo "[4.1] Running cargo check..."
if cargo check --quiet 2>&1; then
    echo "✓ Code compiles successfully"
else
    echo "✗ Compilation failed. See errors above."
    exit 1
fi
echo ""

# Step 2: Auto-fix with clippy
echo "[4.2] Running clippy auto-fix..."
# Build features list for all generated services
FEATURES=""
for SERVICE in ${SERVICES//,/ }; do
    if [ -z "$FEATURES" ]; then
        FEATURES="$SERVICE"
    else
        FEATURES="$FEATURES,$SERVICE"
    fi
done

if cargo clippy --fix --allow-dirty --features "$FEATURES" --quiet 2>&1; then
    echo "✓ Clippy auto-fix completed"
else
    echo "⚠ Some clippy fixes may have failed (this is normal)"
fi
echo ""

# Step 3: Verify no clippy errors remain
echo "[4.3] Verifying clippy compliance..."
CLIPPY_OUTPUT=$(cargo clippy --features "$FEATURES" 2>&1)
CLIPPY_ERRORS=$(echo "$CLIPPY_OUTPUT" | grep "^error" | wc -l | tr -d ' ')
CLIPPY_WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep "^warning" | wc -l | tr -d ' ')

if [ "$CLIPPY_ERRORS" -eq "0" ]; then
    echo "✓ No clippy errors ($CLIPPY_WARNINGS warnings)"
else
    echo "✗ Found $CLIPPY_ERRORS clippy errors"
    echo ""
    echo "Clippy errors:"
    echo "$CLIPPY_OUTPUT" | grep -A 5 "^error"
    exit 1
fi
echo ""

# Step 4: Format code
echo "[4.4] Formatting code..."
if cargo fmt --quiet; then
    echo "✓ Code formatted"
else
    echo "⚠ Formatting issues detected"
fi
echo ""

echo "=================================="
echo "✓ Generation Complete!"
echo "=================================="
echo ""
echo "Generated services: $SERVICES"
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "Quality checks:"
echo "  ✓ Compilation: PASSED"
echo "  ✓ Clippy: $CLIPPY_ERRORS errors, $CLIPPY_WARNINGS warnings"
echo "  ✓ Formatting: APPLIED"
echo ""
echo "Next steps:"
echo "  1. Review generated code in $OUTPUT_DIR"
echo "  2. Run 'cargo test --features $FEATURES' to run tests"
echo "  3. Check examples compile: 'cargo check --examples --features $FEATURES'"
echo "  4. Commit changes: 'git add . && git commit -m \"Generate $SERVICES service\"'"
echo ""
echo "For detailed usage, see: GENERATING_SDK.md"
