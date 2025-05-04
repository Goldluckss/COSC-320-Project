use c4_rust::error::CompilerError;
use c4_rust::parser::Parser;
use c4_rust::vm::VirtualMachine;
use pretty_assertions::assert_eq;

/// Helper function to compile and run C code
fn compile_and_run(source: &str) -> Result<i64, CompilerError> {
    // Parse and compile
    let mut parser = Parser::new(source.to_string(), false);
    parser.init()?;
    parser.parse()?;
    
    // Get the bytecode
    let code = parser.get_code();
    let data = parser.get_data();
    
    // Get the main function
    let main_offset = parser.get_main_function()
        .ok_or_else(|| CompilerError::ParserError("main function not found".to_string()))?;
    
    // Run the program
    let mut vm = VirtualMachine::new(code.to_vec(), data.to_vec(), 1024, false);
    vm.run(main_offset, &[])
}

#[test]
fn test_basic_program() -> Result<(), CompilerError> {
    let source = r#"
        int main() { 
            return 42; 
        }
    "#;
    
    let result = compile_and_run(source)?;
    assert_eq!(result, 42);
    
    Ok(())
}

#[test]
fn test_arithmetic() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            int a;
            int b;
            int add;
            int sub;
            int mul;
            int div;
            int mod;
            
            a = 10;
            b = 5;
            
            add = a + b;      // 15
            sub = a - b;      // 5
            mul = a * b;      // 50
            div = a / b;      // 2
            mod = a % b;      // 0
            
            return add + sub + mul + div + mod;  // 72
        }
    "#;
    
    let result = compile_and_run(source)?;
    assert_eq!(result, 72);
    
    Ok(())
}

#[test]
fn test_control_flow() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            int x;
            int result;
            int i;
            
            x = 10;
            result = 0;
            
            // Test if-else
            if (x > 5) {
                result = result + 10;  // This branch should execute
            } else {
                result = result + 1;
            }
            
            // Test while loop - sum from 1 to 5
            i = 1;
            while (i <= 5) {
                result = result + i;
                i = i + 1;
            }
            
            return result;  // 10 + (1+2+3+4+5) = 25
        }
    "#;
    
    let result = compile_and_run(source)?;
    assert_eq!(result, 25);
    
    Ok(())
}

#[test]
fn test_functions() -> Result<(), CompilerError> {
    let source = r#"
        int add(int a, int b) {
            return a + b;
        }
        
        int main() {
            return add(10, 20);  // 30
        }
    "#;
    
    let result = compile_and_run(source)?;
    assert_eq!(result, 30);
    
    Ok(())
}

#[test]
fn test_recursion() -> Result<(), CompilerError> {
    let source = r#"
        int factorial(int n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n-1);
        }
        
        int main() {
            return factorial(5);  // 5! = 120
        }
    "#;
    
    let result = compile_and_run(source)?;
    assert_eq!(result, 120);
    
    Ok(())
}

#[test]
fn test_pointers_and_arrays() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            // Test pointer operations
            int x;
            int *ptr;
            
            x = 10;
            ptr = &x;
            *ptr = 20;
            
            // Test array operations
            int arr[3];
            arr[0] = 1;
            arr[1] = 2;
            arr[2] = 3;
            
            return x + arr[0] + arr[1] + arr[2];  // 20+1+2+3 = 26
        }
    "#;
    
    let result = compile_and_run(source)?;
    assert_eq!(result, 26);
    
    Ok(())
}

#[test]
fn test_bit_operations() -> Result<(), CompilerError> {
    let source = r#"
        int main() {
            int a;
            int b;
            int c;
            int d;
            int e;
            int f;
            int g;
            int h;
            
            // Bitwise operations
            a = 5 & 3;           // 101 & 011 = 001 = 1
            b = 5 | 3;           // 101 | 011 = 111 = 7
            c = 5 ^ 3;           // 101 ^ 011 = 110 = 6
            d = 1 << 3;          // 1 << 3 = 8
            e = 8 >> 2;          // 8 >> 2 = 2
            
            // Logical operations  
            f = 1 && 1;          // 1
            g = 1 || 0;          // 1
            h = !0;              // 1
            
            return a + b + c + d + e + f + g + h;  // 1+7+6+8+2+1+1+1 = 27
        }
    "#;
    
    let result = compile_and_run(source)?;
    assert_eq!(result, 27);
    
    Ok(())
}

#[test]
fn test_complex_program() -> Result<(), CompilerError> {
    let source = r#"
        // Calculate sum of squares from 1 to n
        int sum_squares(int n) {
            int sum;
            int i;
            
            sum = 0;
            i = 1;
            
            while (i <= n) {
                sum = sum + i * i;
                i = i + 1;
            }
            
            return sum;
        }
        
        // Calculate square of sum from 1 to n
        int square_sum(int n) {
            int sum;
            int i;
            
            sum = 0;
            i = 1;
            
            while (i <= n) {
                sum = sum + i;
                i = i + 1;
            }
            
            return sum * sum;
        }
        
        int main() {
            // Find difference: (1+2+3+4)² - (1²+2²+3²+4²)
            // = 10² - (1+4+9+16) = 100 - 30 = 70
            return square_sum(4) - sum_squares(4);
        }
    "#;
    
    let result = compile_and_run(source)?;
    assert_eq!(result, 70);
    
    Ok(())
}