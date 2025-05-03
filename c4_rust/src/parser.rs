use crate::error::CompilerError;
use crate::lexer::{Lexer, Token};
use crate::symbol::SymbolTable;
use crate::types::{Opcode, TokenType, Type};

/// Parser for the C4 compiler
pub struct Parser {
    lexer: Lexer,
    code: Vec<i64>,
    current: Token,
    symbols: SymbolTable,
    current_type: Type,
    current_class: TokenType,
    current_value: i64,
    current_bp: usize,
    current_loc: usize,
}

impl Parser {
    /// Create a new parser
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to parse
    /// * `print_source` - Whether to print source lines during processing
    pub fn new(source: String, print_source: bool) -> Self {
        Parser {
            lexer: Lexer::new(source, print_source),
            code: Vec::new(),
            current: Token {
                token_type: TokenType::Eof,
                value: None,
                name: None,
            },
            symbols: SymbolTable::new(),
            current_type: Type::INT,
            current_class: TokenType::Glo,
            current_value: 0,
            current_bp: 0,
            current_loc: 0,
        }
    }

    /// Initialize the parser
    pub fn init(&mut self) -> Result<(), CompilerError> {
        self.next_token()?;
        Ok(())
    }

    /// Parse the entire source code
    pub fn parse(&mut self) -> Result<(), CompilerError> {
        self.parse_program()?;
        Ok(())
    }

    /// Get the generated code
    pub fn get_code(&self) -> &[i64] {
        &self.code
    }

    /// Get the next token from the lexer
    fn next_token(&mut self) -> Result<(), CompilerError> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }

    /// Match and consume the current token if it matches the expected type
    fn match_token(&mut self, expected: TokenType) -> Result<(), CompilerError> {
        if self.current.token_type == expected {
            self.next_token()?;
            Ok(())
        } else {
            Err(CompilerError::ParserError(
                format!("Expected {:?}, got {:?}", expected, self.current.token_type)
            ))
        }
    }

    /// Parse the program
    fn parse_program(&mut self) -> Result<(), CompilerError> {
        while self.current.token_type != TokenType::Eof {
            self.parse_declaration()?;
        }
        Ok(())
    }

    /// Parse a declaration (variable or function)
    fn parse_declaration(&mut self) -> Result<(), CompilerError> {
        // Parse type
        match self.current.token_type {
            TokenType::Int => {
                self.current_type = Type::INT;
                self.next_token()?;
            }
            TokenType::Char => {
                self.current_type = Type::CHAR;
                self.next_token()?;
            }
            _ => return Err(CompilerError::ParserError("Expected type".to_string())),
        }

        // Parse identifier
        if self.current.token_type != TokenType::Id {
            return Err(CompilerError::ParserError("Expected identifier".to_string()));
        }
        let name = self.current.name.clone().unwrap();
        self.next_token()?;

        // Parse function or variable
        if self.current.token_type == TokenType::LParen {
            self.parse_function(&name)?;
        } else {
            self.parse_variable(&name)?;
        }

        Ok(())
    }

    /// Parse a function declaration
    fn parse_function(&mut self, name: &str) -> Result<(), CompilerError> {
        self.current_class = TokenType::Fun;
        self.current_bp = self.code.len();
        self.current_loc = 0;

        // Add function to symbol table
        self.symbols.add(name, TokenType::Fun, self.current_type, self.current_bp as i64);

        self.match_token(TokenType::LParen)?;
        self.parse_parameters()?;
        self.match_token(TokenType::RParen)?;
        self.match_token(TokenType::LBrace)?;

        // Generate function prologue
        self.emit(Opcode::ENT as i64);
        self.emit(0); // Space for locals

        // Parse function body
        while self.current.token_type != TokenType::RBrace {
            self.parse_statement()?;
        }
        self.match_token(TokenType::RBrace)?;

        // Generate function epilogue
        self.emit(Opcode::LEV as i64);

        Ok(())
    }

    /// Parse function parameters
    fn parse_parameters(&mut self) -> Result<(), CompilerError> {
        if self.current.token_type == TokenType::Int || self.current.token_type == TokenType::Char {
            let param_type = if self.current.token_type == TokenType::Int {
                Type::INT
            } else {
                Type::CHAR
            };
            self.next_token()?;

            if self.current.token_type != TokenType::Id {
                return Err(CompilerError::ParserError("Expected parameter name".to_string()));
            }
            let name = self.current.name.clone().unwrap();
            self.next_token()?;

            self.symbols.add(&name, TokenType::Loc, param_type, self.current_loc as i64);
            self.current_loc += param_type.size();

            while self.current.token_type == TokenType::Comma {
                self.next_token()?;
                self.parse_parameter()?;
            }
        }
        Ok(())
    }

    /// Parse a single parameter
    fn parse_parameter(&mut self) -> Result<(), CompilerError> {
        let param_type = if self.current.token_type == TokenType::Int {
            Type::INT
        } else if self.current.token_type == TokenType::Char {
            Type::CHAR
        } else {
            return Err(CompilerError::ParserError("Expected parameter type".to_string()));
        };
        self.next_token()?;

        if self.current.token_type != TokenType::Id {
            return Err(CompilerError::ParserError("Expected parameter name".to_string()));
        }
        let name = self.current.name.clone().unwrap();
        self.next_token()?;

        self.symbols.add(&name, TokenType::Loc, param_type, self.current_loc as i64);
        self.current_loc += param_type.size();

        Ok(())
    }

    /// Parse a variable declaration
    fn parse_variable(&mut self, name: &str) -> Result<(), CompilerError> {
        let mut size = 1;
        if self.current.token_type == TokenType::Brak {
            self.next_token()?;
            if self.current.token_type != TokenType::Num {
                return Err(CompilerError::ParserError("Expected array size".to_string()));
            }
            size = self.current.value.unwrap() as usize;
            self.next_token()?;
            self.match_token(TokenType::RBracket)?;
        }

        if self.current_class == TokenType::Glo {
            self.symbols.add(name, TokenType::Glo, self.current_type, self.current_value);
            self.current_value += (size * self.current_type.size()) as i64;
        } else {
            self.symbols.add(name, TokenType::Loc, self.current_type, self.current_loc as i64);
            self.current_loc += size * self.current_type.size();
        }

        if self.current.token_type == TokenType::Assign {
            self.next_token()?;
            self.parse_expression()?;
            self.emit(Opcode::SI as i64);
        }

        self.match_token(TokenType::Semicolon)?;
        Ok(())
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<(), CompilerError> {
        match self.current.token_type {
            TokenType::If => self.parse_if()?,
            TokenType::While => self.parse_while()?,
            TokenType::Return => self.parse_return()?,
            TokenType::LBrace => self.parse_block()?,
            _ => self.parse_expression_statement()?,
        }
        Ok(())
    }

    /// Parse an if statement
    fn parse_if(&mut self) -> Result<(), CompilerError> {
        self.next_token()?;
        self.match_token(TokenType::LParen)?;
        self.parse_expression()?;
        self.match_token(TokenType::RParen)?;

        let if_pos = self.emit(Opcode::BZ as i64);
        self.emit(0); // Placeholder for jump address

        self.parse_statement()?;

        if self.current.token_type == TokenType::Else {
            let else_pos = self.emit(Opcode::JMP as i64);
            self.emit(0); // Placeholder for jump address

            // Update if jump
            self.code[if_pos + 1] = self.code.len() as i64;

            self.next_token()?;
            self.parse_statement()?;

            // Update else jump
            self.code[else_pos + 1] = self.code.len() as i64;
        } else {
            // Update if jump
            self.code[if_pos + 1] = self.code.len() as i64;
        }

        Ok(())
    }

    /// Parse a while statement
    fn parse_while(&mut self) -> Result<(), CompilerError> {
        self.next_token()?;
        let while_pos = self.code.len();
        self.match_token(TokenType::LParen)?;
        self.parse_expression()?;
        self.match_token(TokenType::RParen)?;

        let exit_pos = self.emit(Opcode::BZ as i64);
        self.emit(0); // Placeholder for jump address

        self.parse_statement()?;

        // Jump back to condition
        self.emit(Opcode::JMP as i64);
        self.emit(while_pos as i64);

        // Update exit jump
        self.code[exit_pos + 1] = self.code.len() as i64;

        Ok(())
    }

    /// Parse a return statement
    fn parse_return(&mut self) -> Result<(), CompilerError> {
        self.next_token()?;
        if self.current.token_type != TokenType::Semicolon {
            self.parse_expression()?;
        }
        self.emit(Opcode::LEV as i64);
        self.match_token(TokenType::Semicolon)?;
        Ok(())
    }

    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<(), CompilerError> {
        self.match_token(TokenType::LBrace)?;
        while self.current.token_type != TokenType::RBrace {
            self.parse_statement()?;
        }
        self.match_token(TokenType::RBrace)?;
        Ok(())
    }

    /// Parse an expression statement
    fn parse_expression_statement(&mut self) -> Result<(), CompilerError> {
        self.parse_expression()?;
        self.emit(Opcode::ADJ as i64);
        self.emit(1);
        self.match_token(TokenType::Semicolon)?;
        Ok(())
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> Result<(), CompilerError> {
        self.parse_assignment()?;
        Ok(())
    }

    /// Parse an assignment expression
    fn parse_assignment(&mut self) -> Result<(), CompilerError> {
        let left = self.parse_equality()?;
        if self.current.token_type == TokenType::Assign {
            self.next_token()?;
            self.parse_assignment()?;
            self.emit(Opcode::SI as i64);
        }
        Ok(left)
    }

    /// Parse an equality expression
    fn parse_equality(&mut self) -> Result<(), CompilerError> {
        let left = self.parse_relational()?;
        while self.current.token_type == TokenType::Eq || self.current.token_type == TokenType::Ne {
            let op = self.current.token_type;
            self.next_token()?;
            self.parse_relational()?;
            match op {
                TokenType::Eq => { let _ = self.emit(Opcode::EQ as i64); },
                TokenType::Ne => { let _ = self.emit(Opcode::NE as i64); },
                _ => unreachable!(),
            }
        }
        Ok(left)
    }

    /// Parse a relational expression
    fn parse_relational(&mut self) -> Result<(), CompilerError> {
        let left = self.parse_additive()?;
        while matches!(self.current.token_type, 
            TokenType::Lt | TokenType::Gt | TokenType::Le | TokenType::Ge) {
            let op = self.current.token_type;
            self.next_token()?;
            self.parse_additive()?;
            match op {
                TokenType::Lt => { let _ = self.emit(Opcode::LT as i64); },
                TokenType::Gt => { let _ = self.emit(Opcode::GT as i64); },
                TokenType::Le => { let _ = self.emit(Opcode::LE as i64); },
                TokenType::Ge => { let _ = self.emit(Opcode::GE as i64); },
                _ => unreachable!(),
            }
        }
        Ok(left)
    }

    /// Parse an additive expression
    fn parse_additive(&mut self) -> Result<(), CompilerError> {
        let left = self.parse_multiplicative()?;
        while self.current.token_type == TokenType::Add || self.current.token_type == TokenType::Sub {
            let op = self.current.token_type;
            self.next_token()?;
            self.parse_multiplicative()?;
            match op {
                TokenType::Add => { let _ = self.emit(Opcode::ADD as i64); },
                TokenType::Sub => { let _ = self.emit(Opcode::SUB as i64); },
                _ => unreachable!(),
            }
        }
        Ok(left)
    }

    /// Parse a multiplicative expression
    fn parse_multiplicative(&mut self) -> Result<(), CompilerError> {
        let left = self.parse_unary()?;
        while matches!(self.current.token_type, 
            TokenType::Mul | TokenType::Div | TokenType::Mod) {
            let op = self.current.token_type;
            self.next_token()?;
            self.parse_unary()?;
            match op {
                TokenType::Mul => { let _ = self.emit(Opcode::MUL as i64); },
                TokenType::Div => { let _ = self.emit(Opcode::DIV as i64); },
                TokenType::Mod => { let _ = self.emit(Opcode::MOD as i64); },
                _ => unreachable!(),
            }
        }
        Ok(left)
    }

    /// Parse a unary expression
    fn parse_unary(&mut self) -> Result<(), CompilerError> {
        match self.current.token_type {
            TokenType::Add => {
                self.next_token()?;
                self.parse_unary()
            }
            TokenType::Sub => {
                self.next_token()?;
                self.parse_unary()?;
                self.emit(Opcode::IMM as i64);
                self.emit(-1);
                self.emit(Opcode::MUL as i64);
                Ok(())
            }
            TokenType::Not => {
                self.next_token()?;
                self.parse_unary()?;
                self.emit(Opcode::IMM as i64);
                self.emit(0);
                self.emit(Opcode::EQ as i64);
                Ok(())
            }
            TokenType::BitNot => {
                self.next_token()?;
                self.parse_unary()?;
                self.emit(Opcode::IMM as i64);
                self.emit(-1);
                self.emit(Opcode::XOR as i64);
                Ok(())
            }
            _ => self.parse_primary(),
        }
    }

    /// Parse a primary expression
    fn parse_primary(&mut self) -> Result<(), CompilerError> {
        match self.current.token_type {
            TokenType::Num => {
                self.emit(Opcode::IMM as i64);
                self.emit(self.current.value.unwrap());
                self.next_token()?;
                Ok(())
            }
            TokenType::Id => {
                let name = self.current.name.clone().unwrap();
                self.next_token()?;

                if self.current.token_type == TokenType::LParen {
                    // Function call
                    self.next_token()?;
                    let arg_count = self.parse_arguments()?;
                    self.match_token(TokenType::RParen)?;

                    if let Some(sym) = self.symbols.get(&name) {
                        if sym.class != TokenType::Fun {
                            return Err(CompilerError::ParserError("Not a function".to_string()));
                        }
                        self.emit(Opcode::JSR as i64);
                        self.emit(sym.value);
                        self.emit(Opcode::ADJ as i64);
                        self.emit(arg_count as i64);
                    } else {
                        return Err(CompilerError::ParserError("Undefined function".to_string()));
                    }
                } else {
                    // Variable
                    if let Some(sym) = self.symbols.get(&name) {
                        match sym.class {
                            TokenType::Glo => {
                                self.emit(Opcode::IMM as i64);
                                self.emit(sym.value);
                            }
                            TokenType::Loc => {
                                self.emit(Opcode::LEA as i64);
                                self.emit(sym.value);
                            }
                            _ => return Err(CompilerError::ParserError("Invalid variable class".to_string())),
                        }
                        self.emit(Opcode::LI as i64);
                    } else {
                        return Err(CompilerError::ParserError("Undefined variable".to_string()));
                    }
                }
                Ok(())
            }
            TokenType::LParen => {
                self.next_token()?;
                self.parse_expression()?;
                self.match_token(TokenType::RParen)?;
                Ok(())
            }
            _ => Err(CompilerError::ParserError("Unexpected token".to_string())),
        }
    }

    /// Parse function call arguments
    fn parse_arguments(&mut self) -> Result<usize, CompilerError> {
        let mut count = 0;
        if self.current.token_type != TokenType::RParen {
            loop {
                self.parse_expression()?;
                self.emit(Opcode::PSH as i64);
                count += 1;

                if self.current.token_type == TokenType::Comma {
                    self.next_token()?;
                } else {
                    break;
                }
            }
        }
        Ok(count)
    }

    /// Emit a value to the code segment
    fn emit(&mut self, value: i64) -> usize {
        self.code.push(value);
        self.code.len() - 1
    }
}