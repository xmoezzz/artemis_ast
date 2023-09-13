#!/bin/bash

# Check if an input folder is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 /path/to/input/folder"
    exit 1
fi

INPUT_DIR="$1"

# Check if input folder exists
if [ ! -d "$INPUT_DIR" ]; then
    echo "The provided input directory does not exist."
    exit 2
fi

# Loop through each .ast file in the input folder
find "$INPUT_DIR" -type f -name "*.ast" | while read -r ast_file; do
    # Construct the output file path with .yaml extension
    output_file="${ast_file%.*}.yaml"
    
    # Call the artemis_ast tool
    ./target/release/artemis_ast extract "$ast_file" "$output_file"
done
