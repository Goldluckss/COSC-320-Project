use crate::error::CompilerError;
use crate::lexer::{Lexer, Token};
use crate::symbol::SymbolTable;
use crate::types::{Opcode, TokenType, Type};

/// Parser for C4 compiler
/// 
/// The parser transforms tokens from the lexer into bytecode
/// and manages the symbol table.
pub struct Parser {
    lexer: Lexer,
    code: Vec<i64>,
    current_token: Token,
    symbol_table: SymbolTable,
    data_segment: Vec<u8>,
    
    // Current parsing state
    current_type: Type,
    current_id_name: Option<String>,
    _current_value: i64,
    
    // Local variable offsets
    local_offset: i64,
    
    // Print source flag
    _print_source: bool,
}

impl Parser {
    /// Create a new parser
    pub fn new(source: String, print_source: bool) -> Self {
        Parser {
            lexer: Lexer::new(source, print_source),
            code: Vec::new(),
            current_token: Token {
                token_type: TokenType::Eof,
                value: None,
                name: None,
            },
            symbol_table: SymbolTable::new(),
            data_segment: Vec::new(),
            current_type: Type::INT,
            current_id_name: None,
            _current_value: 0,
            local_offset: 0,
            _print_source: print_source,
        }
    }
    
    /// Initialize parser
    pub fn init(&mut self) -> Result<(), CompilerError> {
        // Initialize system functions
        self.init_system_functions();
        
        // Get the first token
        self.next_token()?;
        
        Ok(())
    }
    
    /// Initialize system function symbols (printf, malloc, etc.)
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
    }
    
    /// Parse the source code
    pub fn parse(&mut self) -> Result<(), CompilerError> {
        // Parse global declarations
        self.parse_declarations()?;
        
        // Check for main function
        if self.get_main_function().is_none() {
            return Err(CompilerError::ParserError("main() not defined".to_string()));
        }
        
        Ok(())
    }
    
    /// Get the generated code
    pub fn get_code(&self) -> &[i64] {
        &self.code
    }
    
    /// Get the data segment
    pub fn get_data(&self) -> &[u8] {
        &self.data_segment
    }
    
    /// Get the main function address
    pub fn get_main_function(&self) -> Option<usize> {
        self.symbol_table.get_main().map(|sym| sym.value as usize)
    }
    
    /// Get the next token from lexer
    fn next_token(&mut self) -> Result<(), CompilerError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }
    
    /// Check if current token matches expected, then advance
    fn match_token(&mut self, expected: TokenType) -> Result<(), CompilerError> {
        if self.current_token.token_type == expected {
            self.next_token()?;
            Ok(())
        } else {
            Err(CompilerError::ParserError(
                format!("Expected {:?}, got {:?}", expected, self.current_token.token_type)
            ))
        }
    }
    
    /// Emit bytecode for an operation
    fn emit(&mut self, code: i64) -> usize {
        let pos = self.code.len();
        self.code.push(code);
        pos
    }
    
    /// Parse declarations (variables and functions)
    fn parse_declarations(&mut self) -> Result<(), CompilerError> {
        while self.current_token.token_type != TokenType::Eof {
            // Parse type
            self.parse_type()?;
            
            // Parse variables or functions
            while self.current_token.token_type != TokenType::Semicolon &&
                  self.current_token.token_type != TokenType::Eof {
                
                // Parse identifier
                if self.current_token.token_type != TokenType::Id {
                    return Err(CompilerError::ParserError(
                        format!("Expected identifier, got {:?}", self.current_token.token_type)
                    ));
                }
                
                // Save identifier name
                self.current_id_name = self.current_token.name.clone();
                self.next_token()?;
                
                // Check for function or variable
                if self.current_token.token_type == TokenType::LParen {
                    self.parse_function()?;
                } else {
                    self.parse_global_variable()?;
                    
                    // Check for multiple variables
                    if self.current_token.token_type == TokenType::Comma {
                        self.next_token()?;
                        continue;
                    }
                }
                
                // Check for semicolon after declarations
                if self.current_token.token_type == TokenType::Semicolon {
                    self.next_token()?;
                    break;
                }
                
                // Check for end of declarations
                if self.current_token.token_type == TokenType::RBrace {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse a type (int, char, etc.)
    fn parse_type(&mut self) -> Result<(), CompilerError> {
        // Set default type
        self.current_type = Type::INT;
        
        match self.current_token.token_type {
            TokenType::Int => {
                self.next_token()?;
            },
            TokenType::Char => {
                self.current_type = Type::CHAR;
                self.next_token()?;
            },
            TokenType::Enum => {
                self.parse_enum()?;
            },
            _ => {
                // Default to int if no type specified
            }
        }
        
        // Parse pointer types
        while self.current_token.token_type == TokenType::Mul {
            self.current_type = self.current_type.to_ptr();
            self.next_token()?;
        }
        
        Ok(())
    }
    
    /// Parse an enum declaration
    fn parse_enum(&mut self) -> Result<(), CompilerError> {
        // Skip 'enum' token
        self.next_token()?;
        
        // Optional enum identifier
        if self.current_token.token_type == TokenType::Id {
            self.next_token()?;
        }
        
        // Check for enum body
        if self.current_token.token_type != TokenType::LBrace {
            return Err(CompilerError::ParserError(
                format!("Expected {{ after enum, got {:?}", self.current_token.token_type)
            ));
        }
        self.next_token()?;
        
        // Parse enum values
        let mut value = 0;
        while self.current_token.token_type != TokenType::RBrace {
            // Check for identifier
            if self.current_token.token_type != TokenType::Id {
                return Err(CompilerError::ParserError(
                    format!("Expected identifier in enum, got {:?}", self.current_token.token_type)
                ));
            }
            
            // Get identifier name
            let id_name = self.current_token.name.clone().unwrap();
            self.next_token()?;
            
            // Check for value assignment
            if self.current_token.token_type == TokenType::Assign {
                self.next_token()?;
                
                if self.current_token.token_type != TokenType::Num {
                    return Err(CompilerError::ParserError(
                        format!("Expected number after =, got {:?}", self.current_token.token_type)
                    ));
                }
                
                value = self.current_token.value.unwrap();
                self.next_token()?;
            }
            
            // Add enum value to symbol table
            self.symbol_table.add(&id_name, TokenType::Num, Type::INT, value);
            
            // Increment value for next enum
            value += 1;
            
            // Check for comma
            if self.current_token.token_type == TokenType::Comma {
                self.next_token()?;
            }
        }
        
        // Skip closing brace
        self.next_token()?;
        
        Ok(())
    }
    
    /// Parse a function declaration
    fn parse_function(&mut self) -> Result<(), CompilerError> {
        // Get function name
        let func_name = self.current_id_name.clone().unwrap();
        
        // Add function to symbol table
        let func_addr = self.code.len() as i64;
        self.symbol_table.add(&func_name, TokenType::Fun, self.current_type, func_addr);
        
        // Parse parameters
        self.match_token(TokenType::LParen)?;
        
        // Enter function scope
        self.symbol_table.enter_scope();
        self.local_offset = 0;
        
        // Parse parameter list
        if self.current_token.token_type != TokenType::RParen {
            self.parse_parameters()?;
        }
        
        self.match_token(TokenType::RParen)?;
        
        // Function body
        self.match_token(TokenType::LBrace)?;
        
        // Reserve space for function prologue
        let prologue_pos = self.emit(Opcode::ENT as i64);
        self.emit(0); // Placeholder for local variable count
        
        // Parse local variable declarations
        while self.current_token.token_type == TokenType::Int || 
              self.current_token.token_type == TokenType::Char {
            self.parse_local_variables()?;
        }
        
        // Update local variable count
        self.code[prologue_pos + 1] = self.local_offset;
        
        // Parse statements
        while self.current_token.token_type != TokenType::RBrace {
            self.parse_statement()?;
        }
        
        // Function epilogue
        self.emit(Opcode::LEV as i64);
        
        // Exit function scope
        self.symbol_table.exit_scope();
        
        // Skip closing brace
        self.next_token()?;
        
        Ok(())
    }
    
    /// Parse function parameters
    fn parse_parameters(&mut self) -> Result<(), CompilerError> {
        loop {
            // Parse parameter type
            let param_type = if self.current_token.token_type == TokenType::Int {
                self.next_token()?;
                Type::INT
            } else if self.current_token.token_type == TokenType::Char {
                self.next_token()?;
                Type::CHAR
            } else {
                return Err(CompilerError::ParserError(
                    format!("Expected type in parameter list, got {:?}", self.current_token.token_type)
                ));
            };
            
            // Parse pointers
            let mut param_type = param_type;
            while self.current_token.token_type == TokenType::Mul {
                param_type = param_type.to_ptr();
                self.next_token()?;
            }
            
            // Parse parameter name
            if self.current_token.token_type != TokenType::Id {
                return Err(CompilerError::ParserError(
                    format!("Expected identifier in parameter list, got {:?}", self.current_token.token_type)
                ));
            }
            
            // Get parameter name
            let param_name = self.current_token.name.clone().unwrap();
            self.next_token()?;
            
            // Add parameter to symbol table
            // Parameters are stored in reverse order on stack, with bp pointing to the old bp
            // bp+0: old bp, bp+1: return address, bp+2: first param, ...
            self.local_offset += 1;
            self.symbol_table.add(&param_name, TokenType::Loc, param_type, self.local_offset);
            
            // Check for more parameters
            if self.current_token.token_type != TokenType::Comma {
                break;
            }
            
            self.next_token()?;
        }
        
        Ok(())
    }
    
    /// Parse global variable declaration
    fn parse_global_variable(&mut self) -> Result<(), CompilerError> {
        // Get variable name
        let var_name = self.current_id_name.clone().unwrap();
        
        // Check for array
        let mut size = 1;
        if self.current_token.token_type == TokenType::Brak {
            self.next_token()?;
            
            if self.current_token.token_type != TokenType::Num {
                return Err(CompilerError::ParserError(
                    format!("Expected array size, got {:?}", self.current_token.token_type)
                ));
            }
            
            size = self.current_token.value.unwrap() as usize;
            self.next_token()?;
            
            self.match_token(TokenType::RBracket)?;
        }
        
        // Calculate data size
        let data_size = size * self.current_type.size();
        
        // Add global variable to symbol table
        let data_addr = self.data_segment.len() as i64;
        self.symbol_table.add(&var_name, TokenType::Glo, self.current_type, data_addr);
        
        // Extend data segment
        self.data_segment.resize(self.data_segment.len() + data_size, 0);
        
        // Check for initialization
        if self.current_token.token_type == TokenType::Assign {
            self.next_token()?;
            
            // Parse initializer
            if self.current_token.token_type == TokenType::Num {
                // Initialize with number
                let value = self.current_token.value.unwrap();
                
                // Store value in data segment
                if self.current_type == Type::CHAR {
                    self.data_segment[data_addr as usize] = value as u8;
                } else {
                    // Store as little-endian integer
                    for i in 0..std::mem::size_of::<i64>() {
                        if (data_addr as usize) + i < self.data_segment.len() {
                            self.data_segment[data_addr as usize + i] = ((value >> (i * 8)) & 0xFF) as u8;
                        }
                    }
                }
                
                self.next_token()?;
            } else if self.current_token.token_type == TokenType::Str {
                // Initialize with string
                let string_content = self.current_token.name.clone().unwrap();
                
                // Copy string to data segment
                for (i, &byte) in string_content.as_bytes().iter().enumerate() {
                    if (data_addr as usize) + i < self.data_segment.len() {
                        self.data_segment[data_addr as usize + i] = byte;
                    }
                }
                
                // Add null terminator
                if (data_addr as usize) + string_content.len() < self.data_segment.len() {
                    self.data_segment[data_addr as usize + string_content.len()] = 0;
                }
                
                self.next_token()?;
            }
        }
        
        Ok(())
    }
    
    /// Parse local variable declarations
    fn parse_local_variables(&mut self) -> Result<(), CompilerError> {
        // Parse type
        self.parse_type()?;
        
        // Parse variable list
        loop {
            // Parse variable name
            if self.current_token.token_type != TokenType::Id {
                return Err(CompilerError::ParserError(
                    format!("Expected identifier, got {:?}", self.current_token.token_type)
                ));
            }
            
            // Get variable name
            let var_name = self.current_token.name.clone().unwrap();
            self.next_token()?;
            
            // Check for array
            let mut size = 1;
            if self.current_token.token_type == TokenType::Brak {
                self.next_token()?;
                
                if self.current_token.token_type != TokenType::Num {
                    return Err(CompilerError::ParserError(
                        format!("Expected array size, got {:?}", self.current_token.token_type)
                    ));
                }
                
                size = self.current_token.value.unwrap() as usize;
                self.next_token()?;
                
                self.match_token(TokenType::RBracket)?;
            }
            
            // Add local variable to symbol table, with negative offset
            self.local_offset += size as i64;
            self.symbol_table.add(&var_name, TokenType::Loc, self.current_type, -self.local_offset);
            
            // Check for initialization
            if self.current_token.token_type == TokenType::Assign {
                self.next_token()?;
                
                // Parse expression for initialization
                self.parse_expression()?;
                
                // Generate code to store value
                self.emit(Opcode::LEA as i64);
                self.emit(-self.local_offset);
                self.emit(Opcode::SI as i64);
            }
            
            // Check for more variables
            if self.current_token.token_type != TokenType::Comma {
                break;
            }
            
            self.next_token()?;
        }
        
        // Skip semicolon
        self.match_token(TokenType::Semicolon)?;
        
        Ok(())
    }
    
    /// Parse a statement
    fn parse_statement(&mut self) -> Result<(), CompilerError> {
        match self.current_token.token_type {
            TokenType::If => self.parse_if_statement()?,
            TokenType::While => self.parse_while_statement()?,
            TokenType::Return => self.parse_return_statement()?,
            TokenType::LBrace => self.parse_block()?,
            TokenType::Semicolon => {
                // Empty statement
                self.next_token()?;
            },
            _ => {
                // Expression statement
                self.parse_expression_statement()?;
            }
        }
        
        Ok(())
    }
    
    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<(), CompilerError> {
        // Skip 'if' token
        self.next_token()?;
        
        // Parse condition
        self.match_token(TokenType::LParen)?;
        self.parse_expression()?;
        self.match_token(TokenType::RParen)?;
        
        // Generate code for condition
        let _jump_false_pos = self.emit(Opcode::BZ as i64);
        let jump_false_placeholder = self.emit(0);
        
        // Parse if body
        self.parse_statement()?;
        
        // Check for else
        if self.current_token.token_type == TokenType::Else {
            self.next_token()?;
            
            // Generate jump for if body
            let _jump_end_pos = self.emit(Opcode::JMP as i64);
            let jump_end_placeholder = self.emit(0);
            
            // Update false jump position
            self.code[jump_false_placeholder] = self.code.len() as i64;
            
            // Parse else body
            self.parse_statement()?;
            
            // Update end jump position
            self.code[jump_end_placeholder] = self.code.len() as i64;
        } else {
            // No else, just update the false jump
            self.code[jump_false_placeholder] = self.code.len() as i64;
        }
        
        Ok(())
    }
    
    /// Parse a while statement
    fn parse_while_statement(&mut self) -> Result<(), CompilerError> {
        // Skip 'while' token
        self.next_token()?;
        
        // Remember loop start position
        let loop_start = self.code.len() as i64;
        
        // Parse condition
        self.match_token(TokenType::LParen)?;
        self.parse_expression()?;
        self.match_token(TokenType::RParen)?;
        
        // Generate code for condition
        let _jump_false_pos = self.emit(Opcode::BZ as i64);
        let jump_false_placeholder = self.emit(0);
        
        // Parse loop body
        self.parse_statement()?;
        
        // Jump back to loop start
        self.emit(Opcode::JMP as i64);
        self.emit(loop_start);
        
        // Update false jump position
        self.code[jump_false_placeholder] = self.code.len() as i64;
        
        Ok(())
    }
    
    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<(), CompilerError> {
        // Skip 'return' token
        self.next_token()?;
        
        // Check for expression
        if self.current_token.token_type != TokenType::Semicolon {
            self.parse_expression()?;
        } else {
            // No expression, return 0
            self.emit(Opcode::IMM as i64);
            self.emit(0);
        }
        
        // Generate return
        self.emit(Opcode::LEV as i64);
        
        // Skip semicolon
        self.match_token(TokenType::Semicolon)?;
        
        Ok(())
    }
    
    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<(), CompilerError> {
        // Skip opening brace
        self.match_token(TokenType::LBrace)?;
        
        // Enter a new scope
        self.symbol_table.enter_scope();
        
        // Parse statements
        while self.current_token.token_type != TokenType::RBrace && 
              self.current_token.token_type != TokenType::Eof {
            self.parse_statement()?;
        }
        
        // Exit scope
        self.symbol_table.exit_scope();
        
        // Skip closing brace
        self.match_token(TokenType::RBrace)?;
        
        Ok(())
    }
    
    /// Parse an expression statement
    fn parse_expression_statement(&mut self) -> Result<(), CompilerError> {
        // Parse expression
        self.parse_expression()?;
        
        // Discard the result
        self.emit(Opcode::ADJ as i64);
        self.emit(1); // Pop one value
        
        // Skip semicolon
        self.match_token(TokenType::Semicolon)?;
        
        Ok(())
    }
    
    /// Parse an expression
    fn parse_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_assignment()
    }
    
    /// Parse an assignment expression
    fn parse_assignment(&mut self) -> Result<(), CompilerError> {
        // Parse the left side of the assignment
        if self.current_token.token_type == TokenType::Id {
            // Check if this is a variable
            let id_name = self.current_token.name.clone().unwrap();
            
            if let Some(symbol) = self.symbol_table.get(&id_name) {
                self.next_token()?;
                
                // Check for assignment
                if self.current_token.token_type == TokenType::Assign {
                    self.next_token()?;
                    
                    // Generate address for the variable
                    match symbol.class {
                        TokenType::Glo => {
                            self.emit(Opcode::IMM as i64);
                            self.emit(symbol.value);
                        },
                        TokenType::Loc => {
                            self.emit(Opcode::LEA as i64);
                            self.emit(symbol.value);
                        },
                        _ => {
                            return Err(CompilerError::ParserError(
                                format!("Cannot assign to {}", id_name)
                            ));
                        }
                    }
                    
                    // Push address to stack
                    self.emit(Opcode::PSH as i64);
                    
                    // Parse right side of assignment
                    self.parse_assignment()?;
                    
                    // Generate store instruction
                    if symbol.typ == Type::CHAR {
                        self.emit(Opcode::SC as i64);
                    } else {
                        self.emit(Opcode::SI as i64);
                    }
                    
                    return Ok(());
                }
                
                // Not an assignment, backtrack
                self.current_token = Token {
                    token_type: TokenType::Id,
                    name: Some(id_name),
                    value: None,
                };
            }
        }
        
        // Not an assignment, parse logical OR expression
        self.parse_logical_or()
    }
    
    /// Parse logical OR expression (||)
    fn parse_logical_or(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_logical_and()?;
        
        // Parse || operators
        while self.current_token.token_type == TokenType::Lor {
            self.next_token()?;
            
            // Generate short-circuit evaluation
            let _jump_true_pos = self.emit(Opcode::BNZ as i64);
            let jump_true_placeholder = self.emit(0);
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_logical_and()?;
            
            // Perform OR operation
            self.emit(Opcode::OR as i64);
            
            // Update jump position
            self.code[jump_true_placeholder] = self.code.len() as i64;
        }
        
        Ok(())
    }
    
    /// Parse logical AND expression (&&)
    fn parse_logical_and(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_bitwise_or()?;
        
        // Parse && operators
        while self.current_token.token_type == TokenType::Lan {
            self.next_token()?;
            
            // Generate short-circuit evaluation
            let _jump_false_pos = self.emit(Opcode::BZ as i64);
            let jump_false_placeholder = self.emit(0);
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_bitwise_or()?;
            
            // Perform AND operation
            self.emit(Opcode::AND as i64);
            
            // Update jump position
            self.code[jump_false_placeholder] = self.code.len() as i64;
        }
        
        Ok(())
    }
    
    /// Parse bitwise OR expression (|)
    fn parse_bitwise_or(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_bitwise_xor()?;
        
        // Parse | operators
        while self.current_token.token_type == TokenType::Or {
            self.next_token()?;
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_bitwise_xor()?;
            
            // Perform bitwise OR
            self.emit(Opcode::OR as i64);
        }
        
        Ok(())
    }
    
    /// Parse bitwise XOR expression (^)
    fn parse_bitwise_xor(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_bitwise_and()?;
        
        // Parse ^ operators
        while self.current_token.token_type == TokenType::Xor {
            self.next_token()?;
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_bitwise_and()?;
            
            // Perform bitwise XOR
            self.emit(Opcode::XOR as i64);
        }
        
        Ok(())
    }
    
    /// Parse bitwise AND expression (&)
    fn parse_bitwise_and(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_equality()?;
        
        // Parse & operators
        while self.current_token.token_type == TokenType::And {
            self.next_token()?;
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_equality()?;
            
            // Perform bitwise AND
            self.emit(Opcode::AND as i64);
        }
        
        Ok(())
    }
    
    /// Parse equality expressions (==, !=)
    fn parse_equality(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_comparison()?;
        
        // Parse == and != operators
        while self.current_token.token_type == TokenType::Eq || 
              self.current_token.token_type == TokenType::Ne {
            
            let op = self.current_token.token_type;
            self.next_token()?;
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_comparison()?;
            
            // Perform comparison
            match op {
                TokenType::Eq => { self.emit(Opcode::EQ as i64); },
                TokenType::Ne => { self.emit(Opcode::NE as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Parse comparison expressions (<, >, <=, >=)
    fn parse_comparison(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_shift()?;
        
        // Parse comparison operators
        while self.current_token.token_type == TokenType::Lt || 
              self.current_token.token_type == TokenType::Gt ||
              self.current_token.token_type == TokenType::Le ||
              self.current_token.token_type == TokenType::Ge {
            
            let op = self.current_token.token_type;
            self.next_token()?;
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_shift()?;
            
            // Perform comparison
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
    
    /// Parse shift expressions (<<, >>)
    fn parse_shift(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_addition()?;
        
        // Parse << and >> operators
        while self.current_token.token_type == TokenType::Shl || 
              self.current_token.token_type == TokenType::Shr {
            
            let op = self.current_token.token_type;
            self.next_token()?;
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_addition()?;
            
            // Perform shift
            match op {
                TokenType::Shl => { self.emit(Opcode::SHL as i64); },
                TokenType::Shr => { self.emit(Opcode::SHR as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Parse addition and subtraction
    fn parse_addition(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_multiplication()?;
        
        // Parse + and - operators
        while self.current_token.token_type == TokenType::Add || 
              self.current_token.token_type == TokenType::Sub {
            
            let op = self.current_token.token_type;
            self.next_token()?;
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_multiplication()?;
            
            // Perform addition or subtraction
            match op {
                TokenType::Add => { self.emit(Opcode::ADD as i64); },
                TokenType::Sub => { self.emit(Opcode::SUB as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Parse multiplication, division, and modulo
    fn parse_multiplication(&mut self) -> Result<(), CompilerError> {
        // Parse left operand
        self.parse_unary()?;
        
        // Parse *, /, and % operators
        while self.current_token.token_type == TokenType::Mul || 
              self.current_token.token_type == TokenType::Div ||
              self.current_token.token_type == TokenType::Mod {
            
            let op = self.current_token.token_type;
            self.next_token()?;
            
            // Push current result
            self.emit(Opcode::PSH as i64);
            
            // Parse right operand
            self.parse_unary()?;
            
            // Perform operation
            match op {
                TokenType::Mul => { self.emit(Opcode::MUL as i64); },
                TokenType::Div => { self.emit(Opcode::DIV as i64); },
                TokenType::Mod => { self.emit(Opcode::MOD as i64); },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    /// Parse unary expressions
    fn parse_unary(&mut self) -> Result<(), CompilerError> {
        match self.current_token.token_type {
            TokenType::Add => {
                // Unary +
                self.next_token()?;
                self.parse_unary()?;
                // No operation needed, + is a no-op
            },
            TokenType::Sub => {
                // Unary -
                self.next_token()?;
                
                // Special case for numeric literals
                if self.current_token.token_type == TokenType::Num {
                    let value = self.current_token.value.unwrap();
                    self.next_token()?;
                    
                    self.emit(Opcode::IMM as i64);
                    self.emit(-value);
                } else {
                    // Load zero and subtract
                    self.emit(Opcode::IMM as i64);
                    self.emit(0);
                    
                    self.emit(Opcode::PSH as i64);
                    
                    self.parse_unary()?;
                    
                    self.emit(Opcode::SUB as i64);
                }
            },
            TokenType::Not => {
                // Logical NOT
                self.next_token()?;
                self.parse_unary()?;
                
                // Compare with zero
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                self.emit(0);
                self.emit(Opcode::EQ as i64);
            },
            TokenType::BitNot => {
                // Bitwise NOT
                self.next_token()?;
                self.parse_unary()?;
                
                // XOR with -1
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                self.emit(-1);
                self.emit(Opcode::XOR as i64);
            },
            TokenType::Mul => {
                // Pointer dereference
                self.next_token()?;
                self.parse_unary()?;
                
                // Load from address
                self.emit(Opcode::LI as i64);
            },
            TokenType::And => {
                // Address-of operator
                self.next_token()?;
                
                // Must be followed by an identifier
                if self.current_token.token_type != TokenType::Id {
                    return Err(CompilerError::ParserError(
                        format!("Expected identifier after &, got {:?}", self.current_token.token_type)
                    ));
                }
                
                // Get variable name
                let var_name = self.current_token.name.clone().unwrap();
                self.next_token()?;
                
                // Look up variable
                if let Some(symbol) = self.symbol_table.get(&var_name) {
                    match symbol.class {
                        TokenType::Glo => {
                            self.emit(Opcode::IMM as i64);
                            self.emit(symbol.value);
                        },
                        TokenType::Loc => {
                            self.emit(Opcode::LEA as i64);
                            self.emit(symbol.value);
                        },
                        _ => {
                            return Err(CompilerError::ParserError(
                                format!("Cannot take address of {}", var_name)
                            ));
                        }
                    }
                } else {
                    return Err(CompilerError::ParserError(
                        format!("Undefined variable: {}", var_name)
                    ));
                }
            },
            TokenType::Inc | TokenType::Dec => {
                // Pre-increment or pre-decrement
                let op = self.current_token.token_type;
                self.next_token()?;
                
                // Must be followed by an identifier
                if self.current_token.token_type != TokenType::Id {
                    return Err(CompilerError::ParserError(
                        format!("Expected identifier after {:?}, got {:?}", op, self.current_token.token_type)
                    ));
                }
                
                // Get variable name
                let var_name = self.current_token.name.clone().unwrap();
                self.next_token()?;
                
                // Look up variable
                if let Some(symbol) = self.symbol_table.get(&var_name) {
                    // Get variable address
                    match symbol.class {
                        TokenType::Glo => {
                            self.emit(Opcode::IMM as i64);
                            self.emit(symbol.value);
                        },
                        TokenType::Loc => {
                            self.emit(Opcode::LEA as i64);
                            self.emit(symbol.value);
                        },
                        _ => {
                            return Err(CompilerError::ParserError(
                                format!("Cannot modify {}", var_name)
                            ));
                        }
                    }
                    
                    // Duplicate address
                    self.emit(Opcode::PSH as i64);
                    
                    // Load current value
                    if symbol.typ == Type::CHAR {
                        self.emit(Opcode::LC as i64);
                    } else {
                        self.emit(Opcode::LI as i64);
                    }
                    
                    // Increment or decrement
                    self.emit(Opcode::PSH as i64);
                    self.emit(Opcode::IMM as i64);
                    self.emit(1);
                    
                    if op == TokenType::Inc {
                        self.emit(Opcode::ADD as i64);
                    } else {
                        self.emit(Opcode::SUB as i64);
                    }
                    
                    // Store back
                    if symbol.typ == Type::CHAR {
                        self.emit(Opcode::SC as i64);
                    } else {
                        self.emit(Opcode::SI as i64);
                    }
                } else {
                    return Err(CompilerError::ParserError(
                        format!("Undefined variable: {}", var_name)
                    ));
                }
            },
            _ => {
                // Not a unary operator, parse primary
                self.parse_primary()?;
            }
        }
        
        Ok(())
    }
    
    /// Parse primary expressions (literals, variables, function calls, etc.)
    fn parse_primary(&mut self) -> Result<(), CompilerError> {
        match self.current_token.token_type {
            TokenType::Num => {
                // Number literal
                let value = self.current_token.value.unwrap();
                self.next_token()?;
                
                self.emit(Opcode::IMM as i64);
                self.emit(value);
            },
            TokenType::Str => {
                // String literal
                let string_content = self.current_token.name.clone().unwrap();
                self.next_token()?;
                
                // Add string to data segment
                let string_addr = self.data_segment.len() as i64;
                
                // Copy string to data segment
                for &byte in string_content.as_bytes() {
                    self.data_segment.push(byte);
                }
                
                // Add null terminator
                self.data_segment.push(0);
                
                // Align to integer boundary
                while self.data_segment.len() % std::mem::size_of::<i64>() != 0 {
                    self.data_segment.push(0);
                }
                
                // Load string address
                self.emit(Opcode::IMM as i64);
                self.emit(string_addr);
            },
            TokenType::Id => {
                // Identifier (variable or function)
                let id_name = self.current_token.name.clone().unwrap();
                self.next_token()?;
                
                if self.current_token.token_type == TokenType::LParen {
                    // Function call
                    self.next_token()?;
                    
                    // Parse arguments
                    let mut arg_count = 0;
                    
                    if self.current_token.token_type != TokenType::RParen {
                        // Parse first argument
                        self.parse_expression()?;
                        self.emit(Opcode::PSH as i64);
                        arg_count += 1;
                        
                        // Parse additional arguments
                        while self.current_token.token_type == TokenType::Comma {
                            self.next_token()?;
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
                                self.emit(Opcode::JSR as i64);
                                self.emit(symbol.value);
                            },
                            _ => {
                                return Err(CompilerError::ParserError(
                                    format!("{} is not a function", id_name)
                                ));
                            }
                        }
                    } else {
                        return Err(CompilerError::ParserError(
                            format!("Undefined function: {}", id_name)
                        ));
                    }
                    
                    // Clean up arguments
                    if arg_count > 0 {
                        self.emit(Opcode::ADJ as i64);
                        self.emit(arg_count);
                    }
                } else {
                    // Variable access
                    if let Some(symbol) = self.symbol_table.get(&id_name) {
                        match symbol.class {
                            TokenType::Glo => {
                                self.emit(Opcode::IMM as i64);
                                self.emit(symbol.value);
                            },
                            TokenType::Loc => {
                                self.emit(Opcode::LEA as i64);
                                self.emit(symbol.value);
                            },
                            TokenType::Num => {
                                // Constant value (like enum)
                                self.emit(Opcode::IMM as i64);
                                self.emit(symbol.value);
                                return Ok(());
                            },
                            _ => {
                                return Err(CompilerError::ParserError(
                                    format!("Invalid variable: {}", id_name)
                                ));
                            }
                        }
                        
                        // Load value
                        if symbol.typ == Type::CHAR {
                            self.emit(Opcode::LC as i64);
                        } else {
                            self.emit(Opcode::LI as i64);
                        }
                    } else {
                        return Err(CompilerError::ParserError(
                            format!("Undefined variable: {}", id_name)
                        ));
                    }
                }
            },
            TokenType::LParen => {
                // Parenthesized expression
                self.next_token()?;
                self.parse_expression()?;
                self.match_token(TokenType::RParen)?;
            },
            TokenType::Sizeof => {
                // sizeof operator
                self.next_token()?;
                
                if self.current_token.token_type == TokenType::LParen {
                    self.next_token()?;
                    
                    // Parse type
                    let size_type = if self.current_token.token_type == TokenType::Int {
                        self.next_token()?;
                        Type::INT
                    } else if self.current_token.token_type == TokenType::Char {
                        self.next_token()?;
                        Type::CHAR
                    } else {
                        return Err(CompilerError::ParserError(
                            format!("Expected type in sizeof, got {:?}", self.current_token.token_type)
                        ));
                    };
                    
                    // Parse pointers
                    let mut size_type = size_type;
                    while self.current_token.token_type == TokenType::Mul {
                        size_type = size_type.to_ptr();
                        self.next_token()?;
                    }
                    
                    self.match_token(TokenType::RParen)?;
                    
                    // Generate code to load size
                    self.emit(Opcode::IMM as i64);
                    self.emit(size_type.size() as i64);
                } else {
                    return Err(CompilerError::ParserError(
                        format!("Expected ( after sizeof, got {:?}", self.current_token.token_type)
                    ));
                }
            },
            _ => {
                return Err(CompilerError::ParserError(
                    format!("Unexpected token in primary expression: {:?}", self.current_token.token_type)
                ));
            }
        }
        
        // Check for array access
        while self.current_token.token_type == TokenType::Brak {
            self.next_token()?;
            
            // Push array address
            self.emit(Opcode::PSH as i64);
            
            // Parse index expression
            self.parse_expression()?;
            
            self.match_token(TokenType::RBracket)?;
            
            // Calculate element address
            self.emit(Opcode::ADD as i64);
            
            // Load value from address
            self.emit(Opcode::LI as i64);
        }
        
        Ok(())
    }
}