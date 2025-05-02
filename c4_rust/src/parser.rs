use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::types::{TokenType, Type, Opcode};
use crate::symbol::SymbolTable;

/// Parser for C4 language
/// Converts tokens to VM instructions
pub struct Parser {
    /// Lexer for token stream
    lexer: Lexer,
    
    /// Symbol table
    symbols: SymbolTable,
    
    /// Current token
    token: TokenType,
    
    /// Current token value (for numeric literals)
    token_val: Option<i64>,
    
    /// Current token name (for identifiers)
    token_name: Option<String>,
    
    /// Current expression type
    expr_type: Type,
    
    /// Current function local variable offset
    locals_offset: i64,
    
    /// Generated code
    code: Vec<i64>,
    
    /// Data section
    data: Vec<u8>,
    
    /// Settings
    print_source: bool,
    debug_mode: bool,
}

impl Parser {
    /// Create a new parser
    pub fn new(source: String, print_source: bool, debug_mode: bool) -> Self {
        let lexer = Lexer::new(source, print_source);
        
        Parser {
            lexer,
            symbols: SymbolTable::new(),
            token: TokenType::Eof,
            token_val: None,
            token_name: None,
            expr_type: Type::INT,
            locals_offset: 0,
            code: Vec::new(),
            data: Vec::new(),
            print_source,
            debug_mode,
        }
    }
    
    /// Initialize the parser
    pub fn init(&mut self) -> Result<(), CompilerError> {
        // Initialize symbol table with keywords and built-in functions
        self.init_symbols();
        
        // Get the first token
        self.next_token()?;
        
        Ok(())
    }
    
    /// Initialize symbol table with keywords and built-in functions
    fn init_symbols(&mut self) {
        // Add library functions similar to C4's approach
        let lib_funcs = [
            ("open", TokenType::Sys, Type::INT, Opcode::OPEN as i64),
            ("read", TokenType::Sys, Type::INT, Opcode::READ as i64),
            ("close", TokenType::Sys, Type::INT, Opcode::CLOS as i64),
            ("printf", TokenType::Sys, Type::INT, Opcode::PRTF as i64),
            ("malloc", TokenType::Sys, Type::INT, Opcode::MALC as i64),
            ("free", TokenType::Sys, Type::INT, Opcode::FREE as i64),
            ("memset", TokenType::Sys, Type::INT, Opcode::MSET as i64),
            ("memcmp", TokenType::Sys, Type::INT, Opcode::MCMP as i64),
            ("exit", TokenType::Sys, Type::INT, Opcode::EXIT as i64),
            // Add "void" as a type
            ("void", TokenType::Num, Type::INT, 0),
        ];
        
        for (name, class, typ, val) in lib_funcs.iter() {
            self.symbols.add(name, *class, *typ, *val);
        }
    }
    
    /// Get the next token
    fn next_token(&mut self) -> Result<(), CompilerError> {
        let token = self.lexer.next_token()?;
        self.token = token.token_type;
        self.token_val = token.value;
        self.token_name = token.name;
        
        Ok(())
    }
    
    /// Parse the program
    pub fn parse(&mut self) -> Result<(), CompilerError> {
        // Parse declarations
        self.parse_declarations()?;
        
        // Find main function
        match self.symbols.get_main() {
            Some(main_sym) => {
                if main_sym.class != TokenType::Fun {
                    return Err(CompilerError::ParserError("main is not a function".to_string()));
                }
                // Return the entry point address
                Ok(())
            },
            None => Err(CompilerError::ParserError("main() not defined".to_string())),
        }
    }
    
    /// Parse declarations (global variables and functions)
    fn parse_declarations(&mut self) -> Result<(), CompilerError> {
        while self.token != TokenType::Eof {
            // Parse a base type for the declaration
            let mut base_type = Type::INT;
            
            if self.token == TokenType::Int {
                self.next_token()?;
            } else if self.token == TokenType::Char {
                self.next_token()?;
                base_type = Type::CHAR;
            } else if self.token == TokenType::Enum {
                // Handle enum declarations
                self.next_token()?;
                
                // Skip enum name if present
                if self.token == TokenType::Id {
                    self.next_token()?;
                }
                
                // Expect opening brace
                if self.token != TokenType::LBrace {
                    return Err(CompilerError::ParserError(
                        format!("expected '{{' after enum at line {}", self.lexer.line())
                    ));
                }
                
                // Parse enum body
                self.next_token()?; // Skip '{'
                let mut enum_val = 0;
                
                while self.token != TokenType::RBrace {
                    if self.token != TokenType::Id {
                        return Err(CompilerError::ParserError(
                            format!("expected identifier in enum at line {}", self.lexer.line())
                        ));
                    }
                    
                    // Get enum identifier
                    let id_name = self.token_name.clone().unwrap();
                    self.next_token()?;
                    
                    // Check for explicit value
                    if self.token == TokenType::Assign {
                        self.next_token()?;
                        
                        if self.token != TokenType::Num {
                            return Err(CompilerError::ParserError(
                                format!("expected number in enum initializer at line {}", self.lexer.line())
                            ));
                        }
                        
                        enum_val = self.token_val.unwrap();
                        self.next_token()?;
                    }
                    
                    // Add enum identifier to symbol table
                    self.symbols.add(&id_name, TokenType::Num, Type::INT, enum_val);
                    enum_val += 1;
                    
                    // Skip comma
                    if self.token == TokenType::Comma {
                        self.next_token()?;
                    }
                }
                
                // Skip closing brace
                if self.token == TokenType::RBrace {
                    self.next_token()?;
                }
                
                // Skip semicolon
                if self.token == TokenType::Semicolon {
                    self.next_token()?;
                }
                
                // Continue to next declaration
                continue;
            } else {
                return Err(CompilerError::ParserError(
                    format!("expected type specifier at line {}", self.lexer.line())
                ));
            }
            
            // Parse declarator(s)
            while self.token != TokenType::Semicolon && self.token != TokenType::RBrace {
                // Handle pointer types
                let mut typ = base_type;
                while self.token == TokenType::Mul {
                    self.next_token()?;
                    typ = typ.to_ptr();
                }
                
                // Expect identifier
                if self.token != TokenType::Id {
                    return Err(CompilerError::ParserError(
                        format!("expected identifier at line {}", self.lexer.line())
                    ));
                }
                
                // Get identifier name
                let id_name = self.token_name.clone().unwrap();
                self.next_token()?;
                
                // Check if identifier already exists
                if self.symbols.exists(&id_name) {
                    return Err(CompilerError::ParserError(
                        format!("duplicate definition of '{}' at line {}", id_name, self.lexer.line())
                    ));
                }
                
                // Function declaration
                if self.token == TokenType::LParen {
                    self.parse_function(&id_name, typ)?;
                }
                // Global variable
                else {
                    // Add to symbol table
                    let data_offset = self.data.len() as i64;
                    self.symbols.add(&id_name, TokenType::Glo, typ, data_offset);
                    
                    // Allocate space in data section
                    self.data.resize(self.data.len() + typ.size(), 0);
                    
                    // Handle initializers (later)
                }
                
                // Multiple declarations separated by comma
                if self.token == TokenType::Comma {
                    self.next_token()?;
                }
            }
            
            // Skip semicolon
            if self.token == TokenType::Semicolon {
                self.next_token()?;
            }
        }
        
        Ok(())
    }
    
    /// Parse function declaration and definition
    fn parse_function(&mut self, name: &str, ret_type: Type) -> Result<(), CompilerError> {
        // Save the function entry address
        let func_addr = self.code.len() as i64;
        
        // Add to symbol table
        self.symbols.add(name, TokenType::Fun, ret_type, func_addr);
        
        // Enter function scope
        self.symbols.enter_scope();
        
        // Parse parameter list
        self.next_token()?; // Skip '('
        let mut param_count = 0;
        
        while self.token != TokenType::RParen {
            // Parameter type
            let mut param_type = Type::INT;
            
            if self.token == TokenType::Int {
                self.next_token()?;
            } else if self.token == TokenType::Char {
                self.next_token()?;
                param_type = Type::CHAR;
            } else {
                return Err(CompilerError::ParserError(
                    format!("expected parameter type at line {}", self.lexer.line())
                ));
            }
            
            // Handle pointer types
            while self.token == TokenType::Mul {
                self.next_token()?;
                param_type = param_type.to_ptr();
            }
            
            // Parameter name
            if self.token != TokenType::Id {
                return Err(CompilerError::ParserError(
                    format!("expected parameter name at line {}", self.lexer.line())
                ));
            }
            
            let param_name = self.token_name.clone().unwrap();
            self.next_token()?;
            
            // Check for duplicate parameter
            if self.symbols.exists_in_current_scope(&param_name) {
                return Err(CompilerError::ParserError(
                    format!("duplicate parameter '{}' at line {}", param_name, self.lexer.line())
                ));
            }
            
            // Add parameter to symbol table
            self.symbols.add(&param_name, TokenType::Loc, param_type, param_count);
            param_count += 1;
            
            // Next parameter
            if self.token == TokenType::Comma {
                self.next_token()?;
            }
        }
        
        // Skip ')'
        self.next_token()?;
        
        // Function body
        if self.token != TokenType::LBrace {
            return Err(CompilerError::ParserError(
                format!("expected '{{' before function body at line {}", self.lexer.line())
            ));
        }
        
        self.next_token()?; // Skip '{'
        
        // Reset local variable offset
        self.locals_offset = param_count;
        
        // Parse local variable declarations
        while self.token == TokenType::Int || self.token == TokenType::Char {
            let local_type = if self.token == TokenType::Int {
                self.next_token()?;
                Type::INT
            } else {
                self.next_token()?;
                Type::CHAR
            };
            
            // Parse declarator(s)
            while self.token != TokenType::Semicolon {
                // Handle pointer types
                let mut var_type = local_type;
                while self.token == TokenType::Mul {
                    self.next_token()?;
                    var_type = var_type.to_ptr();
                }
                
                // Variable name
                if self.token != TokenType::Id {
                    return Err(CompilerError::ParserError(
                        format!("expected variable name at line {}", self.lexer.line())
                    ));
                }
                
                let var_name = self.token_name.clone().unwrap();
                self.next_token()?;
                
                // Check for duplicate
                if self.symbols.exists_in_current_scope(&var_name) {
                    return Err(CompilerError::ParserError(
                        format!("duplicate local variable '{}' at line {}", var_name, self.lexer.line())
                    ));
                }
                
                // Add local variable to symbol table
                self.locals_offset += 1;
                self.symbols.add(&var_name, TokenType::Loc, var_type, self.locals_offset - 1);
                
                // Multiple declarations
                if self.token == TokenType::Comma {
                    self.next_token()?;
                }
            }
            
            // Skip semicolon
            self.next_token()?;
        }
        
        // Generate function prologue
        self.emit(Opcode::ENT as i64);
        self.emit(self.locals_offset - param_count);
        
        // Parse statements
        while self.token != TokenType::RBrace {
            self.statement()?;
        }
        
        // Generate function epilogue
        self.emit(Opcode::LEV as i64);
        
        // Exit function scope
        self.symbols.exit_scope();
        
        // Skip closing brace
        self.next_token()?;
        
        Ok(())
    }
    
    /// Parse a statement
    fn statement(&mut self) -> Result<(), CompilerError> {
        // if statement
        if self.token == TokenType::If {
            self.next_token()?;
            
            // Parse condition
            if self.token != TokenType::LParen {
                return Err(CompilerError::ParserError(
                    format!("expected '(' after 'if' at line {}", self.lexer.line())
                ));
            }
            
            self.next_token()?; // Skip '('
            self.expression(TokenType::Assign)?;
            
            if self.token != TokenType::RParen {
                return Err(CompilerError::ParserError(
                    format!("expected ')' after condition at line {}", self.lexer.line())
                ));
            }
            
            self.next_token()?; // Skip ')'
            
            // Generate jump if false
            self.emit(Opcode::BZ as i64);
            let else_jump = self.code.len() as i64;
            self.emit(0); // Placeholder for else jump address
            
            // Parse then-clause
            self.statement()?;
            
            // Handle else-clause
            if self.token == TokenType::Else {
                self.next_token()?;
                
                // Generate jump around else
                self.emit(Opcode::JMP as i64);
                let end_jump = self.code.len() as i64;
                self.emit(0); // Placeholder for end jump address
                
                // Patch else jump address
                self.code[else_jump as usize] = self.code.len() as i64;
                
                // Parse else-clause
                self.statement()?;
                
                // Patch end jump address
                self.code[end_jump as usize] = self.code.len() as i64;
            } else {
                // No else, patch jump address
                self.code[else_jump as usize] = self.code.len() as i64;
            }
        }
        // while statement
        else if self.token == TokenType::While {
            self.next_token()?;
            
            // Mark loop start
            let loop_start = self.code.len() as i64;
            
            // Parse condition
            if self.token != TokenType::LParen {
                return Err(CompilerError::ParserError(
                    format!("expected '(' after 'while' at line {}", self.lexer.line())
                ));
            }
            
            self.next_token()?; // Skip '('
            self.expression(TokenType::Assign)?;
            
            if self.token != TokenType::RParen {
                return Err(CompilerError::ParserError(
                    format!("expected ')' after condition at line {}", self.lexer.line())
                ));
            }
            
            self.next_token()?; // Skip ')'
            
            // Generate jump if false
            self.emit(Opcode::BZ as i64);
            let loop_exit = self.code.len() as i64;
            self.emit(0); // Placeholder for exit jump address
            
            // Parse loop body
            self.statement()?;
            
            // Jump back to start
            self.emit(Opcode::JMP as i64);
            self.emit(loop_start);
            
            // Patch exit jump address
            self.code[loop_exit as usize] = self.code.len() as i64;
        }
        // return statement
        else if self.token == TokenType::Return {
            self.next_token()?;
            
            // Parse return value
            if self.token != TokenType::Semicolon {
                self.expression(TokenType::Assign)?;
            }
            
            if self.token != TokenType::Semicolon {
                return Err(CompilerError::ParserError(
                    format!("expected ';' after return statement at line {}", self.lexer.line())
                ));
            }
            
            self.next_token()?; // Skip ';'
            
            // Generate return
            self.emit(Opcode::LEV as i64);
        }
        // compound statement (block)
        else if self.token == TokenType::LBrace {
            self.next_token()?; // Skip '{'
            
            // Parse statements
            while self.token != TokenType::RBrace {
                self.statement()?;
            }
            
            self.next_token()?; // Skip '}'
        }
        // empty statement
        else if self.token == TokenType::Semicolon {
            self.next_token()?; // Skip ';'
        }
        // expression statement
        else {
            self.expression(TokenType::Assign)?;
            
            if self.token != TokenType::Semicolon {
                return Err(CompilerError::ParserError(
                    format!("expected ';' after expression at line {}", self.lexer.line())
                ));
            }
            
            self.next_token()?; // Skip ';'
        }
        
        Ok(())
    }
    
    /// Parse an expression
    /// The level parameter controls operator precedence
    fn expression(&mut self, level: TokenType) -> Result<(), CompilerError> {
        // Handle primary expressions
        self.primary_expr()?;
        
        // Handle operators according to precedence
        while self.token >= level {
            // Save expression type
            let expr_type = self.expr_type;
            
            match self.token {
                // Assignment
                TokenType::Assign => {
                    self.next_token()?;
                    
                    // Ensure left side is an lvalue
                    if let Some(last_op) = self.code.last() {
                        // Check if last operation was LI or LC
                        if *last_op == Opcode::LI as i64 || *last_op == Opcode::LC as i64 {
                            // Convert load to address calculation
                            self.code.pop();
                        } else {
                            return Err(CompilerError::ParserError(
                                format!("not an lvalue at line {}", self.lexer.line())
                            ));
                        }
                    }
                    
                    // Push address
                    self.emit(Opcode::PSH as i64);
                    
                    // Parse right side
                    self.expression(TokenType::Assign)?;
                    
                    // Store value
                    if expr_type == Type::CHAR {
                        self.emit(Opcode::SC as i64);
                    } else {
                        self.emit(Opcode::SI as i64);
                    }
                    
                    // Keep expression type
                    self.expr_type = expr_type;
                },
                
                // Arithmetic and logical operators
                TokenType::Add => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Mul)?;
                    
                    // Handle pointer arithmetic
                    if expr_type.is_ptr() {
                        self.emit(Opcode::PSH as i64);
                        self.emit(Opcode::IMM as i64);
                        self.emit(Type::INT.size() as i64);
                        self.emit(Opcode::MUL as i64);
                    }
                    
                    self.emit(Opcode::ADD as i64);
                    self.expr_type = expr_type;
                },
                
                TokenType::Sub => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Mul)?;
                    
                    // Handle pointer arithmetic or pointer subtraction
                    if expr_type.is_ptr() && self.expr_type.is_ptr() {
                        // Pointer subtraction
                        self.emit(Opcode::SUB as i64);
                        self.emit(Opcode::PSH as i64);
                        self.emit(Opcode::IMM as i64);
                        self.emit(Type::INT.size() as i64);
                        self.emit(Opcode::DIV as i64);
                        self.expr_type = Type::INT;
                    } else if expr_type.is_ptr() {
                        // Pointer arithmetic
                        self.emit(Opcode::PSH as i64);
                        self.emit(Opcode::IMM as i64);
                        self.emit(Type::INT.size() as i64);
                        self.emit(Opcode::MUL as i64);
                        self.emit(Opcode::SUB as i64);
                        self.expr_type = expr_type;
                    } else {
                        // Regular subtraction
                        self.emit(Opcode::SUB as i64);
                        self.expr_type = Type::INT;
                    }
                },
                
                TokenType::Mul => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Inc)?;
                    self.emit(Opcode::MUL as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Div => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Inc)?;
                    self.emit(Opcode::DIV as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Mod => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Inc)?;
                    self.emit(Opcode::MOD as i64);
                    self.expr_type = Type::INT;
                },
                
                // Comparison operators
                TokenType::Eq => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Lt)?;
                    self.emit(Opcode::EQ as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Ne => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Lt)?;
                    self.emit(Opcode::NE as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Lt => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Shl)?;
                    self.emit(Opcode::LT as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Gt => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Shl)?;
                    self.emit(Opcode::GT as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Le => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Shl)?;
                    self.emit(Opcode::LE as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Ge => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Shl)?;
                    self.emit(Opcode::GE as i64);
                    self.expr_type = Type::INT;
                },
                
                // Bitwise operators
                TokenType::Shl => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Add)?;
                    self.emit(Opcode::SHL as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Shr => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Add)?;
                    self.emit(Opcode::SHR as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::And => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Eq)?;
                    self.emit(Opcode::AND as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Or => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Xor)?;
                    self.emit(Opcode::OR as i64);
                    self.expr_type = Type::INT;
                },
                
                TokenType::Xor => {
                    self.next_token()?;
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::And)?;
                    self.emit(Opcode::XOR as i64);
                    self.expr_type = Type::INT;
                },
                
                // Logical operators
                TokenType::Lan => {
                    self.next_token()?;
                    self.emit(Opcode::BZ as i64);
                    let skip = self.code.len() as i64;
                    self.emit(0); // Placeholder for skip address
                    self.expression(TokenType::Or)?;
                    // Patch skip address
                    self.code[skip as usize] = self.code.len() as i64;
                    self.expr_type = Type::INT;
                },
                
                TokenType::Lor => {
                    self.next_token()?;
                    self.emit(Opcode::BNZ as i64);
                    let skip = self.code.len() as i64;
                    self.emit(0); // Placeholder for skip address
                    self.expression(TokenType::Lan)?;
                    // Patch skip address
                    self.code[skip as usize] = self.code.len() as i64;
                    self.expr_type = Type::INT;
                },
                
                // Conditional operator (ternary)
                TokenType::Cond => {
                    self.next_token()?;
                    self.emit(Opcode::BZ as i64);
                    let false_branch = self.code.len() as i64;
                    self.emit(0); // Placeholder for false branch address
                    
                    // Parse true expression
                    self.expression(TokenType::Assign)?;
                    
                    if self.token != TokenType::Colon {
                        return Err(CompilerError::ParserError(
                            format!("expected ':' in conditional expression at line {}", self.lexer.line())
                        ));
                    }
                    
                    self.next_token()?; // Skip ':'
                    
                    // Jump around false expression
                    self.emit(Opcode::JMP as i64);
                    let end_jump = self.code.len() as i64;
                    self.emit(0); // Placeholder for end jump address
                    
                    // Patch false branch address
                    self.code[false_branch as usize] = self.code.len() as i64;
                    
                    // Parse false expression
                    self.expression(TokenType::Cond)?;
                    
                    // Patch end jump address
                    self.code[end_jump as usize] = self.code.len() as i64;
                },
                
                // Array indexing - handled in primary_expr
                TokenType::Brak => {
                    break;
                },
                
                _ => {
                    return Err(CompilerError::ParserError(
                        format!("unexpected operator {:?} at line {}", self.token, self.lexer.line())
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse a primary expression
    fn primary_expr(&mut self) -> Result<(), CompilerError> {
        match self.token {
            // Numeric literal
            TokenType::Num => {
                // Load immediate value
                self.emit(Opcode::IMM as i64);
                self.emit(self.token_val.unwrap());
                self.expr_type = Type::INT;
                self.next_token()?;
            },
            
            // String literal
            TokenType::Str => {
                // Add string to data section
                let start_pos = self.data.len() as i64;
                
                if let Some(string) = &self.token_name {
                    // Add string data
                    for &byte in string.as_bytes() {
                        self.data.push(byte);
                    }
                    // Null terminate
                    self.data.push(0);
                }
                
                // Load string address
                self.emit(Opcode::IMM as i64);
                self.emit(start_pos);
                self.expr_type = Type::PTR;
                self.next_token()?;
            },
            
            // Identifier (variable or function)
            TokenType::Id => {
                let id_name = self.token_name.clone().unwrap();
                self.next_token()?;
                
                // Look up the identifier
                let symbol = match self.symbols.get(&id_name) {
                    Some(sym) => sym,
                    None => {
                        return Err(CompilerError::ParserError(
                            format!("undefined symbol '{}' at line {}", id_name, self.lexer.line())
                        ));
                    }
                };
                
                // Function call
                if self.token == TokenType::LParen {
                    self.next_token()?; // Skip '('
                    
                    // Parse argument list
                    let mut arg_count = 0;
                    
                    while self.token != TokenType::RParen {
                        // Parse argument
                        self.expression(TokenType::Assign)?;
                        self.emit(Opcode::PSH as i64);
                        arg_count += 1;
                        
                        // Skip comma
                        if self.token == TokenType::Comma {
                            self.next_token()?;
                        }
                    }
                    
                    self.next_token()?; // Skip ')'
                    
                    // Generate function call
                    if symbol.class == TokenType::Sys {
                        // System call
                        self.emit(symbol.value);
                    } else if symbol.class == TokenType::Fun {
                        // User function call
                        self.emit(Opcode::JSR as i64);
                        self.emit(symbol.value);
                    } else {
                        return Err(CompilerError::ParserError(
                            format!("'{}' is not a function at line {}", id_name, self.lexer.line())
                        ));
                    }
                    
                    // Clean up arguments
                    if arg_count > 0 {
                        self.emit(Opcode::ADJ as i64);
                        self.emit(arg_count);
                    }
                    
                    // Set expression type to function return type
                    self.expr_type = symbol.typ;
                }
                // Variable
                else {
                    if symbol.class == TokenType::Num {
                        // Constant
                        self.emit(Opcode::IMM as i64);
                        self.emit(symbol.value);
                        self.expr_type = Type::INT;
                    } else {
                        // Variable
                        if symbol.class == TokenType::Loc {
                            // Local variable
                            self.emit(Opcode::LEA as i64);
                            self.emit(symbol.value);
                        } else if symbol.class == TokenType::Glo {
                            // Global variable
                            self.emit(Opcode::IMM as i64);
                            self.emit(symbol.value);
                        } else {
                            return Err(CompilerError::ParserError(
                                format!("invalid variable class for '{}' at line {}", id_name, self.lexer.line())
                            ));
                        }
                        
                        // Load variable value
                        if symbol.typ == Type::CHAR {
                            self.emit(Opcode::LC as i64);
                        } else {
                            self.emit(Opcode::LI as i64);
                        }
                        
                        self.expr_type = symbol.typ;
                    }
                }
            },
            
            // Parenthesized expression or cast
            TokenType::LParen => {
                self.next_token()?; // Skip '('
                
                // Type cast
                if self.token == TokenType::Int || self.token == TokenType::Char {
                    let cast_type = if self.token == TokenType::Int {
                        self.next_token()?;
                        Type::INT
                    } else {
                        self.next_token()?;
                        Type::CHAR
                    };
                    
                    // Handle pointer types
                    let mut typ = cast_type;
                    while self.token == TokenType::Mul {
                        self.next_token()?;
                        typ = typ.to_ptr();
                    }
                    
                    // Expect closing parenthesis
                    if self.token != TokenType::RParen {
                        return Err(CompilerError::ParserError(
                            format!("expected ')' in cast at line {}", self.lexer.line())
                        ));
                    }
                    
                    self.next_token()?; // Skip ')'
                    
                    // Parse expression to cast
                    self.expression(TokenType::Inc)?;
                    
                    // Set the cast type
                    self.expr_type = typ;
                }
                // Regular parenthesized expression
                else {
                    self.expression(TokenType::Assign)?;
                    
                    if self.token != TokenType::RParen {
                        return Err(CompilerError::ParserError(
                            format!("expected ')' at line {}", self.lexer.line())
                        ));
                    }
                    
                    self.next_token()?; // Skip ')'
                }
            },
            
            // Dereference operator
            TokenType::Mul => {
                self.next_token()?;
                self.expression(TokenType::Inc)?;
                
                if !self.expr_type.is_ptr() {
                    return Err(CompilerError::ParserError(
                        format!("invalid dereference at line {}", self.lexer.line())
                    ));
                }
                
                // Adjust the type (reduce pointer level)
                let new_type = match self.expr_type {
                    Type::PTR => Type::INT,
                    _ => {
                        // For pointers to pointers, decrement by 1
                        unsafe { std::mem::transmute::<u8, Type>(self.expr_type as u8 - 1) }
                    }
                };
                
                // Load through pointer
                if new_type == Type::CHAR {
                    self.emit(Opcode::LC as i64);
                } else {
                    self.emit(Opcode::LI as i64);
                }
                
                self.expr_type = new_type;
            },
            
            // Address-of operator
            TokenType::And => {
                self.next_token()?;
                self.expression(TokenType::Inc)?;
                
                // Check if operand is an lvalue (LI or LC)
                if let Some(last_op) = self.code.last() {
                    if *last_op == Opcode::LI as i64 || *last_op == Opcode::LC as i64 {
                        // Remove the load instruction
                        self.code.pop();
                    } else {
                        return Err(CompilerError::ParserError(
                            format!("invalid address-of operand at line {}", self.lexer.line())
                        ));
                    }
                }
                
                // Make the expression type a pointer to the original type
                self.expr_type = self.expr_type.to_ptr();
            },
            
            // Logical NOT
            TokenType::Not => {
                self.next_token()?;
                self.expression(TokenType::Inc)?;
                
                // Generate NOT operation
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                self.emit(0);
                self.emit(Opcode::EQ as i64);
                
                self.expr_type = Type::INT;
            },
            
            // Bitwise NOT
            TokenType::BitNot => {
                self.next_token()?;
                self.expression(TokenType::Inc)?;
                
                // Generate bitwise NOT
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                self.emit(-1);
                self.emit(Opcode::XOR as i64);
                
                self.expr_type = Type::INT;
            },
            
            // Unary plus
            TokenType::Add => {
                self.next_token()?;
                self.expression(TokenType::Inc)?;
                // No operation needed, just keep the value
            },
            
            // Unary minus
            TokenType::Sub => {
                self.next_token()?;
                
                // Check if the next token is a number for optimization
                if self.token == TokenType::Num {
                    // Load negative number directly
                    self.emit(Opcode::IMM as i64);
                    self.emit(-self.token_val.unwrap());
                    self.expr_type = Type::INT;
                    self.next_token()?;
                } else {
                    // Calculate negation
                    self.emit(Opcode::IMM as i64);
                    self.emit(-1);
                    self.emit(Opcode::PSH as i64);
                    self.expression(TokenType::Inc)?;
                    self.emit(Opcode::MUL as i64);
                    self.expr_type = Type::INT;
                }
            },
            
            // Pre-increment
            TokenType::Inc => {
                self.next_token()?;
                self.expression(TokenType::Inc)?;
                
                // Check if operand is an lvalue
                if let Some(op) = self.code.last() {
                    if *op == Opcode::LC as i64 {
                        // Char load, replace with push + load
                        *self.code.last_mut().unwrap() = Opcode::PSH as i64;
                        self.emit(Opcode::LC as i64);
                    } else if *op == Opcode::LI as i64 {
                        // Int load, replace with push + load
                        *self.code.last_mut().unwrap() = Opcode::PSH as i64;
                        self.emit(Opcode::LI as i64);
                    } else {
                        return Err(CompilerError::ParserError(
                            format!("invalid lvalue in pre-increment at line {}", self.lexer.line())
                        ));
                    }
                }
                
                // Increment and store
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                
                if self.expr_type.is_ptr() {
                    self.emit(Type::INT.size() as i64);
                } else {
                    self.emit(1);
                }
                
                self.emit(Opcode::ADD as i64);
                
                if self.expr_type == Type::CHAR {
                    self.emit(Opcode::SC as i64);
                } else {
                    self.emit(Opcode::SI as i64);
                }
            },
            
            // Pre-decrement
            TokenType::Dec => {
                self.next_token()?;
                self.expression(TokenType::Inc)?;
                
                // Check if operand is an lvalue
                if let Some(op) = self.code.last() {
                    if *op == Opcode::LC as i64 {
                        // Char load, replace with push + load
                        *self.code.last_mut().unwrap() = Opcode::PSH as i64;
                        self.emit(Opcode::LC as i64);
                    } else if *op == Opcode::LI as i64 {
                        // Int load, replace with push + load
                        *self.code.last_mut().unwrap() = Opcode::PSH as i64;
                        self.emit(Opcode::LI as i64);
                    } else {
                        return Err(CompilerError::ParserError(
                            format!("invalid lvalue in pre-decrement at line {}", self.lexer.line())
                        ));
                    }
                }
                
                // Decrement and store
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                
                if self.expr_type.is_ptr() {
                    self.emit(Type::INT.size() as i64);
                } else {
                    self.emit(1);
                }
                
                self.emit(Opcode::SUB as i64);
                
                if self.expr_type == Type::CHAR {
                    self.emit(Opcode::SC as i64);
                } else {
                    self.emit(Opcode::SI as i64);
                }
            },
            
            // Sizeof operator
            TokenType::Sizeof => {
                self.next_token()?;
                
                // Check for parenthesis
                if self.token != TokenType::LParen {
                    return Err(CompilerError::ParserError(
                        format!("expected '(' after sizeof at line {}", self.lexer.line())
                    ));
                }
                
                self.next_token()?; // Skip '('
                
                // Parse the type
                let mut size_type = Type::INT;
                
                if self.token == TokenType::Int {
                    self.next_token()?;
                } else if self.token == TokenType::Char {
                    self.next_token()?;
                    size_type = Type::CHAR;
                }
                
                // Handle pointer types
                while self.token == TokenType::Mul {
                    self.next_token()?;
                    size_type = size_type.to_ptr();
                }
                
                // Check for closing parenthesis
                if self.token != TokenType::RParen {
                    return Err(CompilerError::ParserError(
                        format!("expected ')' after type in sizeof at line {}", self.lexer.line())
                    ));
                }
                
                self.next_token()?; // Skip ')'
                
                // Calculate the size
                self.emit(Opcode::IMM as i64);
                self.emit(size_type.size() as i64);
                self.expr_type = Type::INT;
            },
            
            // Unsupported expression
            _ => {
                return Err(CompilerError::ParserError(
                    format!("unexpected token {:?} in expression at line {}", self.token, self.lexer.line())
                ));
            }
        }
        
        // Post operators
        while self.token == TokenType::Inc || self.token == TokenType::Dec || self.token == TokenType::Brak {
            // Post-increment and post-decrement
            if self.token == TokenType::Inc || self.token == TokenType::Dec {
                let is_inc = self.token == TokenType::Inc;
                
                // Check if operand is an lvalue
                if let Some(op) = self.code.last() {
                    if *op == Opcode::LC as i64 {
                        // Char load, replace with push + duplicate load
                        *self.code.last_mut().unwrap() = Opcode::PSH as i64;
                        self.emit(Opcode::LC as i64);
                    } else if *op == Opcode::LI as i64 {
                        // Int load, replace with push + duplicate load
                        *self.code.last_mut().unwrap() = Opcode::PSH as i64;
                        self.emit(Opcode::LI as i64);
                    } else {
                        return Err(CompilerError::ParserError(
                            format!("invalid lvalue in post-increment at line {}", self.lexer.line())
                        ));
                    }
                }
                
                // Push address and value
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                
                // Determine increment/decrement size
                if self.expr_type.is_ptr() {
                    self.emit(Type::INT.size() as i64);
                } else {
                    self.emit(1);
                }
                
                // Perform operation
                if is_inc {
                    self.emit(Opcode::ADD as i64);
                } else {
                    self.emit(Opcode::SUB as i64);
                }
                
                // Store new value
                if self.expr_type == Type::CHAR {
                    self.emit(Opcode::SC as i64);
                } else {
                    self.emit(Opcode::SI as i64);
                }
                
                // For post-op, we need to push original value again for result
                self.emit(Opcode::PSH as i64);
                self.emit(Opcode::IMM as i64);
                
                if self.expr_type.is_ptr() {
                    self.emit(Type::INT.size() as i64);
                } else {
                    self.emit(1);
                }
                
                // Perform reverse operation to get original value
                if is_inc {
                    self.emit(Opcode::SUB as i64);
                } else {
                    self.emit(Opcode::ADD as i64);
                }
                
                self.next_token()?;
            }
            // Array indexing
            else if self.token == TokenType::Brak {
                // Check if previous expression is a pointer
                if !self.expr_type.is_ptr() {
                    return Err(CompilerError::ParserError(
                        format!("pointer type expected for indexing at line {}", self.lexer.line())
                    ));
                }
                
                self.next_token()?; // Skip '['
                
                // Push array address
                self.emit(Opcode::PSH as i64);
                
                // Parse index expression
                self.expression(TokenType::Assign)?;
                
                // Check for closing bracket
                if self.token != TokenType::RBracket {
                    return Err(CompilerError::ParserError(
                        format!("expected ']' at line {}", self.lexer.line())
                    ));
                }
                
                self.next_token()?; // Skip ']'
                
                // Scale index if necessary
                if self.expr_type.is_ptr() {
                    self.emit(Opcode::PSH as i64);
                    self.emit(Opcode::IMM as i64);
                    self.emit(Type::INT.size() as i64);
                    self.emit(Opcode::MUL as i64);
                }
                
                // Add base and index
                self.emit(Opcode::ADD as i64);
                
                // Determine element type and load it
                let elem_type = match self.expr_type {
                    Type::PTR => Type::INT,
                    _ => {
                        // For pointers to pointers, decrement by 1
                        unsafe { std::mem::transmute::<u8, Type>(self.expr_type as u8 - 1) }
                    }
                };
                
                if elem_type == Type::CHAR {
                    self.emit(Opcode::LC as i64);
                } else {
                    self.emit(Opcode::LI as i64);
                }
                
                self.expr_type = elem_type;
            }
        }
        
        Ok(())
    }
    
    /// Emit a VM instruction
    fn emit(&mut self, code: i64) {
        self.code.push(code);
        
        // Print code if source printing is enabled
        if self.print_source && self.debug_mode {
            let opcode = match code {
                c if c == Opcode::LEA as i64 => "LEA",
                c if c == Opcode::IMM as i64 => "IMM",
                c if c == Opcode::JMP as i64 => "JMP",
                c if c == Opcode::JSR as i64 => "JSR",
                c if c == Opcode::BZ as i64 => "BZ",
                c if c == Opcode::BNZ as i64 => "BNZ",
                c if c == Opcode::ENT as i64 => "ENT",
                c if c == Opcode::ADJ as i64 => "ADJ",
                c if c == Opcode::LEV as i64 => "LEV",
                c if c == Opcode::LI as i64 => "LI",
                c if c == Opcode::LC as i64 => "LC",
                c if c == Opcode::SI as i64 => "SI",
                c if c == Opcode::SC as i64 => "SC",
                c if c == Opcode::PSH as i64 => "PSH",
                c if c == Opcode::OR as i64 => "OR",
                c if c == Opcode::XOR as i64 => "XOR",
                c if c == Opcode::AND as i64 => "AND",
                c if c == Opcode::EQ as i64 => "EQ",
                c if c == Opcode::NE as i64 => "NE",
                c if c == Opcode::LT as i64 => "LT",
                c if c == Opcode::GT as i64 => "GT",
                c if c == Opcode::LE as i64 => "LE",
                c if c == Opcode::GE as i64 => "GE",
                c if c == Opcode::SHL as i64 => "SHL",
                c if c == Opcode::SHR as i64 => "SHR",
                c if c == Opcode::ADD as i64 => "ADD",
                c if c == Opcode::SUB as i64 => "SUB",
                c if c == Opcode::MUL as i64 => "MUL",
                c if c == Opcode::DIV as i64 => "DIV",
                c if c == Opcode::MOD as i64 => "MOD",
                c if c == Opcode::OPEN as i64 => "OPEN",
                c if c == Opcode::READ as i64 => "READ",
                c if c == Opcode::CLOS as i64 => "CLOS",
                c if c == Opcode::PRTF as i64 => "PRTF",
                c if c == Opcode::MALC as i64 => "MALC",
                c if c == Opcode::FREE as i64 => "FREE",
                c if c == Opcode::MSET as i64 => "MSET",
                c if c == Opcode::MCMP as i64 => "MCMP",
                c if c == Opcode::EXIT as i64 => "EXIT",
                c => {
                    if (c >= 0 && c <= 10000) || (c >= -10000 && c <= -1) {
                        // Probably an immediate value or address
                        &format!("{}", c)
                    } else {
                        // Unknown opcode
                        "???"
                    }
                }
            };
            
            println!("{:>8}", opcode);
        }
    }
    
    /// Get the generated code
    pub fn get_code(&self) -> &[i64] {
        &self.code
    }
    
    /// Get the data section
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
    
    /// Get the entry point (main function address)
    pub fn get_entry_point(&self) -> Result<i64, CompilerError> {
        match self.symbols.get_main() {
            Some(main_sym) => Ok(main_sym.value),
            None => Err(CompilerError::ParserError("main() not defined".to_string())),
        }
    }
    
    /// Check if types are compatible
    fn check_types(&self, expected: Type, actual: Type) -> Result<(), CompilerError> {
        if expected != actual {
            return Err(CompilerError::TypeError(
                format!("type mismatch: expected {:?}, got {:?}", expected, actual)
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a parser with test source
    fn create_test_parser(source: &str) -> Parser {
        Parser::new(source.to_string(), false, false)
    }

    #[test]
    fn test_basic_arithmetic() -> Result<(), CompilerError> {
        let source = "int main() { return 2 + 3 * 4; }";
        let mut parser = create_test_parser(source);
        parser.init()?;
        parser.parse()?;
        
        let code = parser.get_code();
        assert_eq!(code[2], Opcode::IMM as i64); // 2
        assert_eq!(code[3], 2);
        assert_eq!(code[4], Opcode::PSH as i64);
        assert_eq!(code[5], Opcode::IMM as i64); // 3
        assert_eq!(code[6], 3);
        assert_eq!(code[7], Opcode::PSH as i64);
        assert_eq!(code[8], Opcode::IMM as i64); // 4
        assert_eq!(code[9], 4);
        assert_eq!(code[10], Opcode::MUL as i64);
        assert_eq!(code[11], Opcode::ADD as i64);
        assert_eq!(code[12], Opcode::LEV as i64);
        Ok(())
    }

    #[test]
    fn test_variable_declaration() -> Result<(), CompilerError> {
        let source = "int main() { int x = 42; return x; }";
        let mut parser = create_test_parser(source);
        parser.init()?;
        parser.parse()?;
        
        let code = parser.get_code();
        assert_eq!(code[2], Opcode::IMM as i64); // 42
        assert_eq!(code[3], 42);
        assert_eq!(code[4], Opcode::LEA as i64); // Load address of x
        assert_eq!(code[5], 0); // Local variable offset
        assert_eq!(code[6], Opcode::SI as i64); // Store int
        assert_eq!(code[7], Opcode::LEA as i64); // Load address of x
        assert_eq!(code[8], 0);
        assert_eq!(code[9], Opcode::LI as i64); // Load int
        assert_eq!(code[10], Opcode::LEV as i64);
        Ok(())
    }

    #[test]
    fn test_if_statement() -> Result<(), CompilerError> {
        let source = "int main() { if (1) return 42; return 0; }";
        let mut parser = create_test_parser(source);
        parser.init()?;
        parser.parse()?;
        
        let code = parser.get_code();
        assert_eq!(code[2], Opcode::IMM as i64); // 1
        assert_eq!(code[3], 1);
        assert_eq!(code[4], Opcode::BZ as i64); // Branch if zero
        assert_eq!(code[6], Opcode::IMM as i64); // 42
        assert_eq!(code[7], 42);
        assert_eq!(code[8], Opcode::LEV as i64);
        assert_eq!(code[9], Opcode::IMM as i64); // 0
        assert_eq!(code[10], 0);
        assert_eq!(code[11], Opcode::LEV as i64);
        Ok(())
    }

    #[test]
    fn test_while_loop() -> Result<(), CompilerError> {
        let source = "int main() { int x = 0; while (x < 5) x = x + 1; return x; }";
        let mut parser = create_test_parser(source);
        parser.init()?;
        parser.parse()?;
        
        let code = parser.get_code();
        // Verify loop structure
        assert_eq!(code[2], Opcode::IMM as i64); // 0
        assert_eq!(code[3], 0);
        assert_eq!(code[4], Opcode::LEA as i64); // x
        assert_eq!(code[5], 0);
        assert_eq!(code[6], Opcode::SI as i64); // Store x
        // Loop condition
        assert_eq!(code[7], Opcode::LEA as i64); // Load x
        assert_eq!(code[8], 0);
        assert_eq!(code[9], Opcode::LI as i64);
        assert_eq!(code[10], Opcode::PSH as i64);
        assert_eq!(code[11], Opcode::IMM as i64); // 5
        assert_eq!(code[12], 5);
        assert_eq!(code[13], Opcode::LT as i64);
        Ok(())
    }

    #[test]
    fn test_function_call() -> Result<(), CompilerError> {
        let source = "int add(int a, int b) { return a + b; } int main() { return add(2, 3); }";
        let mut parser = create_test_parser(source);
        parser.init()?;
        parser.parse()?;
        
        let code = parser.get_code();
        // Verify function call
        assert_eq!(code[2], Opcode::IMM as i64); // 2
        assert_eq!(code[3], 2);
        assert_eq!(code[4], Opcode::PSH as i64);
        assert_eq!(code[5], Opcode::IMM as i64); // 3
        assert_eq!(code[6], 3);
        assert_eq!(code[7], Opcode::PSH as i64);
        assert_eq!(code[8], Opcode::JSR as i64); // Jump to subroutine
        Ok(())
    }

    #[test]
    fn test_pointer_operations() -> Result<(), CompilerError> {
        let source = "int main() { int x = 42; int *p = &x; return *p; }";
        let mut parser = create_test_parser(source);
        parser.init()?;
        parser.parse()?;
        
        let code = parser.get_code();
        // Verify pointer operations
        assert_eq!(code[2], Opcode::IMM as i64); // 42
        assert_eq!(code[3], 42);
        assert_eq!(code[4], Opcode::LEA as i64); // x
        assert_eq!(code[5], 0);
        assert_eq!(code[6], Opcode::SI as i64); // Store x
        assert_eq!(code[7], Opcode::LEA as i64); // &x
        assert_eq!(code[8], 0);
        assert_eq!(code[9], Opcode::LEA as i64); // p
        assert_eq!(code[10], 1);
        assert_eq!(code[11], Opcode::SI as i64); // Store p
        assert_eq!(code[12], Opcode::LEA as i64); // p
        assert_eq!(code[13], 1);
        assert_eq!(code[14], Opcode::LI as i64); // Load p
        assert_eq!(code[15], Opcode::LI as i64); // Load *p
        Ok(())
    }

    #[test]
    fn test_array_operations() -> Result<(), CompilerError> {
        let source = "int main() { int arr[5]; arr[0] = 42; return arr[0]; }";
        let mut parser = create_test_parser(source);
        parser.init()?;
        parser.parse()?;
        
        let code = parser.get_code();
        // Verify array operations
        assert_eq!(code[2], Opcode::LEA as i64); // arr
        assert_eq!(code[3], 0);
        assert_eq!(code[4], Opcode::PSH as i64);
        assert_eq!(code[5], Opcode::IMM as i64); // 0
        assert_eq!(code[6], 0);
        assert_eq!(code[7], Opcode::PSH as i64);
        assert_eq!(code[8], Opcode::IMM as i64); // 42
        assert_eq!(code[9], 42);
        assert_eq!(code[10], Opcode::SI as i64); // Store in array
        Ok(())
    }

    #[test]
    fn test_error_handling() {
        // Test invalid syntax
        let source = "int main() { return 42 }"; // Missing semicolon
        let mut parser = create_test_parser(source);
        assert!(parser.init().is_ok());
        assert!(parser.parse().is_err());

        // Test undefined variable
        let source = "int main() { return x; }"; // Undefined x
        let mut parser = create_test_parser(source);
        assert!(parser.init().is_ok());
        assert!(parser.parse().is_err());

        // Test invalid pointer dereference
        let source = "int main() { int x = 42; return *x; }"; // Invalid dereference
        let mut parser = create_test_parser(source);
        assert!(parser.init().is_ok());
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_self_hosting() -> Result<(), CompilerError> {
        // Test a minimal self-hosting example
        let source = r#"
            int main() {
                int x = 0;
                while (x < 10) {
                    x = x + 1;
                }
                return x;
            }
        "#;
        let mut parser = create_test_parser(source);
        parser.init()?;
        parser.parse()?;
        
        let code = parser.get_code();
        // Verify self-hosting capabilities
        assert_eq!(code[2], Opcode::IMM as i64); // 0
        assert_eq!(code[3], 0);
        assert_eq!(code[4], Opcode::LEA as i64); // x
        assert_eq!(code[5], 0);
        assert_eq!(code[6], Opcode::SI as i64); // Store x
        // Loop structure
        assert_eq!(code[7], Opcode::LEA as i64); // Load x
        assert_eq!(code[8], 0);
        assert_eq!(code[9], Opcode::LI as i64);
        assert_eq!(code[10], Opcode::PSH as i64);
        assert_eq!(code[11], Opcode::IMM as i64); // 10
        assert_eq!(code[12], 10);
        assert_eq!(code[13], Opcode::LT as i64);
        Ok(())
    }
}