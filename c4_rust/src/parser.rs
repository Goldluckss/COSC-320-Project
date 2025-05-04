use crate::error::CompilerError;
use crate::lexer::{Lexer, Token};
use crate::symbol::SymbolTable;
use crate::types::{Opcode, TokenType, Type};

/// Parser for C4 compiler
/// 
/// The parser transforms tokens from the lexer into bytecode
/// and manages the symbol table.
pub struct Parser {
    /// Lexer for tokenizing source code
    lexer: Lexer,

    /// Generated code segment
    code: Vec<i64>,

    /// Data segment
    data: Vec<u8>,

    /// Current token
    current_token: Token,

    /// Symbol table
    symbol_table: SymbolTable,

    /// Current parsing type
    current_type: Type,

    /// Current identifier name
    current_id_name: Option<String>,

    /// Current token value
    current_value: i64,

    /// Local variable offset
    local_offset: i64,

    /// Print source flag
    print_source: bool,
}

impl Parser {
    /// Create a new parser
    pub fn new(source: String, print_source: bool) -> Self {
        Parser {
            lexer: Lexer::new(source, print_source),
            code: Vec::new(),
            data: Vec::new(),
            current_token: Token {
                token_type: TokenType::Eof,
                value: None,
                name: None,
            },
            symbol_table: SymbolTable::new(),
            current_type: Type::INT,
            current_id_name: None,
            current_value: 0,
            local_offset: 0,
            print_source,
        }
    }

    /// Initialize the parser
    pub fn init(&mut self) -> Result<(), CompilerError> {
        // Initialize system functions
        self.init_system_functions();

        // Get the first token
        self.next_token()?;

        Ok(())
    }

    /// Initialize system function symbols
    fn init_system_functions(&mut self) {
        // System functions are represented by opcodes
        self.symbol_table.add("open", TokenType::Sys, Type::INT, Opcode::OPEN as i64);
        self.symbol_table.add("read", TokenType::Sys, Type::INT, Opcode::READ as i64);
        self.symbol_table.add("close", TokenType::Sys, Type::INT, Opcode::CLOS as i64);
        self.symbol_table.add("printf", TokenType::Sys, Type::INT, Opcode::PRTF as i64);
        self.symbol_table.add("malloc", TokenType::Sys, Type::INT, Opcode::MALC as i64);
        self.symbol_table.add("free", TokenType::Sys, Type::INT, Opcode::FREE as i64);
        self.symbol_table.add("memset", TokenType::Sys, Type::INT, Opcode::MSET as i64);
        self.symbol_table.add("memcmp", TokenType::Sys, Type::INT, Opcode::MCMP as i64);
        self.symbol_table.add("exit", TokenType::Sys, Type::INT, Opcode::EXIT as i64);
        self.symbol_table.add("void", TokenType::Void, Type::INT, 0);
    }

    /// Get the next token from the lexer
    fn next_token(&mut self) -> Result<(), CompilerError> {
        self.current_token = self.lexer.next_token()?;

        // Update current identifier name and value
        match &self.current_token.token_type {
            TokenType::Id => {
                self.current_id_name = self.current_token.name.clone();
            },
            TokenType::Num => {
                self.current_value = self.current_token.value.unwrap_or(0);
            },
            _ => {}
        }

        Ok(())
    }

    /// Emit a value to the code segment
    fn emit(&mut self, val: i64) -> usize {
        let pos = self.code.len();
        self.code.push(val);
        pos
    }

    /// Check if current token matches the expected token, then advance
    fn match_token(&mut self, expected: TokenType) -> Result<(), CompilerError> {
        if self.current_token.token_type == expected {
            self.next_token()?;
            Ok(())
        } else {
            Err(CompilerError::ParserError {
                message: format!("Expected {:?}, got {:?} at line {}", 
                    expected, 
                    self.current_token.token_type,
                    self.lexer.line()),
                location: None,
                source_line: None,
                suggestion: None,
            })
        }
    }

    /// Parse the C source code
    pub fn parse(&mut self) -> Result<(), CompilerError> {
        // Parse global declarations until end of file
        self.parse_declarations()?;

        // Look for main function
        if self.get_main_function().is_none() {
            return Err(CompilerError::ParserError {
                message: "main() not defined".to_string(),
                location: None,
                source_line: None,
                suggestion: None,
            });
        }

        Ok(())
    }

    /// Parse global declarations
    fn parse_declarations(&mut self) -> Result<(), CompilerError> {
        while self.current_token.token_type != TokenType::Eof {
            // Parse type
            self.parse_type()?;
            self.current_type = Type::INT; // Default to INT for now

            // Parse identifier
            if self.current_token.token_type != TokenType::Id {
                return Err(CompilerError::ParserError {
                    message: format!("Expected identifier, got {:?} at line {}", 
                        self.current_token.token_type,
                        self.lexer.line()),
                    location: None,
                    source_line: None,
                    suggestion: None,
                });
            }

            let id_name = self.current_token.name.as_ref().unwrap().clone();
            self.next_token()?;

            // Check for function declaration
            if self.current_token.token_type == TokenType::LParen {
                // Function declaration
                self.next_token()?; // Skip '('
                
                // Parse parameters
                let mut _param_count = 0;
                if self.current_token.token_type != TokenType::RParen {
                    // First parameter
                    self.parse_type()?;
                    if self.current_token.token_type != TokenType::Id {
                        return Err(CompilerError::ParserError {
                            message: format!("Expected parameter name, got {:?} at line {}", 
                                self.current_token.token_type,
                                self.lexer.line()),
                            location: None,
                            source_line: None,
                            suggestion: None,
                        });
                    }
                    let param_name = self.current_token.name.as_ref().unwrap().clone();
                    self.next_token()?;
                    
                    // Add parameter to symbol table
                    self.symbol_table.add(&param_name, TokenType::Id, self.current_type, self.local_offset);
                    self.local_offset += 8; // Each parameter takes 8 bytes
                    _param_count += 1;

                    // More parameters
                    while self.current_token.token_type == TokenType::Comma {
                        self.next_token()?; // Skip ','
                        self.parse_type()?;
                        if self.current_token.token_type != TokenType::Id {
                            return Err(CompilerError::ParserError {
                                message: format!("Expected parameter name, got {:?} at line {}", 
                                    self.current_token.token_type,
                                    self.lexer.line()),
                                location: None,
                                source_line: None,
                                suggestion: None,
                            });
                        }
                        let param_name = self.current_token.name.as_ref().unwrap().clone();
                        self.next_token()?;
                        
                        // Add parameter to symbol table
                        self.symbol_table.add(&param_name, TokenType::Id, self.current_type, self.local_offset);
                        self.local_offset += 8; // Each parameter takes 8 bytes
                        _param_count += 1;
                    }
                }

                self.match_token(TokenType::RParen)?;

                // Add function to symbol table
                let func_pos = self.emit(Opcode::JMP as i64);
                self.emit(0); // Placeholder for function address
                self.symbol_table.add(&id_name, TokenType::Fun, self.current_type, func_pos as i64);

                // Parse function body
                self.match_token(TokenType::LBrace)?;
                
                // Reset local offset for function body
                self.local_offset = 0;
                
                // Parse statements
                while self.current_token.token_type != TokenType::RBrace {
                    self.parse_statement()?;
                }
                
                self.match_token(TokenType::RBrace)?;

                // Update function address
                self.code[func_pos + 1] = self.code.len() as i64;
            } else {
                // Variable declaration
                self.symbol_table.add(&id_name, TokenType::Id, self.current_type, self.local_offset);
                self.local_offset += 8; // Each variable takes 8 bytes

                // Check for initialization
                if self.current_token.token_type == TokenType::Assign {
                    self.next_token()?; // Skip '='
                    self.parse_expression()?;
                }
            }

            // Skip semicolon
            self.match_token(TokenType::Semicolon)?;
        }

        Ok(())
    }

    /// Parse a type (int, char, etc.)
    fn parse_type(&mut self) -> Result<(), CompilerError> {
        // Default to int
        self.current_type = Type::INT;

        // Parse type keyword
        match self.current_token.token_type {
            TokenType::Int => {
                self.next_token(); // Skip 'int'
            },
            TokenType::Char => {
                self.current_type = Type::CHAR;
                self.next_token(); // Skip 'char'
            },
            TokenType::Enum => {
                self.parse_enum()?;
            },
            TokenType::Void => {
                // Skip 'void', but keep Type::INT
                self.next_token();
            },
            _ => {
                // Default to int if no type specified
            }
        }

        // Parse pointers
        while self.current_token.token_type == TokenType::Mul {
            self.current_type = self.current_type.to_ptr();
            self.next_token(); // Skip '*'
        }

        Ok(())
    }

    /// Parse an enum declaration
    fn parse_enum(&mut self) -> Result<(), CompilerError> {
        self.next_token(); // Skip 'enum'

        // Optional enum identifier
        if self.current_token.token_type == TokenType::Id {
            self.next_token(); // Skip identifier
        }

        // Enum body
        if self.current_token.token_type == TokenType::LBrace {
            self.next_token(); // Skip '{'

            let mut value = 0;

            // Parse enum members
            while self.current_token.token_type != TokenType::RBrace {
                // Member name
                if self.current_token.token_type != TokenType::Id {
                    return Err(CompilerError::ParserError {
                        message: format!("Expected identifier in enum declaration at line {}", 
                            self.lexer.line()),
                        location: None,
                        source_line: None,
                        suggestion: None,
                    });
                }

                let enum_name = self.current_token.name.as_ref().unwrap().clone();
                self.next_token(); // Skip identifier

                // Check for explicit value
                if self.current_token.token_type == TokenType::Assign {
                    self.next_token(); // Skip '='

                    if self.current_token.token_type != TokenType::Num {
                        return Err(CompilerError::ParserError {
                            message: format!("Expected numeric value after = in enum at line {}", 
                                self.lexer.line()),
                            location: None,
                            source_line: None,
                            suggestion: None,
                        });
                    }

                    value = self.current_value;
                    self.next_token(); // Skip number
                }

                // Add enum member to symbol table
                self.symbol_table.add(&enum_name, TokenType::Num, Type::INT, value);

                // Increment value for next member
                value += 1;

                // Check for comma
                if self.current_token.token_type == TokenType::Comma {
                    self.next_token(); // Skip ','
                } else {
                    break;
                }
            }

            self.match_token(TokenType::RBrace)?;
        }

        Ok(())
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<(), CompilerError> {
        match self.current_token.token_type {
            TokenType::If => {
                self.next_token()?; // Skip 'if'
                self.match_token(TokenType::LParen)?;
                self.parse_expression()?;
                self.match_token(TokenType::RParen)?;

                // Save position for jump
                let jmp_pos = self.emit(Opcode::BZ as i64);
                self.emit(0); // Placeholder for jump address

                // Parse if body
                self.match_token(TokenType::LBrace)?;
                while self.current_token.token_type != TokenType::RBrace {
                    self.parse_statement()?;
                }
                self.match_token(TokenType::RBrace)?;

                // Update jump address
                let new_code_len = self.code.len();
                self.code[jmp_pos + 1] = new_code_len as i64;
            },
            TokenType::While => {
                let loop_start = self.code.len();
                self.next_token()?; // Skip 'while'
                self.match_token(TokenType::LParen)?;
                self.parse_expression()?;
                self.match_token(TokenType::RParen)?;

                // Save position for jump
                let jmp_pos = self.emit(Opcode::BZ as i64);
                self.emit(0); // Placeholder for jump address

                // Parse while body
                self.match_token(TokenType::LBrace)?;
                while self.current_token.token_type != TokenType::RBrace {
                    self.parse_statement()?;
                }
                self.match_token(TokenType::RBrace)?;

                // Jump back to condition
                self.emit(Opcode::JMP as i64);
                self.emit(loop_start as i64);

                // Update jump address
                let new_code_len = self.code.len();
                self.code[jmp_pos + 1] = new_code_len as i64;
            },
            TokenType::Return => {
                self.next_token()?; // Skip 'return'
                self.parse_expression()?;
                self.emit(Opcode::LEV as i64);
                self.match_token(TokenType::Semicolon)?;
            },
            _ => {
                self.parse_expression()?;
                self.match_token(TokenType::Semicolon)?;
            }
        }

        Ok(())
    }

    /// Parse a unary expression
    fn parse_unary_expression(&mut self) -> Result<(), CompilerError> {
        match self.current_token.token_type {
            TokenType::Num => {
                // Numeric literal
                let value = self.current_value;
                self.next_token()?; // Skip number

                self.emit(Opcode::IMM as i64);
                self.emit(value);
            },
            TokenType::Id => {
                // Identifier (variable or function)
                let id_name = self.current_token.name.as_ref().unwrap().clone();
                self.next_token()?; // Skip identifier

                // Check for function call
                if self.current_token.token_type == TokenType::LParen {
                    self.next_token()?; // Skip '('

                    // Parse arguments
                    let mut arg_count = 0;
                    if self.current_token.token_type != TokenType::RParen {
                        // First argument
                        self.parse_expression()?;
                        self.emit(Opcode::PSH as i64);
                        arg_count += 1;
                        
                        // More arguments
                        while self.current_token.token_type == TokenType::Comma {
                            self.next_token()?; // Skip ','
                            self.parse_expression()?;
                            self.emit(Opcode::PSH as i64);
                            arg_count += 1;
                        }
                    }

                    self.match_token(TokenType::RParen)?;

                    // Look up function
                    if let Some(symbol) = self.symbol_table.get(&id_name) {
                        match symbol.class {
                            TokenType::Sys => {
                                // System call
                                self.emit(symbol.value);
                            },
                            TokenType::Fun => {
                                // User-defined function
                                let value = symbol.value;
                                self.emit(Opcode::JSR as i64);
                                self.emit(value);
                            },
                            _ => {
                                return Err(CompilerError::ParserError {
                                    message: format!("{} is not a function at line {}", 
                                        id_name, 
                                        self.lexer.line()),
                                    location: None,
                                    source_line: None,
                                    suggestion: None,
                                });
                            }
                        }
                    } else {
                        return Err(CompilerError::ParserError {
                            message: format!("Undefined function: {} at line {}", 
                                id_name, 
                                self.lexer.line()),
                            location: None,
                            source_line: None,
                            suggestion: None,
                        });
                    }

                    // Clean up arguments
                    self.emit(Opcode::ADJ as i64);
                    self.emit(arg_count as i64);
                } else {
                    // Variable
                    if let Some(symbol) = self.symbol_table.get(&id_name) {
                        match symbol.class {
                            TokenType::Id => {
                                // Local variable
                                let value = symbol.value;
                                self.emit(Opcode::LEA as i64);
                                self.emit(value);
                                self.emit(Opcode::LI as i64);
                            },
                            TokenType::Glo => {
                                // Global variable
                                let value = symbol.value;
                                self.emit(Opcode::IMM as i64);
                                self.emit(value);
                                self.emit(Opcode::LI as i64);
                            },
                            _ => {
                                return Err(CompilerError::ParserError {
                                    message: format!("Invalid variable: {} at line {}", 
                                        id_name, 
                                        self.lexer.line()),
                                    location: None,
                                    source_line: None,
                                    suggestion: None,
                                });
                            }
                        }
                    } else {
                        return Err(CompilerError::ParserError {
                            message: format!("Undefined variable: {} at line {}", 
                                id_name, 
                                self.lexer.line()),
                            location: None,
                            source_line: None,
                            suggestion: None,
                        });
                    }
                }
            },
            TokenType::Sub => {
                // Negation
                self.next_token()?; // Skip '-'
                self.parse_unary_expression()?;
                self.emit(Opcode::NEG as i64);
            },
            TokenType::Inc => {
                // Pre-increment
                self.next_token()?; // Skip '++'
                self.parse_unary_expression()?;
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                self.emit(1);
                self.emit(Opcode::ADD as i64);
                self.emit(Opcode::SI as i64);
            },
            TokenType::Dec => {
                // Pre-decrement
                self.next_token()?; // Skip '--'
                self.parse_unary_expression()?;
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                self.emit(1);
                self.emit(Opcode::SUB as i64);
                self.emit(Opcode::SI as i64);
            },
            TokenType::Mul => {
                // Dereference
                self.next_token()?; // Skip '*'
                self.parse_unary_expression()?;
                self.emit(Opcode::LI as i64);
            },
            TokenType::And => {
                // Address of
                self.next_token()?; // Skip '&'
                self.parse_unary_expression()?;
            },
            TokenType::Tilde => {
                // Bitwise NOT
                self.next_token()?; // Skip '~'
                self.parse_unary_expression()?;
                self.emit(Opcode::XOR as i64);
                self.emit(Opcode::IMM as i64);
                self.emit(-1);
            },
            TokenType::Sizeof => {
                // Sizeof operator
                self.next_token()?; // Skip 'sizeof'
                self.match_token(TokenType::LParen)?;
                
                let type_size = if self.current_token.token_type == TokenType::Int {
                    self.next_token()?; // Skip 'int'
                    Type::INT.size() as i64
                } else if self.current_token.token_type == TokenType::Char {
                    self.next_token()?; // Skip 'char'
                    Type::CHAR.size() as i64
                } else {
                    return Err(CompilerError::ParserError {
                        message: format!("Expected type in sizeof, got {:?} at line {}", 
                            self.current_token.token_type, 
                            self.lexer.line()),
                        location: None,
                        source_line: None,
                        suggestion: None,
                    });
                };

                self.match_token(TokenType::RParen)?;
                self.emit(Opcode::IMM as i64);
                self.emit(type_size);
            },
            _ => {
                return Err(CompilerError::ParserError {
                    message: format!("Unexpected token in expression: {:?} at line {}", 
                        self.current_token.token_type, 
                        self.lexer.line()),
                    location: None,
                    source_line: None,
                    suggestion: None,
                });
            }
        }

        Ok(())
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_unary_expression()?;

        while self.current_token.token_type == TokenType::Add 
            || self.current_token.token_type == TokenType::Sub
            || self.current_token.token_type == TokenType::Mul
            || self.current_token.token_type == TokenType::Div
            || self.current_token.token_type == TokenType::Mod
            || self.current_token.token_type == TokenType::Lt
            || self.current_token.token_type == TokenType::Gt
            || self.current_token.token_type == TokenType::Le
            || self.current_token.token_type == TokenType::Ge
            || self.current_token.token_type == TokenType::Eq
            || self.current_token.token_type == TokenType::Ne
            || self.current_token.token_type == TokenType::And
            || self.current_token.token_type == TokenType::Or {
            
            let op = self.current_token.token_type.clone();
            self.next_token()?;

            self.parse_unary_expression()?;

            match op {
                TokenType::Add => { self.emit(Opcode::ADD as i64); },
                TokenType::Sub => { self.emit(Opcode::SUB as i64); },
                TokenType::Mul => { self.emit(Opcode::MUL as i64); },
                TokenType::Div => { self.emit(Opcode::DIV as i64); },
                TokenType::Mod => { self.emit(Opcode::MOD as i64); },
                TokenType::Lt => { self.emit(Opcode::LT as i64); },
                TokenType::Gt => { self.emit(Opcode::GT as i64); },
                TokenType::Le => { self.emit(Opcode::LE as i64); },
                TokenType::Ge => { self.emit(Opcode::GE as i64); },
                TokenType::Eq => { self.emit(Opcode::EQ as i64); },
                TokenType::Ne => { self.emit(Opcode::NE as i64); },
                TokenType::And => { self.emit(Opcode::AND as i64); },
                TokenType::Or => { self.emit(Opcode::OR as i64); },
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    /// Get the generated bytecode
    pub fn get_code(&self) -> &[i64] {
        &self.code
    }

    /// Get the data segment
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Get the address of the main function
    pub fn get_main_function(&self) -> Option<usize> {
        self.symbol_table.get_main().map(|symbol| symbol.value as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_basic() -> Result<(), CompilerError> {
        // Basic program to parse
        let source = "int main() { return 42; }";
        let mut parser = Parser::new(source.to_string(), false);

        parser.init()?;
        parser.parse()?;

        // Check that main function is defined
        assert!(parser.get_main_function().is_some());

        // Check generated code
        let code = parser.get_code();

        // Should have JSR instructions for function calls
        let has_jsr = code.contains(&(Opcode::JSR as i64));
        assert!(has_jsr, "Expected JSR instruction for function call");

        // Should have ADJ instruction to clean up arguments
        let has_adj = code.contains(&(Opcode::ADJ as i64));
        assert!(has_adj, "Expected ADJ instruction for argument cleanup");

        Ok(())
    }

    #[test]
    fn test_parser_enum() -> Result<(), CompilerError> {
        // Test enum declarations
        let source = "
        enum Color { RED, GREEN, BLUE = 5, YELLOW };

        int main() {
        return BLUE;
        }
        ";
        let mut parser = Parser::new(source.to_string(), false);

        parser.init()?;
        parser.parse()?;

        // Should compile and return 5 (BLUE's value)
        let code = parser.get_code();
        let main_code_offset = parser.get_main_function().unwrap();

        // Check for IMM 5 instruction in main
        let mut found_5 = false;
        for i in main_code_offset..code.len() - 1 {
            if code[i] == Opcode::IMM as i64 && code[i + 1] == 5 {
                found_5 = true;
                break;
            }
        }

        assert!(found_5, "Expected code to load enum value 5");

        Ok(())
    }

    #[test]
    fn test_parser_pointers() -> Result<(), CompilerError> {
        // Test pointer operations
        let source = "
        int main() {
        int x;
        int *p;

        x = 42;
        p = &x;
        *p = 24;

        return x;
        }
        ";
        let mut parser = Parser::new(source.to_string(), false);

        parser.init()?;
        parser.parse()?;

        // Should have correct pointer operations
        let code = parser.get_code();

        // Should have LI and SI for pointer dereference
        let has_li = code.contains(&(Opcode::LI as i64));
        let has_si = code.contains(&(Opcode::SI as i64));

        assert!(has_li, "Expected LI instruction for pointer operations");
        assert!(has_si, "Expected SI instruction for pointer operations");

        // Should eventually return 24 (the new value of x)
        // This is hard to test without running the VM,
        // but we can check for IMM 24 instruction
        let mut found_24 = false;
        for i in 0..code.len() - 1 {
            if code[i] == Opcode::IMM as i64 && code[i + 1] == 24 {
                found_24 = true;
                break;
            }
        }

        assert!(found_24, "Expected code to use value 24");

        Ok(())
    }

    #[test]
    fn test_parser_arrays() -> Result<(), CompilerError> {
        // Test array operations
        let source = "
        int main() {
        int arr[3];

        arr[0] = 1;
        arr[1] = 2;
        arr[2] = 3;

        return arr[1];
        }
        ";
        let mut parser = Parser::new(source.to_string(), false);

        parser.init()?;
        parser.parse()?;

        // Check for array access instructions
        let code = parser.get_code();

        // Should use Brak (array subscript) and ADD
        let has_add = code.contains(&(Opcode::ADD as i64));
        assert!(has_add, "Expected ADD instruction for array access");

        // Should return 2 (arr[1])
        // Check for IMM 2 and IMM 1 (array index)
        let mut found_1 = false;
        let mut found_2 = false;
        for i in 0..code.len() - 1 {
            if code[i] == Opcode::IMM as i64 && code[i + 1] == 1 {
                found_1 = true;
            }
            if code[i] == Opcode::IMM as i64 && code[i + 1] == 2 {
                found_2 = true;
            }
        }

        assert!(found_1, "Expected code to use index 1");
        assert!(found_2, "Expected code to use value 2");

        Ok(())
    }

    #[test]
    fn test_parser_function_call() -> Result<(), CompilerError> {
        // Test function calls
        let source = "
        int add(int a, int b) {
            return a + b;
        }

        int main() {
            return add(2, 3);
        }
        ";
        let mut parser = Parser::new(source.to_string(), false);

        parser.init()?;
        parser.parse()?;

        // Check generated code
        let code = parser.get_code();

        // Should have JSR instructions for function calls
        let has_jsr = code.contains(&(Opcode::JSR as i64));
        assert!(has_jsr, "Expected JSR instruction for function call");

        // Should have ADJ instruction to clean up arguments
        let has_adj = code.contains(&(Opcode::ADJ as i64));
        assert!(has_adj, "Expected ADJ instruction for argument cleanup");

        Ok(())
    }
}