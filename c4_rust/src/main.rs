use std::env;
use std::fs;
use std::process;

use c4_rust::error::CompilerError;
use c4_rust::lexer::Lexer;
use c4_rust::vm::VirtualMachine;

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
        eprintln!("usage: c4_rust [-s] [-d] file ...");
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
    
    println!("Successfully read source file: {}", filename.unwrap());
    println!("Source length: {} characters", source.len());
    
    // TODO: Implement parser and code generation
    // For now, just tokenize the source to demonstrate lexer functionality
    if src_flag {
        println!("Tokenizing source file...");
        let mut lexer = Lexer::new(source, true);
        
        loop {
            match lexer.next_token() {
                Ok(token) => {
                    println!("{:?}", token);
                    if token.token_type == c4_rust::types::TokenType::Eof {
                        break;
                    }
                },
                Err(err) => {
                    eprintln!("Lexer error: {}", err);
                    process::exit(1);
                }
            }
        }
        
        println!("Tokenization complete");
    }
    
    println!("Note: Full compiler implementation is in progress.");
    
    Ok(())
}