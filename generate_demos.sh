#!/bin/bash
set -e

# Configuration
ASSETS_DIR="$(pwd)/apps/landing/public/assets"
TAPES=("demo.tape" "docs_human.tape" "docs_machine.tape" "issues_human.tape" "issues_machine.tape")

echo "🔧 Preparing Chisel Demo Generation..."

# 1. Build Chisel (Release mode for performance in demos)
echo "📦 Building Chisel binary..."
cargo build --release --bin chisel --quiet

# Verify binary exists
BINARY_PATH="$(pwd)/target/release/chisel"
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Error: Binary not found at $BINARY_PATH"
    exit 1
fi

# 2. Setup Temporary Directory
TEMP_DIR=$(mktemp -d)
echo "🛠️  Working in temp directory: $TEMP_DIR"

# Copy binary to temp dir and add to PATH
cp "$BINARY_PATH" "$TEMP_DIR/"
cp -r "$(pwd)/demo_data" "$TEMP_DIR/"
export PATH="$TEMP_DIR:$PATH"

# 3. Process Tapes
for tape in "${TAPES[@]}"; do
    if [ ! -f "$tape" ]; then
        echo "⚠️  Warning: Tape file '$tape' not found. Skipping."
        continue
    fi

    echo "🎥 Recording $tape..."
    
    # Define paths
    TAPE_ABS_PATH="$(pwd)/$tape"
    OUTPUT_FILENAME="${tape%.tape}.webm"
    OUTPUT_ABS_PATH="$ASSETS_DIR/$OUTPUT_FILENAME"

    # Switch to temp dir to run VHS
    # This ensures 'chisel init' and other file ops happen in temp dir
    pushd "$TEMP_DIR" > /dev/null

    # Clean any previous state in the temp dir
    rm -rf .chisel

    # Run VHS
    # -o overrides the output path in the tape file
    vhs "$TAPE_ABS_PATH" -o "$OUTPUT_FILENAME"

    # Move the generated file to the assets directory
    if [ -f "$OUTPUT_FILENAME" ]; then
        mv "$OUTPUT_FILENAME" "$OUTPUT_ABS_PATH"
        echo "✅ Generated $OUTPUT_ABS_PATH"
    else
        echo "❌ Error: Failed to generate $OUTPUT_FILENAME"
    fi

    popd > /dev/null
done

# 4. Cleanup
rm -rf "$TEMP_DIR"
echo "✨ All demos generated successfully!"
