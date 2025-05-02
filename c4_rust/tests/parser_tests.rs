use c4_rust::error::CompilerError;
use c4_rust::parser::Parser;
use c4_rust::types::Opcode;

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