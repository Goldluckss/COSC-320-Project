use c4_rust::error::CompilerError;
use c4_rust::types::Opcode;
use c4_rust::vm::VirtualMachine;
use pretty_assertions::assert_eq;

#[test]
fn test_vm_basic_operations() -> Result<(), CompilerError> {
    // Create code that tests basic operations
    let code = vec![
        Opcode::IMM as i64, 42,   // Load 42 into AX
        Opcode::EXIT as i64,      // Exit with result in AX
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 42);
    Ok(())
}

#[test]
fn test_vm_arithmetic() -> Result<(), CompilerError> {
    // Create code that tests basic arithmetic operations
    let code = vec![
        // 5 + 3 = 8
        Opcode::IMM as i64, 5,    // Load 5
        Opcode::PSH as i64,       // Push 5
        Opcode::IMM as i64, 3,    // Load 3
        Opcode::ADD as i64,       // Add: 5 + 3 = 8
        
        // 8 * 4 = 32
        Opcode::PSH as i64,       // Push 8
        Opcode::IMM as i64, 4,    // Load 4
        Opcode::MUL as i64,       // Multiply: 8 * 4 = 32
        
        // 32 / 2 = 16
        Opcode::PSH as i64,       // Push 32
        Opcode::IMM as i64, 2,    // Load 2
        Opcode::DIV as i64,       // Divide: 32 / 2 = 16
        
        // 16 - 6 = 10
        Opcode::PSH as i64,       // Push 16
        Opcode::IMM as i64, 6,    // Load 6
        Opcode::SUB as i64,       // Subtract: 16 - 6 = 10
        
        // 10 % 3 = 1
        Opcode::PSH as i64,       // Push 10
        Opcode::IMM as i64, 3,    // Load 3
        Opcode::MOD as i64,       // Modulo: 10 % 3 = 1
        
        Opcode::EXIT as i64,      // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 1);  // Final result should be 1
    Ok(())
}

#[test]
fn test_vm_logical_operations() -> Result<(), CompilerError> {
    // Create code that tests logical/bitwise operations
    let code = vec![
        // Bitwise OR: 5 | 2 = 7
        Opcode::IMM as i64, 5,    // Load 5 (binary 101)
        Opcode::PSH as i64,       // Push 5
        Opcode::IMM as i64, 2,    // Load 2 (binary 010)
        Opcode::OR as i64,        // OR: 101 | 010 = 111 (7)
        
        // Bitwise XOR: 7 ^ 3 = 4
        Opcode::PSH as i64,       // Push 7 (binary 111)
        Opcode::IMM as i64, 3,    // Load 3 (binary 011)
        Opcode::XOR as i64,       // XOR: 111 ^ 011 = 100 (4)
        
        // Bitwise AND: 4 & 6 = 4
        Opcode::PSH as i64,       // Push 4 (binary 100)
        Opcode::IMM as i64, 6,    // Load 6 (binary 110)
        Opcode::AND as i64,       // AND: 100 & 110 = 100 (4)
        
        Opcode::EXIT as i64,      // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 4);  // Final result should be 4
    Ok(())
}

#[test]
fn test_vm_comparison_operations() -> Result<(), CompilerError> {
    // Create code that tests comparison operations
    let code = vec![
        // Equal: 5 == 5 (true = 1)
        Opcode::IMM as i64, 5,    // Load 5
        Opcode::PSH as i64,       // Push 5
        Opcode::IMM as i64, 5,    // Load 5
        Opcode::EQ as i64,        // Equal: 1
        
        // Not equal: 1 != 2 (true = 1)
        Opcode::PSH as i64,       // Push 1
        Opcode::IMM as i64, 2,    // Load 2
        Opcode::NE as i64,        // Not equal: 1
        
        // Less than: 5 < 10 (true = 1)
        Opcode::PSH as i64,       // Push 1
        Opcode::IMM as i64, 5,    // Load 5
        Opcode::PSH as i64,       // Push 5
        Opcode::IMM as i64, 10,   // Load 10
        Opcode::LT as i64,        // Less than: 1
        
        // Greater than: 8 > 4 (true = 1)
        Opcode::PSH as i64,       // Push 1
        Opcode::IMM as i64, 8,    // Load 8
        Opcode::PSH as i64,       // Push 8
        Opcode::IMM as i64, 4,    // Load 4
        Opcode::GT as i64,        // Greater than: 1
        
        // Less than or equal: 5 <= 5 (true = 1)
        Opcode::PSH as i64,       // Push 1
        Opcode::IMM as i64, 5,    // Load 5
        Opcode::PSH as i64,       // Push 5
        Opcode::IMM as i64, 5,    // Load 5
        Opcode::LE as i64,        // Less than or equal: 1
        
        // Greater than or equal: 6 >= 7 (false = 0)
        Opcode::PSH as i64,       // Push 1
        Opcode::IMM as i64, 6,    // Load 6
        Opcode::PSH as i64,       // Push 6
        Opcode::IMM as i64, 7,    // Load 7
        Opcode::GE as i64,        // Greater than or equal: 0
        
        Opcode::EXIT as i64,      // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 0);  // Final result should be 0 (false)
    Ok(())
}

#[test]
fn test_vm_shift_operations() -> Result<(), CompilerError> {
    // Create code that tests shift operations
    let code = vec![
        // Shift left: 1 << 3 = 8
        Opcode::IMM as i64, 1,    // Load 1
        Opcode::PSH as i64,       // Push 1
        Opcode::IMM as i64, 3,    // Load 3
        Opcode::SHL as i64,       // Shift left: 1 << 3 = 8
        
        // Shift right: 16 >> 2 = 4
        Opcode::PSH as i64,       // Push 8
        Opcode::IMM as i64, 16,   // Load 16
        Opcode::PSH as i64,       // Push 16
        Opcode::IMM as i64, 2,    // Load 2
        Opcode::SHR as i64,       // Shift right: 16 >> 2 = 4
        
        Opcode::EXIT as i64,      // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 4);  // Final result should be 4
    Ok(())
}

#[test]
fn test_vm_jump_instructions() -> Result<(), CompilerError> {
    // Create code that tests jump instructions
    let code = vec![
        // Unconditional jump
        Opcode::JMP as i64, 7,        // Jump to position 7
        
        // This should be skipped
        Opcode::IMM as i64, 0,        // Load 0
        Opcode::EXIT as i64,          // Exit (should not be executed)
        
        // This is position 7
        Opcode::IMM as i64, 42,       // Load 42
        Opcode::EXIT as i64,          // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 42);  // Should execute the code after the jump
    Ok(())
}

#[test]
fn test_vm_conditional_branching() -> Result<(), CompilerError> {
    // Create code that tests conditional branching (if-like behavior)
    let code = vec![
        // Test condition (5 > 3)
        Opcode::IMM as i64, 5,        // Load 5
        Opcode::PSH as i64,           // Push 5
        Opcode::IMM as i64, 3,        // Load 3
        Opcode::GT as i64,            // 5 > 3 = true (1)
        
        // Branch if zero (not taken because result is 1)
        Opcode::BZ as i64, 12,        // Branch to "else" if false
        
        // "Then" branch
        Opcode::IMM as i64, 42,       // Load 42
        Opcode::JMP as i64, 14,       // Jump to end
        
        // "Else" branch (position 12)
        Opcode::IMM as i64, 24,       // Load 24
        
        // End (position 14)
        Opcode::EXIT as i64,          // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 42);  // Should take the "then" branch
    
    // Now test with a condition that's false
    let code = vec![
        // Test condition (2 > 5)
        Opcode::IMM as i64, 2,        // Load 2
        Opcode::PSH as i64,           // Push 2
        Opcode::IMM as i64, 5,        // Load 5
        Opcode::GT as i64,            // 2 > 5 = false (0)
        
        // Branch if zero (taken because result is 0)
        Opcode::BZ as i64, 12,        // Branch to "else" if false
        
        // "Then" branch
        Opcode::IMM as i64, 42,       // Load 42
        Opcode::JMP as i64, 14,       // Jump to end
        
        // "Else" branch (position 12)
        Opcode::IMM as i64, 24,       // Load 24
        
        // End (position 14)
        Opcode::EXIT as i64,          // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 24);  // Should take the "else" branch
    Ok(())
}

#[test]
fn test_vm_loop() -> Result<(), CompilerError> {
    // Create code that implements a simple loop (sum 1 to 5)
    let code = vec![
        // Initialize i = 1, sum = 0
        Opcode::IMM as i64, 1,        // Load 1 (i = 1)
        Opcode::PSH as i64,           // Store i on stack
        Opcode::IMM as i64, 0,        // Load 0 (sum = 0)
        Opcode::PSH as i64,           // Store sum on stack
        
        // Loop condition (i <= 5) - position 4
        Opcode::IMM as i64, 5,        // Load 5
        Opcode::PSH as i64,           // Push 5
        Opcode::LEA as i64, -2,       // Load address of i (sp-2)
        Opcode::LI as i64,            // Load i value
        Opcode::LE as i64,            // 5 <= i ?
        Opcode::BZ as i64, 25,        // If false, break loop
        
        // Loop body
        // sum += i
        Opcode::LEA as i64, -1,       // Load address of sum (sp-1)
        Opcode::LI as i64,            // Load sum value
        Opcode::PSH as i64,           // Push sum
        Opcode::LEA as i64, -3,       // Load address of i (sp-3 now)
        Opcode::LI as i64,            // Load i value
        Opcode::ADD as i64,           // sum + i
        Opcode::LEA as i64, -1,       // Load address of sum (sp-1)
        Opcode::SI as i64,            // Store result in sum
        
        // i++
        Opcode::LEA as i64, -2,       // Load address of i (sp-2)
        Opcode::LI as i64,            // Load i value
        Opcode::PSH as i64,           // Push i
        Opcode::IMM as i64, 1,        // Load 1
        Opcode::ADD as i64,           // i + 1
        Opcode::LEA as i64, -2,       // Load address of i (sp-2)
        Opcode::SI as i64,            // Store result in i
        
        // Jump back to loop condition
        Opcode::JMP as i64, 4,        // Jump to condition check
        
        // End of loop (position 25)
        Opcode::LEA as i64, -1,       // Load address of sum
        Opcode::LI as i64,            // Load sum value
        Opcode::EXIT as i64,          // Exit with sum as result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 15);  // Sum of 1 to 5 = 15
    Ok(())
}

#[test]
fn test_vm_function_call() -> Result<(), CompilerError> {
    // Create code that tests function calls
    // We'll implement a main function that calls a function to compute double(x)
    let code = vec![
        // Jump to main
        Opcode::JMP as i64, 8,        // Jump to main
        
        // double function (position 2)
        Opcode::ENT as i64, 0,        // Setup stack frame
        Opcode::LEA as i64, 2,        // Load address of parameter (bp+2)
        Opcode::LI as i64,            // Load value of parameter
        Opcode::PSH as i64,           // Push value
        Opcode::IMM as i64, 2,        // Load 2
        Opcode::MUL as i64,           // Multiply by 2
        Opcode::LEV as i64,           // Return to caller
        
        // Main function (position 8)
        Opcode::IMM as i64, 21,       // Load argument value (21)
        Opcode::PSH as i64,           // Push argument
        Opcode::JSR as i64, 2,        // Call double function
        Opcode::ADJ as i64, 1,        // Adjust stack (remove argument)
        Opcode::EXIT as i64,          // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(8, &[])?; // Start at main function
    
    assert_eq!(result, 42);  // double(21) = 42
    Ok(())
}

#[test]
fn test_vm_nested_function_calls() -> Result<(), CompilerError> {
    // Create code for nested function calls (calculate factorial recursively)
    let code = vec![
        // Jump to main
        Opcode::JMP as i64, 22,       // Jump to main
        
        // Factorial function (position 2)
        Opcode::ENT as i64, 0,        // Setup stack frame
        Opcode::LEA as i64, 2,        // Load address of parameter n (bp+2)
        Opcode::LI as i64,            // Load value of n
        
        // If n <= 1, return 1
        Opcode::PSH as i64,           // Push n
        Opcode::IMM as i64, 1,        // Load 1
        Opcode::LE as i64,            // n <= 1 ?
        Opcode::BZ as i64, 13,        // If false, continue recursion
        
        // Base case: return 1
        Opcode::IMM as i64, 1,        // Load 1
        Opcode::LEV as i64,           // Return 1
        
        // Recursive case: return n * factorial(n-1)
        // Compute n-1
        Opcode::LEA as i64, 2,        // Load address of n (bp+2)
        Opcode::LI as i64,            // Load value of n
        Opcode::PSH as i64,           // Push n
        Opcode::IMM as i64, 1,        // Load 1
        Opcode::SUB as i64,           // n - 1
        
        // Call factorial(n-1)
        Opcode::PSH as i64,           // Push (n-1)
        Opcode::JSR as i64, 2,        // Call factorial
        Opcode::ADJ as i64, 1,        // Adjust stack
        
        // Multiply n * factorial(n-1)
        Opcode::PSH as i64,           // Push factorial(n-1)
        Opcode::LEA as i64, 2,        // Load address of n
        Opcode::LI as i64,            // Load value of n
        Opcode::MUL as i64,           // n * factorial(n-1)
        
        Opcode::LEV as i64,           // Return result
        
        // Main function (position 22)
        Opcode::IMM as i64, 5,        // Calculate factorial(5)
        Opcode::PSH as i64,           // Push argument
        Opcode::JSR as i64, 2,        // Call factorial
        Opcode::ADJ as i64, 1,        // Adjust stack
        Opcode::EXIT as i64,          // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(22, &[])?; // Start at main function
    
    assert_eq!(result, 120);  // factorial(5) = 120
    Ok(())
}

#[test]
fn test_vm_memory_operations() -> Result<(), CompilerError> {
    // Create a data segment with some values
    let data = vec![
        10, 20, 30, 40, 50, 60, 70, 80,  // 8 bytes of data
    ];
    
    // Create code that tests memory operations
    let code = vec![
        // Test loading char from memory
        Opcode::IMM as i64, 0,        // Address 0
        Opcode::LC as i64,            // Load char (should be 10)
        
        // Test storing char to memory
        Opcode::IMM as i64, 100,      // Value 100
        Opcode::PSH as i64,           // Push address 0
        Opcode::IMM as i64, 0,        // Address 0
        Opcode::SC as i64,            // Store char 100 at address 0
        
        // Verify store worked
        Opcode::IMM as i64, 0,        // Address 0
        Opcode::LC as i64,            // Load char (should be 100)
        
        Opcode::EXIT as i64,          // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, data, 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 100);  // Should be the value we stored
    Ok(())
}

#[test]
fn test_vm_stack_operations() -> Result<(), CompilerError> {
    // Create code that tests stack operations
    let code = vec![
        // Push values to stack
        Opcode::IMM as i64, 10,       // Load 10
        Opcode::PSH as i64,           // Push 10
        Opcode::IMM as i64, 20,       // Load 20
        Opcode::PSH as i64,           // Push 20
        Opcode::IMM as i64, 30,       // Load 30
        Opcode::PSH as i64,           // Push 30
        
        // Pop and add
        Opcode::IMM as i64, 0,        // Load 0
        Opcode::PSH as i64,           // Push 0
        Opcode::IMM as i64, 30,       // Load 30 (top of stack)
        Opcode::ADD as i64,           // Add: 0 + 30 = 30
        Opcode::PSH as i64,           // Push 30
        Opcode::IMM as i64, 20,       // Load 20 (next on stack)
        Opcode::ADD as i64,           // Add: 30 + 20 = 50
        Opcode::PSH as i64,           // Push 50
        Opcode::IMM as i64, 10,       // Load 10 (bottom value)
        Opcode::ADD as i64,           // Add: 50 + 10 = 60
        
        Opcode::EXIT as i64,          // Exit with result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 60);  // 10 + 20 + 30 = 60
    Ok(())
}

#[test]
fn test_vm_error_handling() {
    // Test division by zero error
    let code = vec![
        Opcode::IMM as i64, 10,       // Load 10
        Opcode::PSH as i64,           // Push 10
        Opcode::IMM as i64, 0,        // Load 0
        Opcode::DIV as i64,           // Attempt to divide by zero
        Opcode::EXIT as i64,          // This shouldn't be reached
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[]);
    
    // Should return an error
    assert!(result.is_err());
    
    // The error should be a VMError
    if let Err(CompilerError::VMError(message)) = result {
        assert!(message.contains("Division by zero"));
    } else {
        panic!("Expected VMError for division by zero");
    }
    
    // Test jump out of bounds
    let code = vec![
        Opcode::JMP as i64, 999,      // Jump to non-existent code
        Opcode::EXIT as i64,          // This shouldn't be reached
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[]);
    
    // Should return an error
    assert!(result.is_err());
    
    // The error should be a VMError
    if let Err(CompilerError::VMError(message)) = result {
        assert!(message.contains("out of bounds"));
    } else {
        panic!("Expected VMError for jump out of bounds");
    }
}

#[test]
fn test_vm_complex_program() -> Result<(), CompilerError> {
    // Create a more complex program: calculate sum of first 10 even numbers
    let code = vec![
        // Initialize variables: sum = 0, count = 0, num = 0
        Opcode::IMM as i64, 0,        // sum = 0
        Opcode::PSH as i64,           // Store sum
        Opcode::IMM as i64, 0,        // count = 0
        Opcode::PSH as i64,           // Store count
        Opcode::IMM as i64, 0,        // num = 0
        Opcode::PSH as i64,           // Store num
        
        // Loop condition: count < 10 - position 6
        Opcode::LEA as i64, -2,       // Load address of count
        Opcode::LI as i64,            // Load count
        Opcode::PSH as i64,           // Push count
        Opcode::IMM as i64, 10,       // Load 10
        Opcode::LT as i64,            // count < 10?
        Opcode::BZ as i64, 30,        // If false, exit loop
        
        // Increment num by 2
        Opcode::LEA as i64, -1,       // Load address of num
        Opcode::LI as i64,            // Load num
        Opcode::PSH as i64,           // Push num
        Opcode::IMM as i64, 2,        // Load 2
        Opcode::ADD as i64,           // num + 2
        Opcode::LEA as i64, -1,       // Load address of num
        Opcode::SI as i64,            // Store result in num
        
        // Add num to sum
        Opcode::LEA as i64, -3,       // Load address of sum
        Opcode::LI as i64,            // Load sum
        Opcode::PSH as i64,           // Push sum
        Opcode::LEA as i64, -1,       // Load address of num
        Opcode::LI as i64,            // Load num
        Opcode::ADD as i64,           // sum + num
        Opcode::LEA as i64, -3,       // Load address of sum
        Opcode::SI as i64,            // Store result in sum
        
        // Increment count
        Opcode::LEA as i64, -2,       // Load address of count
        Opcode::LI as i64,            // Load count
        Opcode::PSH as i64,           // Push count
        Opcode::IMM as i64, 1,        // Load 1
        Opcode::ADD as i64,           // count + 1
        Opcode::LEA as i64, -2,       // Load address of count
        Opcode::SI as i64,            // Store result in count
        
        // Jump back to condition
        Opcode::JMP as i64, 6,        // Jump to condition
        
        // Loop exit - position 30
        Opcode::LEA as i64, -3,       // Load address of sum
        Opcode::LI as i64,            // Load sum
        Opcode::EXIT as i64,          // Exit with sum as result
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    // Sum of first 10 even numbers: 2+4+6+8+10+12+14+16+18+20 = 110
    assert_eq!(result, 110);
    Ok(())
}