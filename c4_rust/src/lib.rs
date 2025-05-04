/// C4 Compiler in Rust
///
/// This is a Rust implementation of the C4 compiler, originally written by Robert Swierczek.
/// The compiler translates a subset of C into bytecode and includes a virtual machine
/// to execute the compiled code.
///
/// The compiler supports:
/// - char, int, and pointer types
/// - if, while, return, and expression statements
/// - Function definitions and calls
/// - Basic operators: arithmetic, logical, bitwise

// Export all modules
pub mod error;
pub mod lexer;
pub mod parser;
pub mod symbol;
pub mod types;
pub mod vm;

// Re-export commonly used types
pub use parser::Parser;
pub use types::{TokenType, Type, Opcode};
pub use symbol::SymbolTable;