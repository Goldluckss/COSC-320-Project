use std::env;
use std::fs;
use std::process;
use c4_rust::parser::Parser;
use c4_rust::vm::VirtualMachine;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check for command-line flags and input file
    let mut i = 1;
    let mut src_flag = false;
    let mut debug_flag = false;
    let mut input_file = None;
    
    while i < args.len() {
        if args[i] == "-s" {
            src_flag = true;
        } else if args[i] == "-d" {
            debug_flag = true;
        } else {
            input_file = Some(args[i].clone());
            break;
        }
        i += 1;
    }
    
    // Check if we have an input file
    if input_file.is_none() {
        eprintln!("usage: c4_rust [-s] [-d] file ...");
        process::exit(1);
    }
    
    // Read source file
    let input_file = input_file.unwrap();
    let source = match fs::read_to_string(&input_file) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("could not open({}): {}", input_file, err);
            process::exit(1);
        }
    };
    
    // Print banner if -s flag is set
    if src_flag {
        println!("C4 Rust Compiler - Compiling {}", input_file);
    }
    
    // Create parser
    let mut parser = match Parser::new(source, src_flag) {
        mut parser => {
            match parser.init() {
                Ok(()) => parser,
                Err(err) => {
                    eprintln!("Parser initialization error: {}", err);
                    process::exit(1);
                }
            }
        }
    };
    
    // Parse source code
    if let Err(err) = parser.parse() {
        eprintln!("Compilation error: {}", err);
        process::exit(1);
    }
    
    // Get main function
    let main_addr = match parser.get_main_function() {
        Some(addr) => addr,
        None => {
            eprintln!("main() not defined");
            process::exit(1);
        }
    };
    
    // If -s flag is set, just print the source and exit
    if src_flag {
        println!("Compilation successful!");
        
        // Print code segment summary
        println!("\nCode segment size: {} bytes", parser.get_code().len() * 8);
        println!("Data segment size: {} bytes", parser.get_data().len());
        println!("main() function found at offset: {}", (*main_addr).value as usize);
        
        // Exit with success
        process::exit(0);
    }
    
    // Create VM
    let code = parser.get_code().to_vec();
    let data = parser.get_data().to_vec();
    let mut vm = VirtualMachine::new(code, data, 256 * 1024, debug_flag);
    
    // Extract command-line arguments for the program
    let prog_args: Vec<String> = args.iter().skip(i + 1).cloned().collect();
    
    // Run the program
    match vm.run(main_addr.value as usize, &prog_args) {
        Ok(exit_code) => {
            if debug_flag {
                println!("Program exited with code: {}", exit_code);
            }
            process::exit(exit_code as i32);
        },
        Err(err) => {
            eprintln!("Runtime error: {}", err);
            process::exit(1);
        }
    }
}