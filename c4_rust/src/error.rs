use std::fmt;
use std::io;

/// Error types for the compiler
#[derive(Debug)]
pub enum CompilerError {
    /// Lexer errors (tokenization)
    LexerError(String),
    
    /// Parser errors (syntax)
    ParserError(String),
    
    /// Type errors (semantics)
    TypeError(String),
    
    /// VM runtime errors
    VMError(String),
    
    /// IO errors (file operations)
    IOError(io::Error),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::LexerError(msg) => write!(f, "Lexer error: {}", msg),
            CompilerError::ParserError(msg) => write!(f, "Parser error: {}", msg),
            CompilerError::TypeError(msg) => write!(f, "Type error: {}", msg),
            CompilerError::VMError(msg) => write!(f, "VM error: {}", msg),
            CompilerError::IOError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for CompilerError {}

impl From<io::Error> for CompilerError {
    fn from(error: io::Error) -> Self {
        CompilerError::IOError(error)
    }
}