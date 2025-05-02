# C4 Compiler in Rust

This is a Rust implementation of the C4 compiler, originally written in C. The goal is to maintain the same functionality as the original C4 compiler while leveraging Rust's safety features and modern language capabilities.

## Features

- Tokenization of C code
- Parsing of C syntax
- Virtual machine execution
- Symbol table management
- Error handling

## Building

To build the compiler:

```bash
cargo build
```

To run tests:

```bash
cargo test
```

## Usage

To compile a C file:

```bash
target/debug/c4_rust <input_file.c>
```

## Project Structure

```
src/
├── lexer.rs     # Tokenization logic
├── parser.rs    # Parsing logic
├── vm.rs        # Virtual machine implementation
├── symbol.rs    # Symbol table management
├── error.rs     # Error handling
├── types.rs    # Token types 
└── main.rs      # Main entry point
```

## Contributing

Feel free to submit issues and enhancement requests!

## License

MIT
