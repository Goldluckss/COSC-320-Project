use c4_rust::error::CompilerError;
use c4_rust::lexer::Lexer;
use c4_rust::parser::Parser;
use c4_rust::symbol::SymbolTable;
use c4_rust::types::{TokenType, Type, Opcode};
use c4_rust::vm::VirtualMachine;

// ==================== Lexer Tests ====================

/// Test basic tokenization of a simple C program
#[test]
fn test_basic_lexer() -> Result<(), CompilerError> {
    let source = "int main() { return 42; }".to_string();
    let mut lexer = Lexer::new(source, false);
    
    // Check each token
    assert_eq!(lexer.next_token()?.token_type, TokenType::Int);
    assert_eq!(lexer.next_token()?.token_type, TokenType::Id); // main
    assert_eq!(lexer.next_token()?.token_type, TokenType::LParen);
    assert_eq!(lexer.next_token()?.token_type, TokenType::RParen);
    assert_eq!(lexer.next_token()?.token_type, TokenType::LBrace);
    assert_eq!(lexer.next_token()?.token_type, TokenType::Return);
    
    let token = lexer.next_token()?;
    assert_eq!(token.token_type, TokenType::Num);
    assert_eq!(token.value.unwrap(), 42);
    
    assert_eq!(lexer.next_token()?.token_type, TokenType::Semicolon);
    assert_eq!(lexer.next_token()?.token_type, TokenType::RBrace);
    assert_eq!(lexer.next_token()?.token_type, TokenType::Eof);
    
    Ok(())
}

/// Test string literal handling
#[test]
fn test_string_literal() -> Result<(), CompilerError> {
    let source = r#"char *s = "Hello, World!";"#.to_string();
    let mut lexer = Lexer::new(source, false);
    
    assert_eq!(lexer.next_token()?.token_type, TokenType::Char);
    assert_eq!(lexer.next_token()?.token_type, TokenType::Mul);
    assert_eq!(lexer.next_token()?.token_type, TokenType::Id); // s
    assert_eq!(lexer.next_token()?.token_type, TokenType::Assign);
    
    let token = lexer.next_token()?;
    assert_eq!(token.token_type, TokenType::Str);
    assert_eq!(token.name.unwrap(), "Hello, World!");
    
    assert_eq!(lexer.next_token()?.token_type, TokenType::Semicolon);
    
    Ok(())
}

// ==================== Parser Tests ====================

/// Test parsing a simple variable declaration
#[test]
fn test_variable_declaration() -> Result<(), CompilerError> {
    let source = "int x = 42;".to_string();
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // The parser should have created a global variable in its symbol table
    let data = parser.get_data();
    
    // First 8 bytes should contain 42 (on little-endian systems)
    let value = data[0] as i64 | 
                ((data[1] as i64) << 8) |
                ((data[2] as i64) << 16) |
                ((data[3] as i64) << 24) |
                ((data[4] as i64) << 32) |
                ((data[5] as i64) << 40) |
                ((data[6] as i64) << 48) |
                ((data[7] as i64) << 56);
    
    assert_eq!(value, 42);
    
    Ok(())
}

/// Test parsing a simple function
#[test]
fn test_function_parsing() -> Result<(), CompilerError> {
    let source = "int add(int a, int b) { return a + b; }".to_string();
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // The code should include operations to add two parameters
    let code = parser.get_code();
    
    // Test essential instructions are present
    // (We don't check exact positions as they may vary, just that they exist in the right order)
    let mut has_lea_a = false;
    let mut has_li_a = false;
    let mut has_lea_b = false;
    let mut has_li_b = false;
    let mut has_add = false;
    let mut has_lev = false;
    
    for i in 0..code.len() {
        if i + 1 < code.len() && code[i] == Opcode::LEA as i64 && code[i+1] == 2 {
            has_lea_a = true;
        }
        if code[i] == Opcode::LI as i64 && has_lea_a && !has_li_a {
            has_li_a = true;
        }
        if i + 1 < code.len() && code[i] == Opcode::LEA as i64 && code[i+1] == 1 {
            has_lea_b = true;
        }
        if code[i] == Opcode::LI as i64 && has_lea_b && !has_li_b {
            has_li_b = true;
        }
        if code[i] == Opcode::ADD as i64 {
            has_add = true;
        }
        if code[i] == Opcode::LEV as i64 {
            has_lev = true;
        }
    }
    
    assert!(has_lea_a, "Missing LEA instruction for parameter a");
    assert!(has_li_a, "Missing LI instruction for parameter a");
    assert!(has_lea_b, "Missing LEA instruction for parameter b");
    assert!(has_li_b, "Missing LI instruction for parameter b");
    assert!(has_add, "Missing ADD instruction");
    assert!(has_lev, "Missing LEV instruction");
    
    Ok(())
}

// ==================== VM Tests ====================

/// Test basic VM operation with addition
#[test]
fn test_vm_addition() -> Result<(), CompilerError> {
    // Create code that adds 5 + 7
    let code = vec![
        Opcode::IMM as i64, 5,     // Load 5
        Opcode::PSH as i64,        // Push 5 onto stack
        Opcode::IMM as i64, 7,     // Load 7
        Opcode::ADD as i64,        // Add: 5 + 7 = 12
        Opcode::EXIT as i64,       // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 12);  // 5 + 7 = 12
    Ok(())
}

/// Test VM function call
#[test]
fn test_vm_function_call() -> Result<(), CompilerError> {
    // Create code for a main function that calls a function to add two numbers
    let code = vec![
        // Main function
        Opcode::IMM as i64, 10,     // First argument
        Opcode::PSH as i64,
        Opcode::IMM as i64, 20,     // Second argument
        Opcode::PSH as i64,
        Opcode::JSR as i64, 8,      // Call function at offset 8
        Opcode::ADJ as i64, 2,      // Clean up arguments (2 args)
        Opcode::EXIT as i64,        // Exit program
        
        // Add function (offset 8)
        Opcode::ENT as i64, 0,      // Setup stack frame
        Opcode::LEA as i64, 2,      // Load address of first argument
        Opcode::LI as i64,          // Load value
        Opcode::LEA as i64, 1,      // Load address of second argument
        Opcode::LI as i64,          // Load value
        Opcode::ADD as i64,         // Add values
        Opcode::LEV as i64,         // Return to caller
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 30);  // 10 + 20 = 30
    Ok(())
}

// ==================== Integration Tests ====================

/// Test compiling and running a simple program
#[test]
fn test_compile_and_run() -> Result<(), CompilerError> {
    // A simple program that returns 42
    let source = r#"
        int main() {
            return 42;
        }
    "#.to_string();
    
    // Parse and compile
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Get the bytecode
    let code = parser.get_code();
    let data = parser.get_data();
    
    // Get the main function
    let main_offset = parser.get_main_function().expect("main function not found");
    
    // Run the program
    let mut vm = VirtualMachine::new(code.to_vec(), data.to_vec(), 1024, false);
    let result = vm.run(main_offset, &[])?;
    
    // Check the result
    assert_eq!(result, 42);
    
    Ok(())
}

/// Test compiling and running a program with arithmetic
#[test]
fn test_arithmetic_program() -> Result<(), CompilerError> {
    // A program that performs basic arithmetic
    let source = r#"
        int main() {
            int a = 10;
            int b = 5;
            
            // Addition
            int c = a + b;     // 15
            
            // Subtraction
            int d = a - b;     // 5
            
            // Multiplication
            int e = a * b;     // 50
            
            // Division
            int f = a / b;     // 2
            
            return c + d + e + f;  // 15 + 5 + 50 + 2 = 72
        }
    "#.to_string();
    
    // Parse and compile
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Get the bytecode
    let code = parser.get_code();
    let data = parser.get_data();
    
    // Get the main function
    let main_offset = parser.get_main_function().expect("main function not found");
    
    // Run the program
    let mut vm = VirtualMachine::new(code.to_vec(), data.to_vec(), 1024, false);
    let result = vm.run(main_offset, &[])?;
    
    // Check the result (15 + 5 + 50 + 2 = 72)
    assert_eq!(result, 72);
    
    Ok(())
}

/// Test compiling and running a program with if statement
#[test]
fn test_if_statement() -> Result<(), CompilerError> {
    // A program with an if statement
    let source = r#"
        int main() {
            int x = 10;
            
            if (x > 5) {
                return 42;
            } else {
                return 24;
            }
        }
    "#.to_string();
    
    // Parse and compile
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Get the bytecode
    let code = parser.get_code();
    let data = parser.get_data();
    
    // Get the main function
    let main_offset = parser.get_main_function().expect("main function not found");
    
    // Run the program
    let mut vm = VirtualMachine::new(code.to_vec(), data.to_vec(), 1024, false);
    let result = vm.run(main_offset, &[])?;
    
    // Check the result (should take the true branch)
    assert_eq!(result, 42);
    
    Ok(())
}

/// Test compiling and running a program with a while loop
#[test]
fn test_while_loop() -> Result<(), CompilerError> {
    // A program with a while loop that sums numbers from 1 to 10
    let source = r#"
        int main() {
            int sum = 0;
            int i = 1;
            
            while (i <= 10) {
                sum = sum + i;
                i = i + 1;
            }
            
            return sum;  // 1+2+3+4+5+6+7+8+9+10 = 55
        }
    "#.to_string();
    
    // Parse and compile
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Get the bytecode
    let code = parser.get_code();
    let data = parser.get_data();
    
    // Get the main function
    let main_offset = parser.get_main_function().expect("main function not found");
    
    // Run the program
    let mut vm = VirtualMachine::new(code.to_vec(), data.to_vec(), 1024, false);
    let result = vm.run(main_offset, &[])?;
    
    // Check the result (sum from 1 to 10 = 55)
    assert_eq!(result, 55);
    
    Ok(())
}

/// Test function call with parameters
#[test]
fn test_function_call() -> Result<(), CompilerError> {
    // A program with a function that adds two numbers
    let source = r#"
        int add(int a, int b) {
            return a + b;
        }
        
        int main() {
            return add(10, 20);  // 30
        }
    "#.to_string();
    
    // Parse and compile
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Get the bytecode
    let code = parser.get_code();
    let data = parser.get_data();
    
    // Get the main function
    let main_offset = parser.get_main_function().expect("main function not found");
    
    // Run the program
    let mut vm = VirtualMachine::new(code.to_vec(), data.to_vec(), 1024, false);
    let result = vm.run(main_offset, &[])?;
    
    // Check the result (10 + 20 = 30)
    assert_eq!(result, 30);
    
    Ok(())
}