mod error;
mod lexer;
mod parser;
mod symbol;
mod types;
mod vm;

use std::env;
use std::fs;
use std::process;

use crate::error::CompilerError;
use crate::parser::Parser;
use crate::vm::VirtualMachine;

fn main() -> Result<(), CompilerError> {
    let args: Vec<String> = env::args().collect();
    
    let mut src_flag = false;
    let mut debug_flag = false;
    let mut filename = None;
    
    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-s" {
            src_flag = true;
        } else if args[i] == "-d" {
            debug_flag = true;
        } else {
            filename = Some(&args[i]);
            break;
        }
        i += 1;
    }
    
    // Check if filename is provided
    if filename.is_none() {
        eprintln!("usage: c4 [-s] [-d] file ...");
        process::exit(1);
    }
    
    // Read the source file
    let source = match fs::read_to_string(filename.unwrap()) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("could not open({}): {}", filename.unwrap(), err);
            process::exit(1);
        }
    };
    
    // Create parser and compile
    let mut parser = Parser::new(source, src_flag, debug_flag);
    if let Err(e) = parser.init() {
        eprintln!("Initialization error: {}", e);
        process::exit(1);
    }
    
    // Parse the program
    if let Err(e) = parser.parse() {
        eprintln!("Compilation error: {}", e);
        process::exit(1);
    }
    
    // If src_flag is set, don't run the program
    if src_flag {
        println!("Compilation successful");
        return Ok(());
    }
    
    // Create and run VM
    match parser.get_entry_point() {
        Ok(entry_point) => {
            let mut vm = VirtualMachine::new(
                parser.get_code().to_vec(),
                parser.get_data().to_vec(),
                256 * 1024, // Stack size (same as C4)
                debug_flag
            );
            
            // Extract command line args for main
            let program_args: Vec<String> = if i + 1 < args.len() {
                args[i+1..].to_vec()
            } else {
                Vec::new()
            };
            
            // Run the program
            match vm.run(entry_point as usize, &program_args) {
                Ok(exit_code) => {
                    if debug_flag {
                        println!("Program exited with code {}", exit_code);
                    }
                    process::exit(exit_code as i32);
                },
                Err(e) => {
                    eprintln!("Runtime error: {}", e);
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("Entry point error: {}", e);
            process::exit(1);
        }
    }
}