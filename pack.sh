#!/bin/bash

# Check if an input folder is provided
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 /path/to/input/folder /output/folder"
    exit 1
fi

INPUT_DIR="$1"
OUTPUT_DIR="$2"

mkdir -p "$OUTPUT_DIR"

# Check if input folder exists
if [ ! -d "$INPUT_DIR" ]; then
    echo "The provided input directory does not exist."
    exit 2
fi

# Loop through each .ast file in the input folder
find "$INPUT_DIR" -type f -name "*.ast" | while read -r ast_file; do
    # Construct the output file path with .yaml extension
    yaml_file="${ast_file%.*}.cn"

    filename=$(basename "$ast_file")
    new_path="$OUTPUT_DIR/$filename"
    
    # Call the artemis_ast tool
    ./target/release/artemis_ast merge "$ast_file" "$yaml_file" "$new_path"
done
