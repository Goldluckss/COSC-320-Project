use std::fmt;
use std::io;

/// Error types for the compiler
/// 
/// These errors can be raised during lexing, parsing, or VM execution
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let lexer_err = CompilerError::LexerError("unknown token".to_string());
        let parser_err = CompilerError::ParserError("expected semicolon".to_string());
        let type_err = CompilerError::TypeError("incompatible types".to_string());
        let vm_err = CompilerError::VMError("invalid instruction".to_string());
        let io_err = CompilerError::IOError(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        
        assert_eq!(format!("{}", lexer_err), "Lexer error: unknown token");
        assert_eq!(format!("{}", parser_err), "Parser error: expected semicolon");
        assert_eq!(format!("{}", type_err), "Type error: incompatible types");
        assert_eq!(format!("{}", vm_err), "VM error: invalid instruction");
        assert!(format!("{}", io_err).starts_with("IO error: "));
    }
    
    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let compiler_err = CompilerError::from(io_err);
        
        match compiler_err {
            CompilerError::IOError(_) => assert!(true),
            _ => panic!("Expected IOError variant"),
        }
    }
}