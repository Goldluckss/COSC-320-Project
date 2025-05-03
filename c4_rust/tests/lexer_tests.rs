use c4_rust::error::CompilerError;
use c4_rust::lexer::Lexer;
use c4_rust::symbol::SymbolTable;
use c4_rust::types::{TokenType, Type, Opcode};
use c4_rust::vm::VirtualMachine;

#[test]
fn test_simple_expression_vm() -> Result<(), CompilerError> {
    // A simple program that adds two numbers using the VM directly
    let code = vec![
        Opcode::IMM as i64, 5,     // Load 5
        Opcode::PSH as i64,        // Push 5
        Opcode::IMM as i64, 7,     // Load 7
        Opcode::ADD as i64,        // Add 5+7
        Opcode::EXIT as i64,       // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 12); // 5 + 7 = 12
    Ok(())
}

#[test]
fn test_lexer_tokenize_expression() -> Result<(), CompilerError> {
    // Tokenize a simple expression
    let source = "int x = 5 + 7;".to_string();
    let mut lexer = Lexer::new(source, false);
    
    // Verify tokens
    assert_eq!(lexer.next_token()?.token_type, TokenType::Int);
    assert_eq!(lexer.next_token()?.token_type, TokenType::Id);
    assert_eq!(lexer.next_token()?.token_type, TokenType::Assign);
    
    let token = lexer.next_token()?;
    assert_eq!(token.token_type, TokenType::Num);
    assert_eq!(token.value.unwrap(), 5);
    
    assert_eq!(lexer.next_token()?.token_type, TokenType::Add);
    
    let token = lexer.next_token()?;
    assert_eq!(token.token_type, TokenType::Num);
    assert_eq!(token.value.unwrap(), 7);
    
    assert_eq!(lexer.next_token()?.token_type, TokenType::Semicolon);
    
    Ok(())
}

#[test]
fn test_lexer_and_symbol_table() -> Result<(), CompilerError> {
    // A simple program with variables and scopes
    let source = r#"
        int global;
        
        int main() {
            int local = 42;
            {
                int block = 12;
                global = local + block;
            }
            return global;
        }
    "#.to_string();
    
    // Tokenize and build symbol table
    let mut lexer = Lexer::new(source.clone(), false);
    let mut symbol_table = SymbolTable::new();
    
    // Process tokens and build symbol table
    loop {
        let token = lexer.next_token()?;
        
        match token.token_type {
            TokenType::Eof => break,
            
            TokenType::Int => {
                // This could be a variable declaration or function
                let id_token = lexer.next_token()?;
                if id_token.token_type == TokenType::Id {
                    let id_name = id_token.name.unwrap();
                    
                    // Check for function declaration
                    let next_token = lexer.next_token()?;
                    if next_token.token_type == TokenType::LParen {
                        // Function
                        symbol_table.add(&id_name, TokenType::Fun, Type::INT, 0);
                        symbol_table.enter_scope(); // Enter function scope
                    } else {
                        // Variable
                        if symbol_table.get_symbols().len() == 0 || symbol_table.get_scope_count() == 1 {
                            // Global variable
                            symbol_table.add(&id_name, TokenType::Glo, Type::INT, 0);
                        } else {
                            // Local variable
                            symbol_table.add(&id_name, TokenType::Loc, Type::INT, 0);
                        }
                    }
                }
            },
            
            TokenType::LBrace => {
                // Enter a new block scope
                symbol_table.enter_scope();
            },
            
            TokenType::RBrace => {
                // Exit block scope
                symbol_table.exit_scope();
            },
            
            _ => {
                // Skip other tokens for this test
            }
        }
    }
    
    // Check the symbol table
    assert!(symbol_table.exists("global"));
    assert!(symbol_table.exists("main"));
    
    // Should be back at global scope
    assert_eq!(symbol_table.get_scope_count(), 1);
    
    Ok(())
}

// Function call test with smaller code size to avoid out of bounds error
#[test]
fn test_function_call_simulation() -> Result<(), CompilerError> {
    // Simulate a function call with the VM
    let code = vec![
        // Main function (caller)
        Opcode::IMM as i64, 5,      // First argument
        Opcode::PSH as i64,
        Opcode::IMM as i64, 3,      // Second argument
        Opcode::PSH as i64,
        Opcode::JSR as i64, 9,      // Call function at index 9
        Opcode::ADJ as i64, 2,      // Clean up arguments (2 args)
        Opcode::EXIT as i64,        // Return from main
        
        // Add function (callee) at index 9
        Opcode::ENT as i64, 0,      // Setup stack frame (no locals)
        Opcode::LEA as i64, 2,      // Load address of first argument
        Opcode::LI as i64,          // Load value
        Opcode::LEA as i64, 1,      // Load address of second argument
        Opcode::LI as i64,          // Load value
        Opcode::ADD as i64,         // Add the arguments
        Opcode::LEV as i64,         // Return from function
    ];
    
    // Execute the code
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 8); // 5 + 3 = 8
    Ok(())
}