use c4_rust::error::CompilerError;
use c4_rust::types::Opcode;
use c4_rust::vm::VirtualMachine;

/// Test basic VM operations
#[test]
fn test_basic_vm() -> Result<(), CompilerError> {
    // Simple program: return 42
    let code = vec![
        Opcode::IMM as i64, 42,      // Load 42
        Opcode::EXIT as i64,         // Exit with value in AX
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 42);
    Ok(())
}

/// Test arithmetic operations
#[test]
fn test_arithmetic() -> Result<(), CompilerError> {
    // Program to test arithmetic operations
    let code = vec![
        // Addition: 10 + 20 = 30
        Opcode::IMM as i64, 10,      // Load 10
        Opcode::PSH as i64,          // Push 10
        Opcode::IMM as i64, 20,      // Load 20
        Opcode::ADD as i64,          // Add: 10 + 20 = 30
        
        // Subtraction: 30 - 5 = 25
        Opcode::PSH as i64,          // Push 30
        Opcode::IMM as i64, 5,       // Load 5
        Opcode::SUB as i64,          // Subtract: 30 - 5 = 25
        
        // Multiplication: 25 * 2 = 50
        Opcode::PSH as i64,          // Push 25
        Opcode::IMM as i64, 2,       // Load 2
        Opcode::MUL as i64,          // Multiply: 25 * 2 = 50
        
        // Division: 50 / 10 = 5
        Opcode::PSH as i64,          // Push 50
        Opcode::IMM as i64, 10,      // Load 10
        Opcode::DIV as i64,          // Divide: 50 / 10 = 5
        
        // Modulo: 5 % 3 = 2
        Opcode::PSH as i64,          // Push 5
        Opcode::IMM as i64, 3,       // Load 3
        Opcode::MOD as i64,          // Modulo: 5 % 3 = 2
        
        Opcode::EXIT as i64,         // Exit with value in AX
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 2);
    Ok(())
}

/// Test logical and bitwise operations
#[test]
fn test_logical_bitwise() -> Result<(), CompilerError> {
    // Program to test logical and bitwise operations
    let code = vec![
        // Bitwise OR: 5 | 3 = 7
        Opcode::IMM as i64, 5,       // Load 5 (101 binary)
        Opcode::PSH as i64,          // Push 5
        Opcode::IMM as i64, 3,       // Load 3 (011 binary)
        Opcode::OR as i64,           // OR: 5 | 3 = 7 (111 binary)
        
        // Bitwise AND: 7 & 5 = 5
        Opcode::PSH as i64,          // Push 7
        Opcode::IMM as i64, 5,       // Load 5
        Opcode::AND as i64,          // AND: 7 & 5 = 5
        
        // Bitwise XOR: 5 ^ 3 = 6
        Opcode::PSH as i64,          // Push 5
        Opcode::IMM as i64, 3,       // Load 3
        Opcode::XOR as i64,          // XOR: 5 ^ 3 = 6
        
        // Equal: 6 == 6 (true = 1)
        Opcode::PSH as i64,          // Push 6
        Opcode::IMM as i64, 6,       // Load 6
        Opcode::EQ as i64,           // EQ: 6 == 6 = 1
        
        // Not equal: 1 != 0 (true = 1)
        Opcode::PSH as i64,          // Push 1
        Opcode::IMM as i64, 0,       // Load 0
        Opcode::NE as i64,           // NE: 1 != 0 = 1
        
        Opcode::EXIT as i64,         // Exit with value in AX
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 1);
    Ok(())
}

/// Test comparison operations
#[test]
fn test_comparison() -> Result<(), CompilerError> {
    // Program to test comparison operations
    let code = vec![
        // Less than: 5 < 10 (true = 1)
        Opcode::IMM as i64, 5,       // Load 5
        Opcode::PSH as i64,          // Push 5
        Opcode::IMM as i64, 10,      // Load 10
        Opcode::LT as i64,           // LT: 5 < 10 = 1
        
        // Greater than: 1 > 0 (true = 1)
        Opcode::PSH as i64,          // Push 1
        Opcode::IMM as i64, 0,       // Load 0
        Opcode::GT as i64,           // GT: 1 > 0 = 1
        
        // Less than or equal: 1 <= 1 (true = 1)
        Opcode::PSH as i64,          // Push 1
        Opcode::IMM as i64, 1,       // Load 1
        Opcode::LE as i64,           // LE: 1 <= 1 = 1
        
        // Greater than or equal: 1 >= 2 (false = 0)
        Opcode::PSH as i64,          // Push 1
        Opcode::IMM as i64, 2,       // Load 2
        Opcode::GE as i64,           // GE: 1 >= 2 = 0
        
        Opcode::EXIT as i64,         // Exit with value in AX
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 0);
    Ok(())
}

/// Test shift operations
#[test]
fn test_shift() -> Result<(), CompilerError> {
    // Program to test shift operations
    let code = vec![
        // Shift left: 1 << 3 = 8
        Opcode::IMM as i64, 1,       // Load 1
        Opcode::PSH as i64,          // Push 1
        Opcode::IMM as i64, 3,       // Load 3
        Opcode::SHL as i64,          // SHL: 1 << 3 = 8
        
        // Shift right: 8 >> 1 = 4
        Opcode::PSH as i64,          // Push 8
        Opcode::IMM as i64, 1,       // Load 1
        Opcode::SHR as i64,          // SHR: 8 >> 1 = 4
        
        Opcode::EXIT as i64,         // Exit with value in AX
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 4);
    Ok(())
}

/// Test conditional branching (BZ, BNZ instructions)
#[test]
fn test_conditional_branching() -> Result<(), CompilerError> {
    // Program to test conditional branching
    let code = vec![
        // Check if 1 == 0
        Opcode::IMM as i64, 1,       // Load 1
        Opcode::PSH as i64,          // Push 1
        Opcode::IMM as i64, 0,       // Load 0
        Opcode::EQ as i64,           // 1 == 0? (false = 0)
        
        // If result is 0 (condition false), branch to else
        Opcode::BZ as i64, 11,       // Branch to else path if result is 0
        
        // Then path (should not be taken)
        Opcode::IMM as i64, 42,      // Load 42
        Opcode::JMP as i64, 13,      // Jump to end
        
        // Else path
        Opcode::IMM as i64, 24,      // Load 24
        
        // End
        Opcode::EXIT as i64,         // Exit with value in AX
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 24);
    Ok(())
}

/// Test unconditional jumping (JMP instruction)
#[test]
fn test_jumping() -> Result<(), CompilerError> {
    // Program to test jumping
    let code = vec![
        Opcode::JMP as i64, 5,       // Jump past the next instruction
        Opcode::IMM as i64, 10,      // Load 10 (should be skipped)
        Opcode::EXIT as i64,         // Exit with 10 (should be skipped)
        
        // This is where we jump to
        Opcode::IMM as i64, 42,      // Load 42
        Opcode::EXIT as i64,         // Exit with 42
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 42);
    Ok(())
}

/// Test function calls (JSR, ENT, LEV instructions)
#[test]
fn test_function_calls() -> Result<(), CompilerError> {
    // Program to test function calls
    let code = vec![
        // Jump to main
        Opcode::JMP as i64, 9,       // Jump to main
        
        // Function 'add': add(a, b) returns a + b
        Opcode::ENT as i64, 0,       // Set up stack frame
        Opcode::LEA as i64, 2,       // Load address of parameter 'a'
        Opcode::LI as i64,           // Load value of 'a'
        Opcode::PSH as i64,          // Push 'a'
        Opcode::LEA as i64, 3,       // Load address of parameter 'b'
        Opcode::LI as i64,           // Load value of 'b'
        Opcode::ADD as i64,          // Add: a + b
        Opcode::LEV as i64,          // Return from function
        
        // Main function
        Opcode::ENT as i64, 0,       // Set up stack frame
        Opcode::IMM as i64, 10,      // Load 10 (parameter 'a')
        Opcode::PSH as i64,          // Push 'a'
        Opcode::IMM as i64, 20,      // Load 20 (parameter 'b')
        Opcode::PSH as i64,          // Push 'b'
        Opcode::JSR as i64, 1,       // Call 'add' function
        Opcode::ADJ as i64, 2,       // Adjust stack (remove parameters)
        Opcode::LEV as i64,          // Return from main
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(9, &[])?;    // Start execution at main
    
    assert_eq!(result, 30);          // 10 + 20 = 30
    Ok(())
}

/// Test memory operations (LI, LC, SI, SC instructions)
#[test]
fn test_memory_operations() -> Result<(), CompilerError> {
    // Create some initial data
    let mut data = vec![0; 32];
    
    // Program to test memory operations
    let code = vec![
        // Store a value to memory
        Opcode::IMM as i64, 0,       // Address 0
        Opcode::PSH as i64,          // Push address 0
        Opcode::IMM as i64, 42,      // Value 42
        Opcode::SI as i64,           // Store integer: mem[0] = 42
        
        // Store a character to memory
        Opcode::IMM as i64, 8,       // Address 8
        Opcode::PSH as i64,          // Push address 8
        Opcode::IMM as i64, 65,      // Character 'A' (ASCII 65)
        Opcode::SC as i64,           // Store character: mem[8] = 'A'
        
        // Load values back
        Opcode::IMM as i64, 0,       // Address 0
        Opcode::LI as i64,           // Load integer from mem[0]
        Opcode::PSH as i64,          // Push loaded value
        Opcode::IMM as i64, 8,       // Address 8
        Opcode::LC as i64,           // Load character from mem[8]
        Opcode::ADD as i64,          // Add: 42 + 65 = 107
        
        Opcode::EXIT as i64,         // Exit with value in AX
    ];
    
    let mut vm = VirtualMachine::new(code, data, 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 107);         // 42 + 65 = 107
    Ok(())
}

/// Test stack operations (PSH, ADJ instructions)
#[test]
fn test_stack_operations() -> Result<(), CompilerError> {
    // Program to test stack operations
    let code = vec![
        // Push values onto stack
        Opcode::IMM as i64, 10,      // Load 10
        Opcode::PSH as i64,          // Push 10
        Opcode::IMM as i64, 20,      // Load 20
        Opcode::PSH as i64,          // Push 20
        Opcode::IMM as i64, 30,      // Load 30
        Opcode::PSH as i64,          // Push 30
        
        // Sum all values on stack: 10 + 20 + 30 = 60
        Opcode::IMM as i64, 0,       // Initialize sum to 0
        
        // Add first value (30)
        Opcode::PSH as i64,          // Push 0
        Opcode::IMM as i64, 30,      // Load 30 (simulating pop from stack)
        Opcode::ADD as i64,          // 0 + 30 = 30
        
        // Add second value (20)
        Opcode::PSH as i64,          // Push 30
        Opcode::IMM as i64, 20,      // Load 20 (simulating pop from stack)
        Opcode::ADD as i64,          // 30 + 20 = 50
        
        // Add third value (10)
        Opcode::PSH as i64,          // Push 50
        Opcode::IMM as i64, 10,      // Load 10 (simulating pop from stack)
        Opcode::ADD as i64,          // 50 + 10 = 60
        
        // Adjust stack (remove pushed values)
        Opcode::ADJ as i64, 3,       // Adjust stack by 3
        
        Opcode::EXIT as i64,         // Exit with value in AX
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 60);          // 10 + 20 + 30 = 60
    Ok(())
}

/// Test error handling in the VM
#[test]
fn test_vm_errors() {
    // Test 1: Division by zero
    let code = vec![
        Opcode::IMM as i64, 10,      // Load 10
        Opcode::PSH as i64,          // Push 10
        Opcode::IMM as i64, 0,       // Load 0
        Opcode::DIV as i64,          // Attempt division by zero
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[]);
    
    assert!(result.is_err(), "VM should detect division by zero");
    if let Err(CompilerError::VMError { message, .. }) = result {
        assert!(message.contains("zero"), "Error message should mention division by zero");
    }
    
    // Test 2: Jump out of bounds
    let code = vec![
        Opcode::JMP as i64, 100,     // Jump to out-of-bounds address
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[]);
    
    assert!(result.is_err(), "VM should detect out-of-bounds jump");
    if let Err(CompilerError::VMError { message, .. }) = result {
        assert!(message.contains("bounds"), "Error message should mention out-of-bounds");
    }
    
    // Test 3: Stack overflow
    let mut code = vec![
        Opcode::IMM as i64, 1,       // Load 1
    ];
    
    // Add many PSH instructions to cause stack overflow
    for _ in 0..1100 {
        code.push(Opcode::PSH as i64);
    }
    
    code.push(Opcode::EXIT as i64);  // Exit
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[]);
    
    assert!(result.is_err(), "VM should detect stack overflow");
    if let Err(CompilerError::VMError { message, .. }) = result {
        assert!(message.contains("stack") || message.contains("Stack"),
                "Error message should mention stack issue");
    }
}

/// Test execution of a simple C program compiled to bytecode
/// This test simulates a complete end-to-end test of the VM
#[test]
fn test_simple_c_program() -> Result<(), CompilerError> {
    // The bytecode for a simple C program:
    // int main() {
    //     int x = 10;
    //     int y = 20;
    //     return x + y;
    // }
    
    let code = vec![
        // Main function
        Opcode::ENT as i64, 2,       // Set up stack frame with 2 local vars
        
        // x = 10
        Opcode::LEA as i64, -1,      // Load address of local var 'x'
        Opcode::PSH as i64,          // Push address
        Opcode::IMM as i64, 10,      // Load 10
        Opcode::SI as i64,           // Store: x = 10
        
        // y = 20
        Opcode::LEA as i64, -2,      // Load address of local var 'y'
        Opcode::PSH as i64,          // Push address
        Opcode::IMM as i64, 20,      // Load 20
        Opcode::SI as i64,           // Store: y = 20
        
        // return x + y
        Opcode::LEA as i64, -1,      // Load address of 'x'
        Opcode::LI as i64,           // Load value of 'x'
        Opcode::PSH as i64,          // Push 'x'
        Opcode::LEA as i64, -2,      // Load address of 'y'
        Opcode::LI as i64,           // Load value of 'y'
        Opcode::ADD as i64,          // Add: x + y
        
        Opcode::LEV as i64,          // Return from function
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 30);          // 10 + 20 = 30
    Ok(())
}

/// Test VM with a recursive function
#[test]
fn test_recursive_function() -> Result<(), CompilerError> {
    // The bytecode for a factorial function:
    // int factorial(int n) {
    //     if (n <= 1) return 1;
    //     return n * factorial(n-1);
    // }
    
    let code = vec![
        // Jump to main
        Opcode::JMP as i64, 18,      // Jump to main
        
        // Factorial function
        Opcode::ENT as i64, 0,       // Set up stack frame
        
        // if (n <= 1)
        Opcode::LEA as i64, 2,       // Load address of parameter 'n'
        Opcode::LI as i64,           // Load value of 'n'
        Opcode::PSH as i64,          // Push 'n'
        Opcode::IMM as i64, 1,       // Load 1
        Opcode::LE as i64,           // n <= 1?
        Opcode::BZ as i64, 10,       // If not, jump to else
        
        // return 1
        Opcode::IMM as i64, 1,       // Load 1
        Opcode::LEV as i64,          // Return 1
        
        // else: return n * factorial(n-1)
        // Calculate n-1
        Opcode::LEA as i64, 2,       // Load address of 'n'
        Opcode::LI as i64,           // Load value of 'n'
        Opcode::PSH as i64,          // Push 'n'
        Opcode::IMM as i64, 1,       // Load 1
        Opcode::SUB as i64,          // n - 1
        
        // Call factorial(n-1)
        Opcode::PSH as i64,          // Push n-1
        Opcode::JSR as i64, 1,       // Call factorial
        Opcode::ADJ as i64, 1,       // Remove argument
        
        // Multiply n * factorial(n-1)
        Opcode::PSH as i64,          // Push factorial result
        Opcode::LEA as i64, 2,       // Load address of 'n'
        Opcode::LI as i64,           // Load value of 'n'
        Opcode::MUL as i64,          // n * factorial(n-1)
        
        Opcode::LEV as i64,          // Return result
        
        // Main function
        Opcode::IMM as i64, 5,       // Load 5 (calculate factorial(5))
        Opcode::PSH as i64,          // Push 5
        Opcode::JSR as i64, 1,       // Call factorial
        Opcode::ADJ as i64, 1,       // Remove argument
        Opcode::EXIT as i64,         // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(18, &[])?;   // Start at main
    
    assert_eq!(result, 120);         // factorial(5) = 120
    Ok(())
}