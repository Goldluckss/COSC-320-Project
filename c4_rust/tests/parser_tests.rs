use c4_rust::error::CompilerError;
use c4_rust::parser::Parser;
use c4_rust::types::{Opcode, TokenType, Type};

/// Test basic parsing of a simple program
#[test]
fn test_basic_parsing() -> Result<(), CompilerError> {
    let source = "int main() { return 42; }";
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check that main function was defined
    let main_addr = parser.get_main_function();
    assert!(main_addr.is_some(), "Main function should be defined");
    
    // Check generated code
    let code = parser.get_code();
    
    // Expected bytecode pattern (simplified):
    // ENT 0      - Setup stack frame
    // IMM 42     - Load immediate value 42
    // LEV        - Return from function
    
    let has_ent = code.contains(&(Opcode::ENT as i64));
    let mut has_imm_42 = false;
    for i in 0..code.len() - 1 {
        if code[i] == Opcode::IMM as i64 && code[i + 1] == 42 {
            has_imm_42 = true;
            break;
        }
    }
    let has_lev = code.contains(&(Opcode::LEV as i64));
    
    assert!(has_ent, "Missing ENT instruction");
    assert!(has_imm_42, "Missing IMM 42 instruction sequence");
    assert!(has_lev, "Missing LEV instruction");
    
    Ok(())
}

/// Test variable declarations and assignments
#[test]
fn test_variable_declaration() -> Result<(), CompilerError> {
    let source = r#"
        int x;
        int y;
        
        int main() {
            x = 10;
            y = 20;
            return x + y;
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check data segment (should have space for x and y)
    let data = parser.get_data();
    
    // Expect at least 16 bytes (2 integers)
    assert!(data.len() >= 16, "Data segment should have space for 2 integers");
    
    // Check generated code
    let code = parser.get_code();
    
    // Look for common operations: load, add, return
    let mut has_imm_10 = false;
    let mut has_imm_20 = false;
    let has_add = code.contains(&(Opcode::ADD as i64));
    let has_lev = code.contains(&(Opcode::LEV as i64));
    
    for i in 0..code.len() - 1 {
        if code[i] == Opcode::IMM as i64 {
            if code[i + 1] == 10 {
                has_imm_10 = true;
            } else if code[i + 1] == 20 {
                has_imm_20 = true;
            }
        }
    }
    
    assert!(has_imm_10, "Missing IMM 10 instruction");
    assert!(has_imm_20, "Missing IMM 20 instruction");
    assert!(has_add, "Missing ADD instruction");
    assert!(has_lev, "Missing LEV instruction");
    
    Ok(())
}

/// Test if-else statement
#[test]
fn test_if_else() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            if (1) {
                return 42;
            } else {
                return 24;
            }
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check generated code
    let code = parser.get_code();
    
    // If-else should generate branch and jump instructions
    let has_bz = code.contains(&(Opcode::BZ as i64));
    let has_jmp = code.contains(&(Opcode::JMP as i64));
    let mut has_imm_42 = false;
    let mut has_imm_24 = false;
    
    for i in 0..code.len() - 1 {
        if code[i] == Opcode::IMM as i64 {
            if code[i + 1] == 42 {
                has_imm_42 = true;
            } else if code[i + 1] == 24 {
                has_imm_24 = true;
            }
        }
    }
    
    assert!(has_bz, "Missing BZ instruction for if condition");
    assert!(has_jmp, "Missing JMP instruction for else branch");
    assert!(has_imm_42, "Missing IMM 42 instruction");
    assert!(has_imm_24, "Missing IMM 24 instruction");
    
    Ok(())
}

/// Test while loop
#[test]
fn test_while_loop() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            int i;
            int sum;
            
            i = 1;
            sum = 0;
            
            while (i <= 5) {
                sum = sum + i;
                i = i + 1;
            }
            
            return sum;
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check generated code
    let code = parser.get_code();
    
    // While loop should generate branch and jump instructions
    let has_bz = code.contains(&(Opcode::BZ as i64));
    let has_jmp = code.contains(&(Opcode::JMP as i64));
    let has_add = code.contains(&(Opcode::ADD as i64));
    let has_le = code.contains(&(Opcode::LE as i64));
    
    assert!(has_bz, "Missing BZ instruction for while condition");
    assert!(has_jmp, "Missing JMP instruction for loop");
    assert!(has_add, "Missing ADD instruction");
    assert!(has_le, "Missing LE instruction");
    
    Ok(())
}

/// Test function calls
#[test]
fn test_function_call() -> Result<(), CompilerError> {
    let source = r#"
        int add(int a, int b) {
            return a + b;
        }
        
        int main() {
            return add(10, 20);
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check generated code
    let code = parser.get_code();
    
    // Function call should generate JSR and ADJ instructions
    let has_jsr = code.contains(&(Opcode::JSR as i64));
    let has_adj = code.contains(&(Opcode::ADJ as i64));
    let has_add = code.contains(&(Opcode::ADD as i64));
    
    assert!(has_jsr, "Missing JSR instruction for function call");
    assert!(has_adj, "Missing ADJ instruction for stack adjustment");
    assert!(has_add, "Missing ADD instruction in function body");
    
    Ok(())
}

/// Test recursive function
#[test]
fn test_recursion() -> Result<(), CompilerError> {
    let source = r#"
        int factorial(int n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        
        int main() {
            return factorial(5);
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check generated code
    let code = parser.get_code();
    
    // Recursive function should call itself
    let has_jsr = code.contains(&(Opcode::JSR as i64));
    let has_mul = code.contains(&(Opcode::MUL as i64));
    let has_sub = code.contains(&(Opcode::SUB as i64));
    let has_le = code.contains(&(Opcode::LE as i64));
    
    assert!(has_jsr, "Missing JSR instruction for recursive call");
    assert!(has_mul, "Missing MUL instruction");
    assert!(has_sub, "Missing SUB instruction");
    assert!(has_le, "Missing LE instruction");
    
    Ok(())
}

/// Test array operations
#[test]
fn test_arrays() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            int arr[3];
            
            arr[0] = 10;
            arr[1] = 20;
            arr[2] = 30;
            
            return arr[1];
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check generated code
    let code = parser.get_code();
    
    // Array operations should generate ADD and index operations
    let has_add = code.contains(&(Opcode::ADD as i64));
    let has_si = code.contains(&(Opcode::SI as i64));
    let has_li = code.contains(&(Opcode::LI as i64));
    
    assert!(has_add, "Missing ADD instruction for array indexing");
    assert!(has_si, "Missing SI instruction for array assignment");
    assert!(has_li, "Missing LI instruction for array access");
    
    Ok(())
}

/// Test pointer operations
#[test]
fn test_pointers() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            int x;
            int *p;
            
            x = 42;
            p = &x;
            
            return *p;
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check generated code
    let code = parser.get_code();
    
    // Pointer operations should generate LI and address operations
    let has_lea = code.contains(&(Opcode::LEA as i64));
    let has_li = code.contains(&(Opcode::LI as i64));
    
    assert!(has_lea, "Missing LEA instruction for address-of");
    assert!(has_li, "Missing LI instruction for dereference");
    
    Ok(())
}

/// Test enum declaration
#[test]
fn test_enum() -> Result<(), CompilerError> {
    let source = r#"
        enum Color { RED, GREEN, BLUE = 5, YELLOW };
        
        int main() {
            return BLUE;
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Check generated code
    let code = parser.get_code();
    
    // Should generate IMM 5 for BLUE
    let mut has_imm_5 = false;
    
    for i in 0..code.len() - 1 {
        if code[i] == Opcode::IMM as i64 && code[i + 1] == 5 {
            has_imm_5 = true;
            break;
        }
    }
    
    assert!(has_imm_5, "Missing IMM 5 instruction for enum value");
    
    Ok(())
}

/// Test operator precedence
#[test]
fn test_precedence() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            return 2 + 3 * 4;
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // The result should be 14, not 20, due to precedence
    // But this is hard to test directly from the bytecode
    // We'll check for multiplication and addition
    let code = parser.get_code();
    
    let has_mul = code.contains(&(Opcode::MUL as i64));
    let has_add = code.contains(&(Opcode::ADD as i64));
    
    assert!(has_mul, "Missing MUL instruction");
    assert!(has_add, "Missing ADD instruction");
    
    Ok(())
}

/// Test sizeof operator
#[test]
fn test_sizeof() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            return sizeof(int) + sizeof(char);
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // sizeof(int) should be 8 and sizeof(char) should be 1
    let code = parser.get_code();
    
    let mut has_imm_8 = false;
    let mut has_imm_1 = false;
    
    for i in 0..code.len() - 1 {
        if code[i] == Opcode::IMM as i64 {
            if code[i + 1] == 8 {
                has_imm_8 = true;
            } else if code[i + 1] == 1 {
                has_imm_1 = true;
            }
        }
    }
    
    assert!(has_imm_8 || has_imm_1, "Missing sizeof values");
    
    Ok(())
}

/// Test error handling
#[test]
fn test_parser_error() {
    // Missing semicolon
    let source = "int main() { return 42 }";
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init().unwrap();
    let result = parser.parse();
    
    assert!(result.is_err(), "Parser should detect syntax error");
    
    // Check error message
    match result {
        Err(CompilerError::ParserError { message, .. }) => {
            assert!(message.contains("semicolon") || message.contains("Semicolon"), 
                "Error message should mention semicolon, got: {}", message);
        },
        Err(err) => panic!("Expected ParserError, got: {:?}", err),
        Ok(_) => panic!("Expected error, got success"),
    }
}

/// Test parsing of a more complex program
#[test]
fn test_complex_program() -> Result<(), CompilerError> {
    let source = r#"
        int fibonacci(int n) {
            if (n <= 1) return n;
            return fibonacci(n-1) + fibonacci(n-2);
        }
        
        int main() {
            return fibonacci(10);
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    parser.parse()?;
    
    // Just check that it parses without errors
    
    Ok(())
}

/// Test that our parser can handle a simple snippet from the original C4
#[test]
fn test_c4_snippet() -> Result<(), CompilerError> {
    // A small snippet similar to what's in the original C4.c
    let source = r#"
        enum { CHAR, INT, PTR };
        
        int next() {
            char *pp;
            int tk;
            
            while (tk = *p) {
                ++p;
                if (tk == '\n') {
                    ++line;
                }
                else if (tk == '#') {
                    while (*p != 0 && *p != '\n') ++p;
                }
                else {
                    return tk;
                }
            }
            return 0;
        }
    "#;
    
    let mut parser = Parser::new(source.to_string(), false);
    
    parser.init()?;
    
    // Just check that it parses without errors
    let result = parser.parse();
    
    // It might fail because it's missing main, that's OK
    match result {
        Err(CompilerError::ParserError { message, .. }) => {
            assert!(message.contains("main"), 
                "Should only fail because of missing main, got: {}", message);
        },
        Ok(_) => {}, // Sometimes it might not fail, which is also fine
        Err(err) => panic!("Unexpected error: {:?}", err),
    }
    
    Ok(())
}