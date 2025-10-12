#!/bin/bash

# Script to collect default keymaps from QMK firmware
set -e

# Change to the project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "🔍 Collecting QMK keymaps from ~/qmk_firmware/keyboards/..."
echo "📂 Working directory: $(pwd)"

# Check if QMK firmware directory exists
QMK_DIR="$HOME/qmk_firmware"
if [ ! -d "$QMK_DIR" ]; then
    echo "❌ Error: QMK firmware directory not found at $QMK_DIR"
    echo "   Please ensure QMK firmware is installed and accessible."
    exit 1
fi

# Create tests/files directory if it doesn't exist
mkdir -p tests/files

# Clean previous collection
echo "🧹 Cleaning previous keymap collection..."
rm -f tests/files/*_keymap.c
rm -f tests/files/keyboards.txt

# Initialize counters
total_keyboards=0
collected_keymaps=0
skipped_variants=0

# Create keyboards manifest file
echo "# QMK Keyboards Collection Manifest" > tests/files/keyboards.txt
echo "# Generated on: $(date)" >> tests/files/keyboards.txt
echo "# Source: $QMK_DIR/keyboards/" >> tests/files/keyboards.txt
echo "" >> tests/files/keyboards.txt

echo "📋 Collecting keymaps..."

# Find all keyboard directories
for keyboard_dir in "$QMK_DIR/keyboards"/*; do
    if [ ! -d "$keyboard_dir" ]; then
        continue
    fi
    
    keyboard_name=$(basename "$keyboard_dir")
    total_keyboards=$((total_keyboards + 1))
    
    # Look for default keymap.c files
    # Priority: 1) Direct keymap.c, 2) keymaps/default/keymap.c
    keymap_file=""
    
    # Check for direct keymap.c (main variant)
    if [ -f "$keyboard_dir/keymap.c" ]; then
        keymap_file="$keyboard_dir/keymap.c"
        echo "✅ Found main keymap: $keyboard_name"
    # Check for keymaps/default/keymap.c
    elif [ -f "$keyboard_dir/keymaps/default/keymap.c" ]; then
        keymap_file="$keyboard_dir/keymaps/default/keymap.c"
        echo "✅ Found default keymap: $keyboard_name"
    else
        # Check subdirectories for variants, but prefer the main one
        found_variant=false
        for variant_dir in "$keyboard_dir"/*; do
            if [ -d "$variant_dir" ] && [ -f "$variant_dir/keymaps/default/keymap.c" ]; then
                if [ "$found_variant" = false ]; then
                    keymap_file="$variant_dir/keymaps/default/keymap.c"
                    variant_name=$(basename "$variant_dir")
                    echo "✅ Found variant keymap: $keyboard_name/$variant_name"
                    found_variant=true
                else
                    skipped_variants=$((skipped_variants + 1))
                fi
            fi
        done
    fi
    
    # Copy the keymap file if found
    if [ -n "$keymap_file" ] && [ -f "$keymap_file" ]; then
        # Create a safe filename
        safe_name=$(echo "$keyboard_name" | sed 's/[^a-zA-Z0-9_-]/_/g')
        dest_file="tests/files/${safe_name}_keymap.c"
        
        # Copy the file
        cp "$keymap_file" "$dest_file"
        collected_keymaps=$((collected_keymaps + 1))
        
        # Add to manifest
        echo "${safe_name}_keymap.c" >> tests/files/keyboards.txt
    else
        echo "⚠️  No keymap found for: $keyboard_name"
    fi
    
    # Limit collection to first 200 keyboards to avoid overwhelming
    if [ $collected_keymaps -ge 200 ]; then
        echo "🛑 Reached collection limit of 200 keymaps"
        break
    fi
done

# Finalize manifest
echo "" >> tests/files/keyboards.txt
echo "# Summary:" >> tests/files/keyboards.txt
echo "# Total keyboards scanned: $total_keyboards" >> tests/files/keyboards.txt
echo "# Keymaps collected: $collected_keymaps" >> tests/files/keyboards.txt
echo "# Variants skipped: $skipped_variants" >> tests/files/keyboards.txt

echo ""
echo "🎉 Keymap collection completed!"
echo "📊 Statistics:"
echo "   • Total keyboards scanned: $total_keyboards"
echo "   • Keymaps collected: $collected_keymaps"
echo "   • Variants skipped: $skipped_variants"
echo "   • Collection limit: 200 keymaps"
echo ""
echo "📁 Files created:"
echo "   • tests/files/keyboards.txt (manifest)"
echo "   • tests/files/*_keymap.c ($collected_keymaps files)"
echo ""
echo "💡 Next steps:"
echo "   1. Run: cargo test qmk_compatibility --release -- --nocapture"
echo "   2. Check: tests/compatibility_report.md"
