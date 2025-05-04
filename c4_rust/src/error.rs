use std::fmt;
use std::io;

/// Source location information for error reporting
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize) -> Self {
        SourceLocation { line, column }
    }
    
    /// Format the location as "line:column"
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.line, self.column)
    }
}

/// Error types for the compiler
/// 
/// These errors can be raised during lexing, parsing, or VM execution
#[derive(Debug)]
pub enum CompilerError {
    /// Lexer errors (tokenization)
    LexerError {
        message: String,
        location: Option<SourceLocation>,
        source_line: Option<String>,
    },
    
    /// Parser errors (syntax)
    ParserError {
        message: String,
        location: Option<SourceLocation>,
        source_line: Option<String>,
        suggestion: Option<String>,
    },
    
    /// Type errors (semantics)
    TypeError {
        message: String,
        location: Option<SourceLocation>,
        source_line: Option<String>,
        suggestion: Option<String>,
    },
    
    /// VM runtime errors
    VMError {
        message: String,
        instruction: Option<String>,
        cycle: Option<i64>,
    },
    
    /// IO errors (file operations)
    IOError(io::Error),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::LexerError { message, location, source_line } => {
                writeln!(f, "Lexer error: {}", message)?;
                
                if let Some(loc) = location {
                    writeln!(f, "  --> {}", loc.to_string())?;
                    
                    if let Some(line) = source_line {
                        writeln!(f, "   |")?;
                        writeln!(f, "{} |", loc.line)?;
                        writeln!(f, "   | {}", line)?;
                        writeln!(f, "   | {}{}",
                               " ".repeat(loc.column - 1),
                               "^")?;
                    }
                }
                
                Ok(())
            },
            CompilerError::ParserError { message, location, source_line, suggestion } => {
                writeln!(f, "Parser error: {}", message)?;
                
                if let Some(loc) = location {
                    writeln!(f, "  --> {}", loc.to_string())?;
                    
                    if let Some(line) = source_line {
                        writeln!(f, "   |")?;
                        writeln!(f, "{} |", loc.line)?;
                        writeln!(f, "   | {}", line)?;
                        writeln!(f, "   | {}{}",
                               " ".repeat(loc.column - 1),
                               "^")?;
                    }
                }
                
                if let Some(hint) = suggestion {
                    writeln!(f, "Suggestion: {}", hint)?;
                }
                
                Ok(())
            },
            CompilerError::TypeError { message, location, source_line, suggestion } => {
                writeln!(f, "Type error: {}", message)?;
                
                if let Some(loc) = location {
                    writeln!(f, "  --> {}", loc.to_string())?;
                    
                    if let Some(line) = source_line {
                        writeln!(f, "   |")?;
                        writeln!(f, "{} |", loc.line)?;
                        writeln!(f, "   | {}", line)?;
                        writeln!(f, "   | {}{}",
                               " ".repeat(loc.column - 1),
                               "^")?;
                    }
                }
                
                if let Some(hint) = suggestion {
                    writeln!(f, "Suggestion: {}", hint)?;
                }
                
                Ok(())
            },
            CompilerError::VMError { message, instruction, cycle } => {
                writeln!(f, "VM error: {}", message)?;
                
                if let Some(instr) = instruction {
                    writeln!(f, "  When executing: {}", instr)?;
                }
                
                if let Some(c) = cycle {
                    writeln!(f, "  At cycle: {}", c)?;
                }
                
                Ok(())
            },
            CompilerError::IOError(err) => {
                writeln!(f, "IO error: {}", err)
            },
        }
    }
}

impl std::error::Error for CompilerError {}

impl From<io::Error> for CompilerError {
    fn from(error: io::Error) -> Self {
        CompilerError::IOError(error)
    }
}

/// Helper functions to create specific errors
impl CompilerError {
    /// Create a lexer error
    pub fn lexer_error(message: &str, line: usize, column: usize, source_line: Option<&str>) -> Self {
        CompilerError::LexerError {
            message: message.to_string(),
            location: Some(SourceLocation::new(line, column)),
            source_line: source_line.map(|s| s.to_string()),
        }
    }
    
    /// Create a simple lexer error without location information
    pub fn simple_lexer_error(message: &str) -> Self {
        CompilerError::LexerError {
            message: message.to_string(),
            location: None,
            source_line: None,
        }
    }
    
    /// Create a parser error
    pub fn parser_error(
        message: &str, 
        line: usize, 
        column: usize, 
        source_line: Option<&str>,
        suggestion: Option<&str>,
    ) -> Self {
        CompilerError::ParserError {
            message: message.to_string(),
            location: Some(SourceLocation::new(line, column)),
            source_line: source_line.map(|s| s.to_string()),
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }
    
    /// Create a simple parser error without location information
    pub fn simple_parser_error(message: &str) -> Self {
        CompilerError::ParserError {
            message: message.to_string(),
            location: None,
            source_line: None,
            suggestion: None,
        }
    }
    
    /// Create a type error
    pub fn type_error(
        message: &str, 
        line: usize, 
        column: usize, 
        source_line: Option<&str>,
        suggestion: Option<&str>,
    ) -> Self {
        CompilerError::TypeError {
            message: message.to_string(),
            location: Some(SourceLocation::new(line, column)),
            source_line: source_line.map(|s| s.to_string()),
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }
    
    /// Create a simple type error without location information
    pub fn simple_type_error(message: &str) -> Self {
        CompilerError::TypeError {
            message: message.to_string(),
            location: None,
            source_line: None,
            suggestion: None,
        }
    }
    
    /// Create a VM error
    pub fn vm_error(message: &str, instruction: Option<&str>, cycle: Option<i64>) -> Self {
        CompilerError::VMError {
            message: message.to_string(),
            instruction: instruction.map(|s| s.to_string()),
            cycle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_source_location() {
        let loc = SourceLocation::new(10, 5);
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.to_string(), "10:5");
    }
    
    #[test]
    fn test_lexer_error() {
        let err = CompilerError::lexer_error(
            "Unexpected character '$'", 
            5, 
            10, 
            Some("    int x = $100;")
        );
        
        if let CompilerError::LexerError { message, location, source_line } = err {
            assert_eq!(message, "Unexpected character '$'");
            assert_eq!(location, Some(SourceLocation::new(5, 10)));
            assert_eq!(source_line, Some("    int x = $100;".to_string()));
        } else {
            panic!("Expected LexerError variant");
        }
    }
    
    #[test]
    fn test_parser_error_with_suggestion() {
        let err = CompilerError::parser_error(
            "Expected semicolon after expression", 
            7, 
            15, 
            Some("    int x = 5"),
            Some("Add a semicolon: 'int x = 5;'")
        );
        
        if let CompilerError::ParserError { message, location, source_line, suggestion } = err {
            assert_eq!(message, "Expected semicolon after expression");
            assert_eq!(location, Some(SourceLocation::new(7, 15)));
            assert_eq!(source_line, Some("    int x = 5".to_string()));
            assert_eq!(suggestion, Some("Add a semicolon: 'int x = 5;'".to_string()));
        } else {
            panic!("Expected ParserError variant");
        }
    }
    
    #[test]
    fn test_vm_error() {
        let err = CompilerError::vm_error("Division by zero", Some("DIV"), Some(42));
        
        if let CompilerError::VMError { message, instruction, cycle } = err {
            assert_eq!(message, "Division by zero");
            assert_eq!(instruction, Some("DIV".to_string()));
            assert_eq!(cycle, Some(42));
        } else {
            panic!("Expected VMError variant");
        }
    }
    
    #[test]
    fn test_error_display() {
        let lexer_err = CompilerError::lexer_error(
            "Unexpected character '@'", 
            3, 
            12, 
            Some("int x = @100;")
        );
        
        let display = format!("{}", lexer_err);
        assert!(display.contains("Lexer error: Unexpected character '@'"));
        assert!(display.contains("3:12"));
        assert!(display.contains("int x = @100;"));
        assert!(display.contains("           ^"));
        
        let parser_err = CompilerError::parser_error(
            "Expected semicolon", 
            5, 
            10, 
            Some("int x = 5"),
            Some("Add a semicolon after '5'")
        );
        
        let display = format!("{}", parser_err);
        assert!(display.contains("Parser error: Expected semicolon"));
        assert!(display.contains("5:10"));
        assert!(display.contains("int x = 5"));
        assert!(display.contains("         ^"));
        assert!(display.contains("Suggestion: Add a semicolon after '5'"));
    }
}