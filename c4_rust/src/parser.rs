use crate::error::CompilerError;
use crate::lexer::{Lexer, Token};
use crate::symbol::{Symbol, SymbolTable};
use crate::types::{Opcode, TokenType, Type};
use std::collections::HashMap;

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
            let message = format!("Expected {:?}, got {:?}", expected, self.current_token.token_type);
            let location = self.lexer.line();
            let source_line = self.lexer.get_current_line();
            
            let suggestion = match expected {
                TokenType::Semicolon => Some("Add a semicolon at the end of the statement".to_string()),
                TokenType::LBrace => Some("Add an opening brace '{'".to_string()),
                TokenType::RBrace => Some("Add a closing brace '}'".to_string()),
                TokenType::LParen => Some("Add an opening parenthesis '('".to_string()),
                TokenType::RParen => Some("Add a closing parenthesis ')'".to_string()),
                _ => None,
            };
            
            Err(CompilerError::ParserError {
                message,
                location: Some(crate::error::SourceLocation::new(location, self.lexer.column())),
                source_line: Some(source_line),
                suggestion,
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
            let base_type = self.parse_type()?;
            
            // Continue parsing declarations until we hit a semicolon or closing brace
            while self.current_token.token_type != TokenType::Semicolon && 
                  self.current_token.token_type != TokenType::RBrace {
                
                let mut ty = base_type;
                
                // Handle pointer types with multiple '*'
                while self.current_token.token_type == TokenType::Mul {
                    ty = ty.to_ptr();
                    self.next_token()?;
                }
                
                // Parse identifier
                if self.current_token.token_type != TokenType::Id {
                    return Err(CompilerError::ParserError {
                        message: format!("Expected identifier, got {:?}", self.current_token.token_type),
                        location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                        source_line: Some(self.lexer.get_current_line()),
                        suggestion: None,
                    });
                }
                
                let id_name = self.current_token.name.as_ref()
                    .ok_or_else(|| CompilerError::ParserError {
                        message: "Missing identifier name".to_string(),
                        location: None,
                        source_line: None,
                        suggestion: None,
                    })?
                    .clone();
                
                self.next_token()?;
                
                // Handle array declarations
                let mut is_array = false;
                let mut array_size = 0;
                
                if self.current_token.token_type == TokenType::Brak {
                    is_array = true;
                    self.next_token()?;
                    
                    // Parse array size
                    if self.current_token.token_type == TokenType::Num {
                        array_size = self.current_value;
                        self.next_token()?;
                    } else {
                        return Err(CompilerError::ParserError {
                            message: "Expected array size".to_string(),
                            location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                            source_line: Some(self.lexer.get_current_line()),
                            suggestion: None,
                        });
                    }
                    
                    // Close bracket
                    self.match_token(TokenType::RBracket)?;
                    
                    // Adjust type for array (in C4, arrays are just pointers)
                    ty = ty.to_ptr();
                }
                
                // Check for function declaration
                if self.current_token.token_type == TokenType::LParen {
                    // Function declaration
                    self.next_token()?; // Skip '('
                    
                    // Create new function symbol
                    let fn_addr = self.code.len();
                    self.symbol_table.add(&id_name, TokenType::Fun, ty, fn_addr as i64);
                    
                    // Enter function scope
                    self.symbol_table.enter_scope();
                    
                    // Reset local offset for parameters
                    self.local_offset = 8; // Skip return address and base pointer
                    
                    // Parse parameters
                    if self.current_token.token_type != TokenType::RParen {
                        // First parameter
                        let param_type = self.parse_type()?;
                        
                        // Handle pointer types in parameters
                        let mut ptr_type = param_type;
                        while self.current_token.token_type == TokenType::Mul {
                            ptr_type = ptr_type.to_ptr();
                            self.next_token()?;
                        }
                        
                        if self.current_token.token_type != TokenType::Id {
                            return Err(CompilerError::ParserError {
                                message: "Expected parameter name".to_string(),
                                location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                                source_line: Some(self.lexer.get_current_line()),
                                suggestion: None,
                            });
                        }
                        
                        let param_name = self.current_token.name.as_ref().unwrap().clone();
                        self.next_token()?;
                        
                        // Add parameter to symbol table (in reversed order due to stack layout)
                        self.symbol_table.add(&param_name, TokenType::Loc, ptr_type, self.local_offset);
                        self.local_offset += 8; // Each parameter takes 8 bytes
                        
                        // More parameters
                        while self.current_token.token_type == TokenType::Comma {
                            self.next_token()?; // Skip ','
                            
                            let param_type = self.parse_type()?;
                            
                            // Handle pointer types in parameters
                            let mut ptr_type = param_type;
                            while self.current_token.token_type == TokenType::Mul {
                                ptr_type = ptr_type.to_ptr();
                                self.next_token()?;
                            }
                            
                            if self.current_token.token_type != TokenType::Id {
                                return Err(CompilerError::ParserError {
                                    message: "Expected parameter name".to_string(),
                                    location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                                    source_line: Some(self.lexer.get_current_line()),
                                    suggestion: None,
                                });
                            }
                            
                            let param_name = self.current_token.name.as_ref().unwrap().clone();
                            self.next_token()?;
                            
                            // Add parameter to symbol table
                            self.symbol_table.add(&param_name, TokenType::Loc, ptr_type, self.local_offset);
                            self.local_offset += 8; // Each parameter takes 8 bytes
                        }
                    }
                    
                    self.match_token(TokenType::RParen)?;
                    
                    // Parse function body
                    self.match_token(TokenType::LBrace)?;
                    
                    // Setup stack frame
                    self.emit(Opcode::ENT as i64);
                    self.emit(0); // Placeholder for local variable space
                    
                    // Reset local offset for local variables (negative offsets from base pointer)
                    self.local_offset = -8;
                    
                    // Parse local variable declarations at the beginning of function
                    while self.current_token.token_type == TokenType::Int || 
                          self.current_token.token_type == TokenType::Char {
                        
                        let local_type = self.parse_type()?;
                        
                        // Parse all variables of this type
                        while self.current_token.token_type != TokenType::Semicolon {
                            // Handle pointer types
                            let mut ptr_type = local_type;
                            while self.current_token.token_type == TokenType::Mul {
                                ptr_type = ptr_type.to_ptr();
                                self.next_token()?;
                            }
                            
                            if self.current_token.token_type != TokenType::Id {
                                return Err(CompilerError::ParserError {
                                    message: "Expected local variable name".to_string(),
                                    location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                                    source_line: Some(self.lexer.get_current_line()),
                                    suggestion: None,
                                });
                            }
                            
                            let var_name = self.current_token.name.as_ref().unwrap().clone();
                            self.next_token()?;
                            
                            // Handle array declarations
                            if self.current_token.token_type == TokenType::Brak {
                                self.next_token()?;
                                
                                // Get array size
                                if self.current_token.token_type == TokenType::Num {
                                    let size = self.current_value;
                                    self.next_token()?;
                                    
                                    // Make space for array
                                    self.local_offset -= 8 * size;
                                    
                                    // Register as pointer type
                                    self.symbol_table.add(&var_name, TokenType::Loc, ptr_type.to_ptr(), self.local_offset);
                                } else {
                                    return Err(CompilerError::ParserError {
                                        message: "Expected array size".to_string(),
                                        location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                                        source_line: Some(self.lexer.get_current_line()),
                                        suggestion: None,
                                    });
                                }
                                
                                self.match_token(TokenType::RBracket)?;
                            } else {
                                // Regular variable
                                self.symbol_table.add(&var_name, TokenType::Loc, ptr_type, self.local_offset);
                                self.local_offset -= 8; // Each variable takes 8 bytes
                            }
                            
                            // Check for comma for multiple declarations
                            if self.current_token.token_type == TokenType::Comma {
                                self.next_token()?;
                            } else {
                                break;
                            }
                        }
                        
                        self.match_token(TokenType::Semicolon)?;
                    }
                    
                    // Update ENT instruction with local variable count
                    self.code[fn_addr + 1] = (-self.local_offset / 8) as i64;
                    
                    // Parse statements
                    while self.current_token.token_type != TokenType::RBrace {
                        self.parse_statement()?;
                    }
                    
                    // Add implicit return if none exists
                    // (In C, reaching the end of a function without a return is undefined,
                    // but in C4 we'll just return 0)
                    if self.code.last() != Some(&(Opcode::LEV as i64)) {
                        self.emit(Opcode::IMM as i64);
                        self.emit(0);
                        self.emit(Opcode::LEV as i64);
                    }
                    
                    self.match_token(TokenType::RBrace)?;
                    
                    // Exit function scope
                    self.symbol_table.exit_scope();
                } else {
                    // Global variable declaration
                    if is_array {
                        // Allocate array space
                        let var_addr = self.data.len();
                        self.data.resize(var_addr + (array_size as usize) * ty.size(), 0);
                        
                        // Register as pointer type
                        self.symbol_table.add(&id_name, TokenType::Glo, ty, var_addr as i64);
                    } else {
                        // Regular variable
                        let var_addr = self.data.len();
                        self.data.resize(var_addr + ty.size(), 0);
                        
                        self.symbol_table.add(&id_name, TokenType::Glo, ty, var_addr as i64);
                    }
                    
                    // Check for initialization
                    if self.current_token.token_type == TokenType::Assign {
                        self.next_token()?; // Skip '='
                        
                        // For now, we only support numeric initializers at global scope
                        if self.current_token.token_type != TokenType::Num {
                            return Err(CompilerError::ParserError {
                                message: "Expected numeric initializer for global variable".to_string(),
                                location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                                source_line: Some(self.lexer.get_current_line()),
                                suggestion: None,
                            });
                        }
                        
                        // Store the initializer value
                        let value = self.current_value;
                        self.next_token()?;
                        
                        // Update the data segment (crude, but works for POD types)
                        let var = self.symbol_table.get(&id_name).unwrap();
                        let addr = var.value as usize;
                        
                        match ty {
                            Type::CHAR => {
                                if addr < self.data.len() {
                                    self.data[addr] = value as u8;
                                }
                            },
                            _ => {
                                // For INT and PTR, store as 64-bit value
                                if addr + 8 <= self.data.len() {
                                    let bytes = value.to_le_bytes();
                                    for i in 0..8 {
                                        self.data[addr + i] = bytes[i];
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Check for comma for multiple declarations
                if self.current_token.token_type == TokenType::Comma {
                    self.next_token()?;
                } else {
                    break;
                }
            }
            
            // Skip semicolon
            if self.current_token.token_type == TokenType::Semicolon {
                self.next_token()?;
            } else if self.current_token.token_type != TokenType::RBrace && 
                      self.current_token.token_type != TokenType::Eof {
                return Err(CompilerError::ParserError {
                    message: "Expected semicolon after declaration".to_string(),
                    location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                    source_line: Some(self.lexer.get_current_line()),
                    suggestion: Some("Add a semicolon at the end of the declaration".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Parse a type (int, char, etc.)
    fn parse_type(&mut self) -> Result<Type, CompilerError> {
        // Default to int
        let mut typ = Type::INT;

        // Parse type keyword
        match self.current_token.token_type {
            TokenType::Int => {
                typ = Type::INT;
                self.next_token()?; // Skip 'int'
            },
            TokenType::Char => {
                typ = Type::CHAR;
                self.next_token()?; // Skip 'char'
            },
            TokenType::Enum => {
                self.parse_enum()?;
                typ = Type::INT; // Enum values are integers
            },
            TokenType::Void => {
                // Void is just INT with a special flag
                typ = Type::INT;
                self.next_token()?; // Skip 'void'
            },
            _ => {
                // Return default INT type if no type specified
            }
        }

        Ok(typ)
    }

    /// Parse an enum declaration
    fn parse_enum(&mut self) -> Result<(), CompilerError> {
        self.next_token()?; // Skip 'enum'

        // Optional enum identifier
        if self.current_token.token_type == TokenType::Id {
            self.next_token()?; // Skip identifier
        }

        // Enum body
        if self.current_token.token_type == TokenType::LBrace {
            self.next_token()?; // Skip '{'

            let mut value = 0;

            // Parse enum members
            while self.current_token.token_type != TokenType::RBrace {
                // Member name
                if self.current_token.token_type != TokenType::Id {
                    return Err(CompilerError::ParserError {
                        message: "Expected identifier in enum declaration".to_string(),
                        location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                        source_line: Some(self.lexer.get_current_line()),
                        suggestion: None,
                    });
                }

                let enum_name = self.current_token.name.as_ref().unwrap().clone();
                self.next_token()?; // Skip identifier

                // Check for explicit value
                if self.current_token.token_type == TokenType::Assign {
                    self.next_token()?; // Skip '='

                    if self.current_token.token_type != TokenType::Num {
                        return Err(CompilerError::ParserError {
                            message: "Expected numeric value after = in enum".to_string(),
                            location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                            source_line: Some(self.lexer.get_current_line()),
                            suggestion: None,
                        });
                    }

                    value = self.current_value;
                    self.next_token()?; // Skip number
                }

                // Add enum member to symbol table
                self.symbol_table.add(&enum_name, TokenType::Num, Type::INT, value);

                // Increment value for next member
                value += 1;

                // Check for comma
                if self.current_token.token_type == TokenType::Comma {
                    self.next_token()?; // Skip ','
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

                // Emit branch if zero
                let jz_addr = self.emit(Opcode::BZ as i64);
                self.emit(0); // Placeholder for jump address

                // Parse 'if' body
                self.parse_statement()?;

                // Check for 'else'
                let else_jump_addr = if self.current_token.token_type == TokenType::Else {
                    self.next_token()?; // Skip 'else'
                    
                    // Jump over else block
                    let jmp_addr = self.emit(Opcode::JMP as i64);
                    self.emit(0); // Placeholder for jump address
                    
                    // Update the 'if' branch target to jump to the 'else' part
                    self.code[jz_addr + 1] = self.code.len() as i64;
                    
                    // Parse 'else' body
                    self.parse_statement()?;
                    
                    // Return the address of the jump after the 'if' block
                    Some(jmp_addr)
                } else {
                    // No 'else' part, update branch target to jump to here
                    self.code[jz_addr + 1] = self.code.len() as i64;
                    None
                };

                // If we had an else, update the jump at the end of 'if' block
                if let Some(addr) = else_jump_addr {
                    self.code[addr + 1] = self.code.len() as i64;
                }
            },
            TokenType::While => {
                self.next_token()?; // Skip 'while'
                
                // Save the start address for loop condition
                let loop_start = self.code.len();
                
                self.match_token(TokenType::LParen)?;
                self.parse_expression()?;
                self.match_token(TokenType::RParen)?;

                // Emit branch if zero
                let jz_addr = self.emit(Opcode::BZ as i64);
                self.emit(0); // Placeholder for jump address

                // Parse while body
                self.parse_statement()?;

                // Jump back to condition
                self.emit(Opcode::JMP as i64);
                self.emit(loop_start as i64);

                // Update branch target
                self.code[jz_addr + 1] = self.code.len() as i64;
            },
            TokenType::Return => {
                self.next_token()?; // Skip 'return'
                
                // Parse return value (if any)
                if self.current_token.token_type != TokenType::Semicolon {
                    self.parse_expression()?;
                } else {
                    // Implicit return 0
                    self.emit(Opcode::IMM as i64);
                    self.emit(0);
                }
                
                // Return from function
                self.emit(Opcode::LEV as i64);
                self.match_token(TokenType::Semicolon)?;
            },
            TokenType::LBrace => {
                self.next_token()?; // Skip '{'
                
                // Parse all statements in the block
                while self.current_token.token_type != TokenType::RBrace && 
                      self.current_token.token_type != TokenType::Eof {
                    self.parse_statement()?;
                }
                
                self.match_token(TokenType::RBrace)?;
            },
            TokenType::Semicolon => {
                // Empty statement
                self.next_token()?;
            },
            _ => {
                // Expression statement
                self.parse_expression()?;
                self.match_token(TokenType::Semicolon)?;
                
                // Discard the expression result
                // (in C, expression statement results are discarded)
                // In some cases we need to adjust the stack
                if self.code.last() == Some(&(Opcode::PSH as i64)) {
                    self.code.pop(); // Remove the PSH instruction
                }
            }
        }

        Ok(())
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_assignment_expression()
    }

    /// Parse an assignment expression
    fn parse_assignment_expression(&mut self) -> Result<(), CompilerError> {
        // First, check if this is an assignment
        if self.current_token.token_type == TokenType::Id {
            let var_name = self.current_token.name.as_ref().unwrap().clone();
            
            // Get variable info first
            let var = self.symbol_table.get(&var_name).cloned();
            
            // Look ahead to see if the next token is '='
            let is_assignment = match self.peek_next_token()? {
                TokenType::Assign => true,
                _ => false,
            };
            
            if is_assignment {
                // This is an assignment expression
                self.next_token()?; // Skip identifier
                self.next_token()?; // Skip '='
                
                if let Some(var) = var {
                    // Load variable address
                    match var.class {
                        TokenType::Loc => {
                            self.emit(Opcode::LEA as i64);
                            self.emit(var.value);
                        },
                        TokenType::Glo => {
                            self.emit(Opcode::IMM as i64);
                            self.emit(var.value);
                        },
                        _ => {
                            return Err(CompilerError::ParserError {
                                message: format!("Cannot assign to {}", var_name),
                                location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                                source_line: Some(self.lexer.get_current_line()),
                                suggestion: None,
                            });
                        }
                    }
                    
                    // Push address to stack
                    self.emit(Opcode::PSH as i64);
                    
                    // Parse the right-hand side
                    self.parse_expression()?;
                    
                    // Store the result
                    if var.typ == Type::CHAR {
                        self.emit(Opcode::SC as i64);
                    } else {
                        self.emit(Opcode::SI as i64);
                    }
                    
                    return Ok(());
                }
            }
        }
        
        // Not an assignment, parse as conditional expression
        self.parse_logical_or_expression()
    }

    /// Parse a logical OR expression
    fn parse_logical_or_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_logical_and_expression()?;
        
        while self.current_token.token_type == TokenType::Lor {
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            
            // Mark location for jump if true
            let jnz_addr = self.emit(Opcode::BNZ as i64);
            self.emit(0); // Placeholder for jump address
            
            // Evaluate right side of OR
            self.parse_logical_and_expression()?;
            
            // Update jump address to current location
            self.code[jnz_addr + 1] = self.code.len() as i64;
        }
        
        Ok(())
    }
    
    /// Parse a logical AND expression
    fn parse_logical_and_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_bitwise_or_expression()?;
        
        while self.current_token.token_type == TokenType::Lan {
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            
            // Mark location for jump if false
            let jz_addr = self.emit(Opcode::BZ as i64);
            self.emit(0); // Placeholder for jump address
            
            // Evaluate right side of AND
            self.parse_bitwise_or_expression()?;
            
            // Update jump address to current location
            self.code[jz_addr + 1] = self.code.len() as i64;
        }
        
        Ok(())
    }
    
    /// Parse a bitwise OR expression
    fn parse_bitwise_or_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_bitwise_xor_expression()?;
        
        while self.current_token.token_type == TokenType::Or {
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            self.parse_bitwise_xor_expression()?;
            self.emit(Opcode::OR as i64);
        }
        
        Ok(())
    }
    
    /// Parse a bitwise XOR expression
    fn parse_bitwise_xor_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_bitwise_and_expression()?;
        
        while self.current_token.token_type == TokenType::Xor {
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            self.parse_bitwise_and_expression()?;
            self.emit(Opcode::XOR as i64);
        }
        
        Ok(())
    }
    
    /// Parse a bitwise AND expression
    fn parse_bitwise_and_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_equality_expression()?;
        
        while self.current_token.token_type == TokenType::And {
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            self.parse_equality_expression()?;
            self.emit(Opcode::AND as i64);
        }
        
        Ok(())
    }
    
    /// Parse an equality expression
    fn parse_equality_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_relational_expression()?;
        
        while self.current_token.token_type == TokenType::Eq || 
              self.current_token.token_type == TokenType::Ne {
            let op = self.current_token.token_type;
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            self.parse_relational_expression()?;
            
            match op {
                TokenType::Eq => { self.emit(Opcode::EQ as i64); },
                TokenType::Ne => { self.emit(Opcode::NE as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Parse a relational expression
    fn parse_relational_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_shift_expression()?;
        
        while self.current_token.token_type == TokenType::Lt || 
              self.current_token.token_type == TokenType::Gt ||
              self.current_token.token_type == TokenType::Le ||
              self.current_token.token_type == TokenType::Ge {
            let op = self.current_token.token_type;
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            self.parse_shift_expression()?;
            
            match op {
                TokenType::Lt => { self.emit(Opcode::LT as i64); },
                TokenType::Gt => { self.emit(Opcode::GT as i64); },
                TokenType::Le => { self.emit(Opcode::LE as i64); },
                TokenType::Ge => { self.emit(Opcode::GE as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Parse a shift expression
    fn parse_shift_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_additive_expression()?;
        
        while self.current_token.token_type == TokenType::Shl || 
              self.current_token.token_type == TokenType::Shr {
            let op = self.current_token.token_type;
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            self.parse_additive_expression()?;
            
            match op {
                TokenType::Shl => { self.emit(Opcode::SHL as i64); },
                TokenType::Shr => { self.emit(Opcode::SHR as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Parse an additive expression
    fn parse_additive_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_multiplicative_expression()?;
        
        while self.current_token.token_type == TokenType::Add || 
              self.current_token.token_type == TokenType::Sub {
            let op = self.current_token.token_type;
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            self.parse_multiplicative_expression()?;
            
            match op {
                TokenType::Add => { self.emit(Opcode::ADD as i64); },
                TokenType::Sub => { self.emit(Opcode::SUB as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Parse a multiplicative expression
    fn parse_multiplicative_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_unary_expression()?;
        
        while self.current_token.token_type == TokenType::Mul || 
              self.current_token.token_type == TokenType::Div ||
              self.current_token.token_type == TokenType::Mod {
            let op = self.current_token.token_type;
            self.emit(Opcode::PSH as i64);
            self.next_token()?;
            self.parse_unary_expression()?;
            
            match op {
                TokenType::Mul => { self.emit(Opcode::MUL as i64); },
                TokenType::Div => { self.emit(Opcode::DIV as i64); },
                TokenType::Mod => { self.emit(Opcode::MOD as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }

    /// Parse a unary expression
    fn parse_unary_expression(&mut self) -> Result<(), CompilerError> {
        match self.current_token.token_type {
            TokenType::Add => {
                self.next_token()?;
                self.parse_primary_expression()
            },
            TokenType::Sub => {
                self.next_token()?;
                self.parse_primary_expression()?;
                self.emit(Opcode::NEG as i64);
                Ok(())
            },
            TokenType::Tilde => {
                self.next_token()?;
                self.parse_primary_expression()?;
                self.emit(Opcode::XOR as i64);
                self.emit(-1);
                Ok(())
            },
            _ => self.parse_primary_expression()
        }
    }

    /// Parse a primary expression
    fn parse_primary_expression(&mut self) -> Result<(), CompilerError> {
        match self.current_token.token_type {
            TokenType::Num => {
                self.emit(Opcode::IMM as i64);
                self.emit(self.current_value);
                self.next_token()?;
                Ok(())
            },
            TokenType::Id => {
                let id_name = self.current_token.name.as_ref().unwrap().clone();
                self.next_token()?;
                
                if self.current_token.token_type == TokenType::LParen {
                    // Function call
                    self.next_token()?;
                    
                    // Parse arguments
                    let mut arg_count = 0;
                    if self.current_token.token_type != TokenType::RParen {
                        loop {
                            self.parse_expression()?;
                            arg_count += 1;
                            
                            if self.current_token.token_type == TokenType::RParen {
                                break;
                            }
                            
                            self.match_token(TokenType::Comma)?;
                        }
                    }
                    
                    self.match_token(TokenType::RParen)?;
                    
                    // Call function
                    if let Some(sym) = self.symbol_table.get(&id_name).cloned() {
                        self.emit(Opcode::JSR as i64);
                        self.emit(sym.value);
                        self.emit(arg_count as i64);
                    } else {
                        return Err(CompilerError::ParserError {
                            message: format!("Undefined function: {}", id_name),
                            location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                            source_line: Some(self.lexer.get_current_line()),
                            suggestion: None,
                        });
                    }
                } else {
                    // Variable
                    if let Some(sym) = self.symbol_table.get(&id_name).cloned() {
                        match sym.class {
                            TokenType::Loc => {
                                self.emit(Opcode::LEA as i64);
                                self.emit(sym.value);
                                if sym.typ == Type::CHAR {
                                    self.emit(Opcode::LC as i64);
                                } else {
                                    self.emit(Opcode::LI as i64);
                                }
                            },
                            TokenType::Glo => {
                                self.emit(Opcode::IMM as i64);
                                self.emit(sym.value);
                                if sym.typ == Type::CHAR {
                                    self.emit(Opcode::LC as i64);
                                } else {
                                    self.emit(Opcode::LI as i64);
                                }
                            },
                            _ => {
                                return Err(CompilerError::ParserError {
                                    message: format!("Invalid symbol type: {:?}", sym.class),
                                    location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                                    source_line: Some(self.lexer.get_current_line()),
                                    suggestion: None,
                                });
                            }
                        }
                    } else {
                        return Err(CompilerError::ParserError {
                            message: format!("Undefined variable: {}", id_name),
                            location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                            source_line: Some(self.lexer.get_current_line()),
                            suggestion: None,
                        });
                    }
                }
                Ok(())
            },
            TokenType::LParen => {
                self.next_token()?;
                self.parse_expression()?;
                self.match_token(TokenType::RParen)?;
                Ok(())
            },
            _ => {
                Err(CompilerError::ParserError {
                    message: format!("Unexpected token: {:?}", self.current_token.token_type),
                    location: Some(crate::error::SourceLocation::new(self.lexer.line(), self.lexer.column())),
                    source_line: Some(self.lexer.get_current_line()),
                    suggestion: None,
                })
            }
        }
    }

    /// Peek at the next token without consuming it
    fn peek_next_token(&mut self) -> Result<TokenType, CompilerError> {
        let current = self.current_token.clone();
        let next = self.lexer.next_token()?;
        self.current_token = current;
        Ok(next.token_type)
    }

    /// Get the main function symbol if it exists
    pub fn get_main_function(&self) -> Option<&Symbol> {
        self.symbol_table.get("main")
    }

    /// Get the code segment
    pub fn get_code(&self) -> &[i64] {
        &self.code
    }

    /// Get the data segment
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}