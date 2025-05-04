use crate::types::TokenType;
use crate::error::CompilerError;
use std::collections::HashMap;

/// Represents the current token with its metadata
#[derive(Debug, Clone)]
pub struct Token {
    /// Type of the token
    pub token_type: TokenType,
    /// Value for numeric tokens
    pub value: Option<i64>,
    /// Name for identifier tokens
    pub name: Option<String>,
}

/// The lexer state for tokenizing source code
pub struct Lexer {
    /// Source code
    source: String,
    /// Current position in source
    position: usize,
    /// Line start position (for error reporting)
    line_position: usize,
    /// Current line number
    line: usize,
    /// Current column position
    column: usize,
    
    /// Current token information
    current: Token,
    
    /// Keyword mapping
    keywords: HashMap<String, TokenType>,
    
    /// Flag to print source lines during processing
    print_source: bool,
    
    /// Store source as lines for error reporting
    source_lines: Vec<String>,
}

impl Lexer {
    /// Create a new lexer from source code string
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to tokenize
    /// * `print_source` - Whether to print source lines during processing
    pub fn new(source: String, print_source: bool) -> Self {
        // Split source into lines for error reporting
        let source_lines: Vec<String> = source.lines().map(|l| l.to_string()).collect();
        
        let mut lexer = Lexer {
            source,
            position: 0,
            line_position: 0,
            line: 1,
            column: 1,
            current: Token {
                token_type: TokenType::Eof,
                value: None,
                name: None,
            },
            keywords: HashMap::new(),
            print_source,
            source_lines,
        };
        
        // Initialize keyword map
        lexer.init_keywords();
        
        lexer
    }
    
    /// Initialize the keyword mapping
    fn init_keywords(&mut self) {
        // C keywords recognized by the C4 compiler
        self.keywords.insert("char".to_string(), TokenType::Char);
        self.keywords.insert("else".to_string(), TokenType::Else);
        self.keywords.insert("enum".to_string(), TokenType::Enum);
        self.keywords.insert("if".to_string(), TokenType::If);
        self.keywords.insert("int".to_string(), TokenType::Int);
        self.keywords.insert("return".to_string(), TokenType::Return);
        self.keywords.insert("sizeof".to_string(), TokenType::Sizeof);
        self.keywords.insert("while".to_string(), TokenType::While);
        self.keywords.insert("void".to_string(), TokenType::Void);
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
        self.column += 1;
        
        // Reset column on newline
        if current == Some('\n') {
            self.column = 1;
        }
        
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
                    // We'll use Tilde for logical NOT (like C4.c)
                    Token {
                        token_type: TokenType::Tilde,
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
                    token_type: TokenType::Tilde,
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
            // Preprocessor directive or comment
            '#' => {
                self.advance();
                // Skip until end of line
                while let Some(ch) = self.current_char() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                return self.next_token(); // Get next token after directive
            },
            // Unrecognized character
            _ => {
                return Err(CompilerError::LexerError {
                    message: format!("Unexpected character: '{}' at line {}", ch, self.line),
                    location: None,
                    source_line: None,
                });
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
                self.column = 1; // Reset column on newline
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
        let _start_column = self.column;
        
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
        let _start_column = self.column;
        
        // Check for hex or octal prefix
        let first_digit = self.current_char().unwrap();
        if first_digit == '0' {
            self.advance();
            
            if let Some(ch) = self.current_char() {
                if ch == 'x' || ch == 'X' {
                    // Hexadecimal
                    self.advance();
                    
                    let hex_start = self.position;
                    while let Some(ch) = self.current_char() {
                        if ch.is_digit(16) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    let hex_str = &self.source[hex_start..self.position];
                    if hex_str.is_empty() {
                        return Err(CompilerError::LexerError {
                            message: format!("Invalid hexadecimal number at line {}", self.line),
                            location: None,
                            source_line: None,
                        });
                    }
                    
                    let value = i64::from_str_radix(hex_str, 16).map_err(|_e| {
                        CompilerError::LexerError {
                            message: format!("Invalid hexadecimal number: 0x{}", hex_str),
                            location: None,
                            source_line: None,
                        }
                    })?;
                    
                    return Ok(Token {
                        token_type: TokenType::Num,
                        value: Some(value),
                        name: None,
                    });
                } else if ch >= '0' && ch <= '7' {
                    // Octal
                    let oct_start = self.position - 1; // Include the leading 0
                    
                    while let Some(ch) = self.current_char() {
                        if ch >= '0' && ch <= '7' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    let oct_str = &self.source[oct_start..self.position];
                    let value = i64::from_str_radix(oct_str, 8).map_err(|_e| {
                        CompilerError::LexerError {
                            message: format!("Invalid octal number: {}", oct_str),
                            location: None,
                            source_line: None,
                        }
                    })?;
                    
                    return Ok(Token {
                        token_type: TokenType::Num,
                        value: Some(value),
                        name: None,
                    });
                } else {
                    // Just a zero
                    return Ok(Token {
                        token_type: TokenType::Num,
                        value: Some(0),
                        name: None,
                    });
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
        
        // Decimal number
        while let Some(ch) = self.current_char() {
            if ch.is_digit(10) {
                self.advance();
            } else {
                break;
            }
        }
        
        // Parse the decimal value
        let dec_str = &self.source[start_pos..self.position];
        let value = dec_str.parse::<i64>().map_err(|_e| {
            CompilerError::LexerError {
                message: format!("Invalid decimal number: {}", dec_str),
                location: None,
                source_line: None,
            }
        })?;
        
        Ok(Token {
            token_type: TokenType::Num,
            value: Some(value),
            name: None,
        })
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
                    None => return Err(CompilerError::LexerError {
                        message: format!("Unexpected end of file in escape sequence at line {}", self.line),
                        location: None,
                        source_line: None,
                    }),
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
            return Err(CompilerError::LexerError {
                message: format!("Unterminated {} literal at line {}", 
                    if is_string { "string" } else { "character" }, 
                    self.line),
                location: None,
                source_line: None,
            });
        }
        
        if is_string {
            // Return the string value (for C4 compatibility, this is the address)
            Ok(Token {
                token_type: TokenType::Num,
                value: Some(string_content.as_ptr() as i64),
                name: Some(string_content),
            })
        } else {
            // For character literals, use Num token type with the character value
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
    
    /// Get the current column position
    pub fn column(&self) -> usize {
        self.column
    }
    
    /// Get the current line content for error reporting
    pub fn get_current_line(&self) -> String {
        if self.line <= self.source_lines.len() {
            self.source_lines[self.line - 1].clone()
        } else {
            String::new()
        }
    }
}