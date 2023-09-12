# Artemis AST Script Processor

This utility offers a set of Rust functions to parse and manipulate Artemis AST-based scripts. 

## Features

1. **Tokenization**: Converts raw script into a series of tokens.
2. **Parsing**: Transforms tokens into a structured AST represented as a HashMap.
3. **AST Pruning**: Removes unnecessary elements from the AST for a cleaner representation.
4. **Script Generation**: Converts the modified AST back into its script form.
5. **Scenario Extraction and Replacement**: Aids in replacing parts of the script based on your requirements.

## Getting Started

### Build

1. **Clone the Repository**:

    ```bash
    git clone https://github.com/your_username/ast-script-processor.git
    cd ast-script-processor
    ```

2. **Build the Project**:

    With Rust installed, building is as simple as:

    ```bash
    cargo build --release
    ```

    This will create an optimized executable in the `target/release` directory.

### Usage

1. Parse the AST:

   ```rust
   let ast = parse_ast("path/to/script.txt").unwrap();
   ```

2. Prune the AST:

   ```rust
   prune_ast(&mut ast);
   ```

3. Convert the AST back to a script:

   ```rust
   let script = hashmap_to_script(&ast).unwrap();
   ```

4. Extract and replace scenario:

   ```rust
   replace_secnario(&ast, vec!["11", "22"]).unwrap();
   ```


## License

[MIT](https://choosealicense.com/licenses/mit/)
