use c4_rust::error::CompilerError;
use c4_rust::parser::Parser;
use c4_rust::types::{TokenType, Type, Opcode};
use pretty_assertions::assert_eq;

// Add this to the Parser implementation in src/parser.rs:
// pub fn get_symbol_table(&self) -> &SymbolTable {
//     &self.symbol_table
// }

#[test]
fn test_parser_basic() -> Result<(), CompilerError> {
    // Test a minimal program
    let source = "int main() { return 42; }".to_string();
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Check that main function was defined
    let main_addr = parser.get_main_function();
    assert!(main_addr.is_some(), "main function should be defined");
    
    Ok(())
}

#[test]
fn test_parser_variables() -> Result<(), CompilerError> {
    // Test variable declarations with initialization
    let source = "int x = 42; char c = 'A';".to_string();
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Check data segment contains the initialized values
    let data = parser.get_data();
    assert!(!data.is_empty(), "Data segment should not be empty");
    
    // First 8 bytes should contain 42 (integer)
    let x_value = data[0] as i64 | 
                 ((data[1] as i64) << 8) |
                 ((data[2] as i64) << 16) |
                 ((data[3] as i64) << 24) |
                 ((data[4] as i64) << 32) |
                 ((data[5] as i64) << 40) |
                 ((data[6] as i64) << 48) |
                 ((data[7] as i64) << 56);
    
    assert_eq!(x_value, 42);
    
    // Next byte should contain 'A'
    assert_eq!(data[8], b'A');
    
    Ok(())
}

#[test]
fn test_parser_enum() -> Result<(), CompilerError> {
    // Test enum declaration
    let source = "enum Color { RED, GREEN, BLUE = 5, YELLOW };".to_string();
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    Ok(())
}

#[test]
fn test_parser_function() -> Result<(), CompilerError> {
    // Test function declaration and body
    let source = "int add(int a, int b) { return a + b; }".to_string();
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Check generated code for key instructions
    let code = parser.get_code();
    assert!(!code.is_empty(), "Code segment should not be empty");
    
    // Should have ADD and LEV opcodes
    let has_add = code.iter().any(|&op| op == Opcode::ADD as i64);
    let has_lev = code.iter().any(|&op| op == Opcode::LEV as i64);
    
    assert!(has_add, "Function should have an ADD instruction");
    assert!(has_lev, "Function should have a LEV (return) instruction");
    
    Ok(())
}

#[test]
fn test_parser_control_flow() -> Result<(), CompilerError> {
    // Test if and while statements
    let source = r#"
        int test() {
            int a = 0;
            
            if (a == 0) {
                a = 1;
            } else {
                a = 2;
            }
            
            while (a < 5) {
                a = a + 1;
            }
            
            return a;
        }
    "#.to_string();
    
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Check code for control flow instructions
    let code = parser.get_code();
    
    // Should have branch and jump instructions
    let has_bz = code.iter().any(|&op| op == Opcode::BZ as i64);
    let has_jmp = code.iter().any(|&op| op == Opcode::JMP as i64);
    
    assert!(has_bz, "Control flow should use BZ instruction");
    assert!(has_jmp, "Control flow should use JMP instruction");
    
    Ok(())
}

#[test]
fn test_parser_expressions() -> Result<(), CompilerError> {
    // Test complex expressions and operator precedence
    let source = r#"
        int test() {
            int a = 2 + 3 * 4;        // 14
            int b = (2 + 3) * 4;      // 20
            int c = 10 - 2 - 3;       // 5
            int d = 8 / 4 / 2;        // 1
            int e = 8 & 5 | 2;        // 2
            int f = 1 << 3 >> 1;      // 4
            return a + b + c + d + e + f;
        }
    "#.to_string();
    
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Check for arithmetic operations
    let code = parser.get_code();
    
    let has_add = code.iter().any(|&op| op == Opcode::ADD as i64);
    let has_sub = code.iter().any(|&op| op == Opcode::SUB as i64);
    let has_mul = code.iter().any(|&op| op == Opcode::MUL as i64);
    let has_div = code.iter().any(|&op| op == Opcode::DIV as i64);
    
    assert!(has_add && has_sub && has_mul && has_div, 
            "Expression parsing should include arithmetic operations");
    
    Ok(())
}

#[test]
fn test_parser_memory() -> Result<(), CompilerError> {
    // Test array and pointer operations
    let source = r#"
        int test() {
            int a = 42;
            int *ptr = &a;
            *ptr = 100;
            
            int arr[3];
            arr[0] = 10;
            arr[1] = 20;
            arr[2] = 30;
            
            return *ptr + arr[1];
        }
    "#.to_string();
    
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // Check for memory operations
    let code = parser.get_code();
    
    // Should have load/store instructions
    let has_li = code.iter().any(|&op| op == Opcode::LI as i64);
    let has_si = code.iter().any(|&op| op == Opcode::SI as i64);
    
    assert!(has_li, "Memory operations should include LI (load int)");
    assert!(has_si, "Memory operations should include SI (store int)");
    
    Ok(())
}

#[test]
fn test_parser_string() -> Result<(), CompilerError> {
    // Test string literal handling
    let source = r#"char *str = "Hello, World!";"#.to_string();
    let mut parser = Parser::new(source, false);
    parser.init()?;
    parser.parse()?;
    
    // String should be in data segment
    let data = parser.get_data();
    
    let string_data: Vec<u8> = data.iter()
        .take_while(|&&b| b != 0) // Stop at null terminator
        .cloned()
        .collect();
    
    let string = String::from_utf8(string_data).unwrap();
    assert_eq!(string, "Hello, World!");
    
    Ok(())
}

#[test]
fn test_parser_error() {
    // Test error handling
    let source = "int broken_function(int x, int y);".to_string();
    let mut parser = Parser::new(source, false);
    parser.init().unwrap();
    
    let result = parser.parse();
    assert!(result.is_err(), "Parser should detect errors in invalid code");
    
    if let Err(CompilerError::ParserError(_)) = result {
        // Expected error type
    } else {
        panic!("Expected ParserError");
    }
}