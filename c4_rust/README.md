# C4 Compiler in Rust

This is a Rust implementation of the C4 compiler, originally written in C by Robert Swierczek. The C4 compiler is a minimal, self-hosting C compiler capable of compiling itself.

## Features

- Tokenization of C code (lexer)
- Symbol table management with scope support
- Virtual machine for code execution
- Support for the C4 subset of C:
  - `char`, `int`, and pointer types
  - `if`, `while`, `return`, and expression statements
  - Function definitions and calls

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
cargo run -- [-s] [-d] <input_file.c>
```

Where:
- `-s`: Print source and generated code
- `-d`: Debug mode (print VM instructions during execution)

## Project Structure

The project is organized into several modules:

```
src/
├── lexer.rs     # Tokenization of source code
├── parser.rs    # Parsing and code generation (in progress)
├── symbol.rs    # Symbol table management
├── vm.rs        # Virtual machine implementation
├── types.rs     # Common types an