use c4_rust::error::CompilerError;
use c4_rust::parser::Parser;
use c4_rust::vm::VirtualMachine;

/// Compiles and runs a C program using the C4 Rust compiler
/// 
/// This helper function simplifies testing by handling the compile and run steps.
/// 
/// # Arguments
/// 
/// * `source` - The C source code to compile
/// 
/// # Returns
/// 
/// The program's return value if successful
pub fn compile_and_run(source: &str) -> Result<i64, CompilerError> {
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

/// Compiles a C program using the C4 Rust compiler but doesn't run it
/// 
/// This is useful for testing cases where we just want to check that compilation succeeds.
/// 
/// # Arguments
/// 
/// * `source` - The C source code to compile
/// 
/// # Returns
/// 
/// The Parser instance containing the compiled code
pub fn compile_only(source: &str) -> Result<Parser, CompilerError> {
    // Parse and compile
    let mut parser = Parser::new(source.to_string(), false);
    parser.init()?;
    parser.parse()?;
    
    Ok(parser)
}