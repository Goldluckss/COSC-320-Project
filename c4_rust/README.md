# C4 Rust Compiler

This project is a Rust implementation of the C4 compiler, originally written by Robert Swierczek in C. The C4 compiler implements a subset of C and is notably compact and self-hosting.

## Features

This Rust implementation maintains the same functionality as the original C4:

- Support for the same subset of C:
  - `char`, `int`, and pointer types
  - `if`, `while`, `return`, and expression statements
  - Function definitions and calls
  - Standard operators: arithmetic, logical, bitwise

- Self-hosting capability (can compile its own source code)
- Virtual machine for executing compiled bytecode
- Enhanced error handling with detailed diagnostics

## Enhanced Error Reporting

As a significant improvement over the original C4 compiler, this implementation features enhanced error reporting that provides detailed, context-rich error messages. This makes debugging easier and improves the overall developer experience.

### Key Features of Enhanced Error Reporting:

1. **Location Information**: All errors include precise line and column numbers
2. **Source Context**: Error messages display the relevant line of source code with a pointer to the exact error location
3. **Helpful Suggestions**: Where possible, errors include suggestions for fixing the problem
4. **Categorized Errors**: Errors are clearly categorized as lexer, parser, type, or VM errors
5. **VM Debugging**: Runtime errors include information about the executed instruction and VM cycle

## Building the Compiler

### Prerequisites

- Rust and Cargo (install from [rustup.rs](https://rustup.rs))

### Building from Source

1. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/c4_rust.git
   cd c4_rust
   ```

2. Build the compiler with Cargo:
   ```bash
   cargo build --release
   ```

3. The compiler executable will be located at `./target/release/c4_rust`

## Running the Compiler

The C4 Rust compiler supports the same command-line options as the original C4:

```bash
# Basic usage
./target/release/c4_rust source.c

# Show source code and generated instructions during compilation
./target/release/c4_rust -s source.c

# Show VM execution debugging information
./target/release/c4_rust -d source.c

# Both source and debug output
./target/release/c4_rust -s -d source.c
```

### Compiling Sample Programs

Here are examples of compiling and running some sample C programs:

#### Hello World

Create a file named `hello.c`:
```c
int main() {
    printf("Hello, World!\n");
    return 0;
}
```

Compile and run:
```bash
./target/release/c4_rust hello.c
```

#### Factorial Function

Create a file named `factorial.c`:
```c
int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

int main() {
    printf("5! = %d\n", factorial(5));
    return 0;
}
```

Compile and run:
```bash
./target/release/c4_rust factorial.c
```

#### Fibonacci Sequence

Create a file named `fibonacci.c`:
```c
int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n-1) + fibonacci(n-2);
}

int main() {
    int i;
    i = 0;
    while (i < 10) {
        printf("fibonacci(%d) = %d\n", i, fibonacci(i));
        i = i + 1;
    }
    return 0;
}
```

Compile and run:
```bash
./target/release/c4_rust fibonacci.c
```

## Self-hosting: Compiling the Original C4 Compiler

One of the most interesting features of C4 is its ability to compile itself (self-hosting). You can test this with our Rust implementation by having it compile the original C4 source code:

1. Download the original C4 source code:
   ```bash
   curl -O https://raw.githubusercontent.com/rswier/c4/master/c4.c
   ```

2. Compile it with our C4 Rust compiler:
   ```bash
   ./target/release/c4_rust c4.c
   ```

3. You can use the debugging flag to see the compilation and execution process:
   ```bash
   ./target/release/c4_rust -s -d c4.c
   ```

This demonstrates that our Rust implementation correctly handles the complete subset of C used in the original C4.

## Running the Test Suite

This project includes a comprehensive test suite to verify all compiler components:

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test lexer_     # Test the lexer
cargo test parser_    # Test the parser
cargo test symbol_    # Test the symbol table
cargo test vm_        # Test the virtual machine

# Run tests with verbose output
cargo test -- --nocapture
```

### Running Tests for a Specific Feature

You can also run tests for a specific feature:

```bash
# Test array operations
cargo test test_arrays

# Test function calls
cargo test test_function_call

# Test if-else statements
cargo test test_if_else
```

## Project Structure

The C4 Rust compiler is organized into these modules:

- `main.rs` - Entry point and command-line handling
- `lexer.rs` - Lexical analyzer for tokenizing source code
- `parser.rs` - Parser for generating bytecode from tokens
- `symbol.rs` - Symbol table for variable and function tracking
- `vm.rs` - Virtual machine for executing compiled bytecode
- `types.rs` - Type definitions used across the compiler
- `error.rs` - Enhanced error handling system with source context

## C4 Language Subset

The C4 compiler supports a small but powerful subset of C:

### Types
- `int` (64-bit integers in this implementation)
- `char` (8-bit characters)
- Pointers (with `*` syntax)
- Arrays

### Statements
- If-else: `if (condition) { ... } else { ... }`
- While loops: `while (condition) { ... }`
- Return: `return expression;`
- Expression statements: `a = b + c;`
- Blocks: `{ ... }`

### Expressions
- Binary operators: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `&`, `|`, `^`, `<<`, `>>`
- Unary operators: `-`, `!`, `~`, `*` (dereference), `&` (address-of)
- Function calls: `func(arg1, arg2)`
- Array access: `array[index]`
- Assignment: `var = expression`
- Pre/post increment/decrement: `++var`, `var++`, `--var`, `var--`

### Declarations
- Global variables: `int var;`
- Local variables: `int var;`
- Functions: `int func(int param) { ... }`
- Enums: `enum Name { VALUE1, VALUE2 };`

## Limitations

The C4 subset does not support:

- Structs and unions
- Floating-point types
- Switch statements
- For loops (use while instead)
- Standard library (except for a few system calls)
- Preprocessor directives (except for comments)

## Contributing

Contributions are welcome! Here are ways you can contribute:

1. Reporting bugs
2. Suggesting enhancements
3. Adding new features
4. Improving documentation
5. Submitting pull requests
