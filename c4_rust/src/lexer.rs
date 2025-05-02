use crate::types::TokenType;
use crate::error::CompilerError;
use std::collections::HashMap;

/// Represents the current token with its metadata
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Option<i64>,
    pub name: Option<String>,
}

/// The lexer state for tokenizing source code
pub struct Lexer {
    // Source code and position tracking
    source: String,
    position: usize,
    line_position: usize,
    line: usize,
    
    // Current token information
    current: Token,
    
    // Keyword mapping
    keywords: HashMap<String, TokenType>,
    
    // Settings
    print_source: bool,
}

impl Lexer {
    /// Create a new lexer from source code string
    pub fn new(source: String, print_source: bool) -> Self {
        let mut lexer = Lexer {
            source,
            position: 0,
            line_position: 0,
            line: 1,
            current: Token {
                token_type: TokenType::Eof,
                value: None,
                name: None,
            },
            keywords: HashMap::new(),
            print_source,
        };
        
        // Initialize keyword map
        lexer.init_keywords();
        
        lexer
    }
    
    /// Initialize the keyword mapping
    fn init_keywords(&mut self) {
        // C keywords
        self.keywords.insert("char".to_string(), TokenType::Char);
        self.keywords.insert("else".to_string(), TokenType::Else);
        self.keywords.insert("enum".to_string(), TokenType::Enum);
        self.keywords.insert("if".to_string(), TokenType::If);
        self.keywords.insert("int".to_string(), TokenType::Int);
        self.keywords.insert("return".to_string(), TokenType::Return);
        self.keywords.insert("sizeof".to_string(), TokenType::Sizeof);
        self.keywords.insert("while".to_string(), TokenType::While);
    }
    
    /// Get the current character or None if at end of source
    fn current_char(&self) -> Option<char> {
        self.source.chars().nth(self.position)
    }
    
    /// Peek at the next character without advancing
    fn peek_char(&self) -> Option<char> {
        self.source.chars().nth(self.position + 1)
    }
    
    /// Advance to the next character
    fn advance(&mut self) -> Option<char> {
        let current = self.current_char();
        self.position += 1;
        current
    }
    
    /// Get the next token from the source code
    pub fn next_token(&mut self) -> Result<Token, CompilerError> {
        // Skip whitespace and comments
        self.skip_whitespace()?;
        
        // Check for end of file
        if self.position >= self.source.len() {
            self.current = Token {
                token_type: TokenType::Eof,
                value: None,
                name: None,
            };
            return Ok(self.current.clone());
        }
        
        // Process the next token based on the current character
        let ch = self.current_char().unwrap();
        
        let token = match ch {
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier()?,
            '0'..='9' => self.read_number()?,
            '"' | '\'' => self.read_string_or_char()?,
            '/' => {
                self.advance();
                if let Some('/') = self.current_char() {
                    // Line comment
                    self.skip_line_comment()?;
                    return self.next_token(); // Recursively get the next token
                } else {
                    Token {
                        token_type: TokenType::Div,
                        value: None,
                        name: None,
                    }
                }
            },
            // Handle operators
            '=' => {
                self.advance();
                if let Some('=') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Eq,
                        value: None,
                        name: None,
                    }
                } else {
                    Token {
                        token_type: TokenType::Assign,
                        value: None,
                        name: None,
                    }
                }
            },
            '+' => {
                self.advance();
                if let Some('+') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Inc,
                        value: None,
                        name: None,
                    }
                } else {
                    Token {
                        token_type: TokenType::Add,
                        value: None,
                        name: None,
                    }
                }
            },
            '-' => {
                self.advance();
                if let Some('-') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Dec,
                        value: None,
                        name: None,
                    }
                } else {
                    Token {
                        token_type: TokenType::Sub,
                        value: None,
                        name: None,
                    }
                }
            },
            '!' => {
                self.advance();
                if let Some('=') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Ne,
                        value: None,
                        name: None,
                    }
                } else {
                    Token {
                        token_type: TokenType::Not,
                        value: None,
                        name: None,
                    }
                }
            },
            '<' => {
                self.advance();
                if let Some('=') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Le,
                        value: None,
                        name: None,
                    }
                } else if let Some('<') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Shl,
                        value: None,
                        name: None,
                    }
                } else {
                    Token {
                        token_type: TokenType::Lt,
                        value: None,
                        name: None,
                    }
                }
            },
            '>' => {
                self.advance();
                if let Some('=') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Ge,
                        value: None,
                        name: None,
                    }
                } else if let Some('>') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Shr,
                        value: None,
                        name: None,
                    }
                } else {
                    Token {
                        token_type: TokenType::Gt,
                        value: None,
                        name: None,
                    }
                }
            },
            '|' => {
                self.advance();
                if let Some('|') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Lor,
                        value: None,
                        name: None,
                    }
                } else {
                    Token {
                        token_type: TokenType::Or,
                        value: None,
                        name: None,
                    }
                }
            },
            '&' => {
                self.advance();
                if let Some('&') = self.current_char() {
                    self.advance();
                    Token {
                        token_type: TokenType::Lan,
                        value: None,
                        name: None,
                    }
                } else {
                    Token {
                        token_type: TokenType::And,
                        value: None,
                        name: None,
                    }
                }
            },
            '^' => {
                self.advance();
                Token {
                    token_type: TokenType::Xor,
                    value: None,
                    name: None,
                }
            },
            '%' => {
                self.advance();
                Token {
                    token_type: TokenType::Mod,
                    value: None,
                    name: None,
                }
            },
            '*' => {
                self.advance();
                Token {
                    token_type: TokenType::Mul,
                    value: None,
                    name: None,
                }
            },
            '[' => {
                self.advance();
                Token {
                    token_type: TokenType::Brak,
                    value: None,
                    name: None,
                }
            },
            '?' => {
                self.advance();
                Token {
                    token_type: TokenType::Cond,
                    value: None,
                    name: None,
                }
            },
            // Single character tokens
            '~' => {
                self.advance();
                Token {
                    token_type: TokenType::BitNot,
                    value: None,
                    name: None,
                }
            },
            ';' => {
                self.advance();
                Token {
                    token_type: TokenType::Semicolon,
                    value: None,
                    name: None,
                }
            },
            '{' => {
                self.advance();
                Token {
                    token_type: TokenType::LBrace,
                    value: None,
                    name: None,
                }
            },
            '}' => {
                self.advance();
                Token {
                    token_type: TokenType::RBrace,
                    value: None,
                    name: None,
                }
            },
            '(' => {
                self.advance();
                Token {
                    token_type: TokenType::LParen,
                    value: None,
                    name: None,
                }
            },
            ')' => {
                self.advance();
                Token {
                    token_type: TokenType::RParen,
                    value: None,
                    name: None,
                }
            },
            ']' => {
                self.advance();
                Token {
                    token_type: TokenType::RBracket,
                    value: None,
                    name: None,
                }
            },
            ',' => {
                self.advance();
                Token {
                    token_type: TokenType::Comma,
                    value: None,
                    name: None,
                }
            },
            ':' => {
                self.advance();
                Token {
                    token_type: TokenType::Colon,
                    value: None,
                    name: None,
                }
            },
            // Unrecognized character
            _ => {
                return Err(CompilerError::LexerError(
                    format!("Unexpected character: '{}' at line {}", ch, self.line)
                ));
            }
        };
        
        self.current = token.clone();
        Ok(token)
    }
    
    /// Skip whitespace and track line numbers
    fn skip_whitespace(&mut self) -> Result<(), CompilerError> {
        while let Some(ch) = self.current_char() {
            if !ch.is_whitespace() {
                break;
            }
            
            // Track line numbers
            if ch == '\n' {
                if self.print_source {
                    let line_content = &self.source[self.line_position..self.position];
                    println!("{}: {}", self.line, line_content);
                    // In the original C4, this is where it would print generated code
                }
                self.line += 1;
                self.line_position = self.position + 1;
            }
            
            self.advance();
        }
        
        Ok(())
    }
    
    /// Skip a line comment
    fn skip_line_comment(&mut self) -> Result<(), CompilerError> {
        self.advance(); // Skip the second '/'
        
        while let Some(ch) = self.current_char() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        
        Ok(())
    }
    
    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> Result<Token, CompilerError> {
        let start_pos = self.position;
        
        // Read the entire identifier
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        // Extract the identifier
        let identifier = &self.source[start_pos..self.position];
        
        // Check if it's a keyword
        if let Some(&token_type) = self.keywords.get(identifier) {
            return Ok(Token {
                token_type,
                value: None,
                name: None,
            });
        }
        
        // It's a user-defined identifier
        Ok(Token {
            token_type: TokenType::Id,
            value: None,
            name: Some(identifier.to_string()),
        })
    }
    
    /// Read a numeric literal
    fn read_number(&mut self) -> Result<Token, CompilerError> {
        let start_pos = self.position;
        let mut is_hex = false;
        let mut is_octal = false;
        
        // Check for hex or octal prefix
        let first_digit = self.current_char().unwrap();
        if first_digit == '0' {
            self.advance();
            
            if let Some(ch) = self.current_char() {
                if ch == 'x' || ch == 'X' {
                    is_hex = true;
                    self.advance();
                } else if ch >= '0' && ch <= '7' {
                    is_octal = true;
                }
            } else {
                // Just a zero
                return Ok(Token {
                    token_type: TokenType::Num,
                    value: Some(0),
                    name: None,
                });
            }
        }
        
        // Read the rest of the number
        if is_hex {
            // Read hexadecimal
            while let Some(ch) = self.current_char() {
                if ch.is_digit(16) {
                    self.advance();
                } else {
                    break;
                }
            }
            
            // Parse the hex value
            let hex_str = &self.source[start_pos + 2..self.position]; // Skip "0x"
            if hex_str.is_empty() {
                return Err(CompilerError::LexerError(
                    format!("Invalid hexadecimal number at line {}", self.line)
                ));
            }
            
            let value = i64::from_str_radix(hex_str, 16).map_err(|_| {
                CompilerError::LexerError(format!("Invalid hexadecimal number: 0x{}", hex_str))
            })?;
            
            Ok(Token {
                token_type: TokenType::Num,
                value: Some(value),
                name: None,
            })
        } else if is_octal {
            // Read octal
            while let Some(ch) = self.current_char() {
                if ch >= '0' && ch <= '7' {
                    self.advance();
                } else {
                    break;
                }
            }
            
            // Parse the octal value
            let oct_str = &self.source[start_pos..self.position];
            let value = i64::from_str_radix(oct_str, 8).map_err(|_| {
                CompilerError::LexerError(format!("Invalid octal number: {}", oct_str))
            })?;
            
            Ok(Token {
                token_type: TokenType::Num,
                value: Some(value),
                name: None,
            })
        } else {
            // Read decimal
            while let Some(ch) = self.current_char() {
                if ch.is_digit(10) {
                    self.advance();
                } else {
                    break;
                }
            }
            
            // Parse the decimal value
            let dec_str = &self.source[start_pos..self.position];
            let value = dec_str.parse::<i64>().map_err(|_| {
                CompilerError::LexerError(format!("Invalid decimal number: {}", dec_str))
            })?;
            
            Ok(Token {
                token_type: TokenType::Num,
                value: Some(value),
                name: None,
            })
        }
    }
    
    /// Read a string or character literal
    fn read_string_or_char(&mut self) -> Result<Token, CompilerError> {
        let quote_char = self.current_char().unwrap();
        let is_string = quote_char == '"';
        
        self.advance(); // Skip the opening quote
        
        let mut value: i64 = 0;
        let mut string_content = String::new();
        
        // Read until the closing quote
        while let Some(ch) = self.current_char() {
            if ch == quote_char {
                break;
            }
            
            // Handle escape sequences
            let char_value = if ch == '\\' {
                self.advance();
                match self.current_char() {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some('"') => '"',
                    Some('0') => '\0',
                    Some(esc) => esc,
                    None => return Err(CompilerError::LexerError(
                        format!("Unexpected end of file in escape sequence at line {}", self.line)
                    )),
                }
            } else {
                ch
            };
            
            self.advance();
            
            if !is_string {
                // For character literals, just store the value
                value = char_value as i64;
            } else {
                // For string literals, append to the content
                string_content.push(char_value);
            }
        }
        
        // Skip the closing quote
        if self.current_char() == Some(quote_char) {
            self.advance();
        } else {
            return Err(CompilerError::LexerError(
                format!("Unterminated {} literal at line {}", 
                    if is_string { "string" } else { "character" }, 
                    self.line
                )
            ));
        }
        
        if is_string {
            // For string literals, we'll use a special token type and store the content
            Ok(Token {
                token_type: TokenType::Str,
                value: None,
                name: Some(string_content),
            })
        } else {
            // For character literals, we'll use Num token type with the character value
            Ok(Token {
                token_type: TokenType::Num,
                value: Some(value),
                name: None,
            })
        }
    }
    
    /// Get the current token
    pub fn current_token(&self) -> &Token {
        &self.current
    }
    
    /// Get the current line number
    pub fn line(&self) -> usize {
        self.line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "int char if else while return sizeof enum".to_string();
        let mut lexer = Lexer::new(source, false);
        
        let expected = vec![
            TokenType::Int,
            TokenType::Char,
            TokenType::If,
            TokenType::Else,
            TokenType::While,
            TokenType::Return,
            TokenType::Sizeof,
            TokenType::Enum,
        ];
        
        for expected_type in expected {
            let token = lexer.next_token().unwrap();
            assert_eq!(token.token_type, expected_type);
        }
        
        // Should be EOF at the end
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Eof);
    }
    
    #[test]
    fn test_identifiers() {
        let source = "variable _underscore var123".to_string();
        let mut lexer = Lexer::new(source, false);
        
        // First token: "variable"
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Id);
        assert_eq!(token.name.unwrap(), "variable");
        
        // Second token: "_underscore"
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Id);
        assert_eq!(token.name.unwrap(), "_underscore");
        
        // Third token: "var123"
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token_type, TokenType::Id);
        assert_eq!(token.name.unwrap(), "var123");
    }
}