use c4_rust::error::CompilerError;
use c4_rust::types::Opcode;
use c4_rust::vm::VirtualMachine;

#[test]
fn test_basic_execution() -> Result<(), CompilerError> {
    // Program: return 42
    let code = vec![
        Opcode::IMM as i64, 42,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
    let result = vm.run(0, &[])?;
    
    assert_eq!(result, 42);
    Ok(())
}

#[test]
fn test_arithmetic_operations() -> Result<(), CompilerError> {
    // Test addition
    let add_code = vec![
        Opcode::IMM as i64, 5,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 3,
        Opcode::ADD as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(add_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 8);
    
    // Test subtraction
    let sub_code = vec![
        Opcode::IMM as i64, 10,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 4,
        Opcode::SUB as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(sub_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 6);
    
    // Test multiplication
    let mul_code = vec![
        Opcode::IMM as i64, 6,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 7,
        Opcode::MUL as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(mul_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 42);
    
    // Test division
    let div_code = vec![
        Opcode::IMM as i64, 20,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 4,
        Opcode::DIV as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(div_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 5);
    
    Ok(())
}

#[test]
fn test_comparison_operations() -> Result<(), CompilerError> {
    // Test equal
    let eq_code = vec![
        Opcode::IMM as i64, 10,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 10,
        Opcode::EQ as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(eq_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 1); // true
    
    // Test not equal
    let ne_code = vec![
        Opcode::IMM as i64, 10,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 5,
        Opcode::NE as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(ne_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 1); // true
    
    // Test less than
    let lt_code = vec![
        Opcode::IMM as i64, 5,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 10,
        Opcode::LT as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(lt_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 1); // true
    
    // Test greater than
    let gt_code = vec![
        Opcode::IMM as i64, 15,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 10,
        Opcode::GT as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(gt_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 1); // true
    
    Ok(())
}

#[test]
fn test_memory_operations() -> Result<(), CompilerError> {
    // Test storing and loading integer
    let mut data = vec![0; 100]; // Allocate data space
    
    let mem_code = vec![
        // Store 42 at address 0
        Opcode::IMM as i64, 0,      // Address
        Opcode::PSH as i64,
        Opcode::IMM as i64, 42,     // Value
        Opcode::SI as i64,
        
        // Store 84 at address 8
        Opcode::IMM as i64, 8,      // Address
        Opcode::PSH as i64,
        Opcode::IMM as i64, 84,     // Value
        Opcode::SI as i64,
        
        // Load from address 0
        Opcode::IMM as i64, 0,
        Opcode::LI as i64,
        Opcode::PSH as i64,
        
        // Load from address 8
        Opcode::IMM as i64, 8,
        Opcode::LI as i64,
        Opcode::ADD as i64,      // Add the two values
        
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(mem_code, data, 1024, false);
    assert_eq!(vm.run(0, &[])?, 126); // 42 + 84 = 126
    
    // Test char operations
    let mut data = vec![0; 100]; // Allocate data space
    
    let char_code = vec![
        // Store 'A' at address 0
        Opcode::IMM as i64, 0,      // Address
        Opcode::PSH as i64,
        Opcode::IMM as i64, 65,     // 'A'
        Opcode::SC as i64,
        
        // Store 'B' at address 1
        Opcode::IMM as i64, 1,      // Address
        Opcode::PSH as i64,
        Opcode::IMM as i64, 66,     // 'B'
        Opcode::SC as i64,
        
        // Load from address 0
        Opcode::IMM as i64, 0,
        Opcode::LC as i64,
        
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(char_code, data, 1024, false);
    assert_eq!(vm.run(0, &[])?, 65); // 'A'
    
    Ok(())
}

#[test]
fn test_branching() -> Result<(), CompilerError> {
    // Test if-then-else with BZ (branch if zero)
    let if_code = vec![
        Opcode::IMM as i64, 1,      // Condition (true)
        Opcode::BZ as i64, 6,       // Branch to else if zero
        Opcode::IMM as i64, 42,     // Then clause
        Opcode::JMP as i64, 8,      // Jump past else
        Opcode::IMM as i64, 24,     // Else clause
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(if_code, Vec::new(), 1024, false);
    // This should execute the 'then' clause, not branch, and return 42
    assert_eq!(vm.run(0, &[])?, 42);
    
    // Test with condition false
    let if_code2 = vec![
        Opcode::IMM as i64, 0,      // Condition (false)
        Opcode::BZ as i64, 6,       // Branch to else if zero
        Opcode::IMM as i64, 42,     // Then clause
        Opcode::JMP as i64, 8,      // Jump past else
        Opcode::IMM as i64, 24,     // Else clause
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(if_code2, Vec::new(), 1024, false);
    // This should branch to the 'else' clause and return 24
    assert_eq!(vm.run(0, &[])?, 24);
    
    Ok(())
}

#[test]
fn test_function_call() -> Result<(), CompilerError> {
    // Test function call with JSR and LEV
    // Main function calls add(5, 3) and returns the result
    let function_code = vec![
        // Main function
        Opcode::IMM as i64, 5,      // First argument
        Opcode::PSH as i64,
        Opcode::IMM as i64, 3,      // Second argument
        Opcode::PSH as i64,
        Opcode::JSR as i64, 9,      // Call add function at index 9
        Opcode::ADJ as i64, 2,      // Clean up arguments (2 args)
        Opcode::EXIT as i64,        // Return from main
        
        // Add function (index 9)
        Opcode::ENT as i64, 0,      // Setup stack frame (no locals)
        Opcode::LEA as i64, 2,      // Load address of first argument (a)
        Opcode::LI as i64,          // Load value of a
        Opcode::LEA as i64, 1,      // Load address of second argument (b)
        Opcode::LI as i64,          // Load value of b
        Opcode::ADD as i64,         // Add a + b
        Opcode::LEV as i64,         // Return from function
    ];
    
    let mut vm = VirtualMachine::new(function_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 8); // 5 + 3 = 8
    
    Ok(())
}

#[test]
fn test_error_handling() {
    // Test division by zero
    let div_zero_code = vec![
        Opcode::IMM as i64, 42,
        Opcode::PSH as i64,
        Opcode::IMM as i64, 0,      // Divisor is zero
        Opcode::DIV as i64,
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(div_zero_code, Vec::new(), 1024, false);
    let result = vm.run(0, &[]);
    assert!(result.is_err());
    
    if let Err(CompilerError::VMError(msg)) = result {
        assert!(msg.contains("Division by zero"));
    } else {
        panic!("Expected VMError for division by zero");
    }
    
    // Test invalid jump address
    let bad_jump_code = vec![
        Opcode::JMP as i64, 999,    // Jump to non-existent address
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(bad_jump_code, Vec::new(), 1024, false);
    let result = vm.run(0, &[]);
    assert!(result.is_err());
    
    // Test invalid memory access
    let bad_mem_code = vec![
        Opcode::IMM as i64, 999,    // Address outside data segment
        Opcode::LI as i64,          // Attempt to load
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(bad_mem_code, vec![0; 10], 1024, false);
    let result = vm.run(0, &[]);
    assert!(result.is_err());
}

#[test]
fn test_simple_loop() -> Result<(), CompilerError> {
    // Compute sum from 1 to 10
    let loop_code = vec![
        // Initialize sum = 0, i = 1
        Opcode::IMM as i64, 0,      // sum = 0
        Opcode::PSH as i64,         // Save sum
        Opcode::IMM as i64, 1,      // i = 1
        Opcode::PSH as i64,         // Save i
        
        // Loop start (index 4)
        // Check if i > 10
        Opcode::PSH as i64,         // Duplicate i
        Opcode::IMM as i64, 10,
        Opcode::GT as i64,          // i > 10?
        Opcode::BNZ as i64, 20,     // If true, exit loop
        
        // Loop body: sum += i, i++
        Opcode::PSH as i64,         // Get i
        Opcode::PSH as i64,         // Get sum
        Opcode::ADD as i64,         // sum += i
        Opcode::PSH as i64,         // Save new sum
        
        Opcode::PSH as i64,         // Get i
        Opcode::IMM as i64, 1,
        Opcode::ADD as i64,         // i++
        Opcode::PSH as i64,         // Save new i
        
        // Continue loop
        Opcode::JMP as i64, 4,      // Jump to loop start
        
        // Exit loop with sum value (index 20)
        Opcode::EXIT as i64,
    ];
    
    let mut vm = VirtualMachine::new(loop_code, Vec::new(), 1024, false);
    assert_eq!(vm.run(0, &[])?, 55); // Sum from 1 to 10 = 55
    
    Ok(())
}