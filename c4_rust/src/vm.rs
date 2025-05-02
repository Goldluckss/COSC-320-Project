use crate::error::CompilerError;
use crate::types::Opcode;
use std::io::{self, Write};

/// Virtual Machine for executing compiled C4 code
/// 
/// This VM executes the bytecode produced by the C4 compiler.
/// It has a simple register-based architecture with a stack.
pub struct VirtualMachine {
    // VM registers
    pc: usize,     // program counter
    sp: usize,     // stack pointer
    bp: usize,     // base pointer
    ax: i64,       // accumulator
    
    // Memory areas
    code: Vec<i64>,    // code segment
    stack: Vec<i64>,   // stack segment
    data: Vec<u8>,     // data segment
    
    // Debugging
    debug: bool,
    cycle: i64,
}

impl VirtualMachine {
    /// Create a new virtual machine
    ///
    /// # Arguments
    ///
    /// * `code` - The bytecode to execute
    /// * `data` - Initial data segment
    /// * `stack_size` - Size of the stack
    /// * `debug` - Whether to print debug information
    pub fn new(code: Vec<i64>, data: Vec<u8>, stack_size: usize, debug: bool) -> Self {
        let stack = vec![0; stack_size];
        
        VirtualMachine {
            pc: 0,
            sp: stack_size,  // Initialize at end of stack
            bp: stack_size,  // Initialize at end of stack
            ax: 0,
            code,
            stack,
            data,
            debug,
            cycle: 0,
        }
    }
    
    /// Run the VM starting at the specified entry point
    ///
    /// # Arguments
    ///
    /// * `entry_point` - Starting point in the code segment
    /// * `args` - Command line arguments to pass to the program
    ///
    /// # Returns
    ///
    /// The exit code from the program
    pub fn run(&mut self, entry_point: usize, args: &[String]) -> Result<i64, CompilerError> {
        // Setup stack for main()
        self.pc = entry_point;
        
        // Setup argc, argv
        self.sp -= 1;  // Decrement before storing
        self.stack[self.sp] = args.len() as i64;
        
        // Push argv pointer (dummy for now)
        self.sp -= 1;  // Decrement before storing
        self.stack[self.sp] = 0;
        
        // Setup return address (EXIT)
        self.sp -= 1;  // Decrement before storing
        self.stack[self.sp] = Opcode::EXIT as i64;
        
        // Main execution loop
        loop {
            self.cycle += 1;
            
            // Check if PC is valid
            if self.pc >= self.code.len() {
                return Err(CompilerError::VMError(
                    format!("Program counter out of bounds: {}", self.pc)
                ));
            }
            
            // Fetch instruction
            let op = self.code[self.pc];
            
            // Debug output
            if self.debug {
                let op_name = match op {
                    i if i == Opcode::LEA as i64 => "LEA",
                    i if i == Opcode::IMM as i64 => "IMM",
                    i if i == Opcode::JMP as i64 => "JMP",
                    i if i == Opcode::JSR as i64 => "JSR",
                    i if i == Opcode::BZ as i64 => "BZ",
                    i if i == Opcode::BNZ as i64 => "BNZ",
                    i if i == Opcode::ENT as i64 => "ENT",
                    i if i == Opcode::ADJ as i64 => "ADJ",
                    i if i == Opcode::LEV as i64 => "LEV",
                    i if i == Opcode::LI as i64 => "LI",
                    i if i == Opcode::LC as i64 => "LC",
                    i if i == Opcode::SI as i64 => "SI",
                    i if i == Opcode::SC as i64 => "SC",
                    i if i == Opcode::PSH as i64 => "PSH",
                    i if i == Opcode::OR as i64 => "OR",
                    i if i == Opcode::XOR as i64 => "XOR",
                    i if i == Opcode::AND as i64 => "AND",
                    i if i == Opcode::EQ as i64 => "EQ",
                    i if i == Opcode::NE as i64 => "NE",
                    i if i == Opcode::LT as i64 => "LT",
                    i if i == Opcode::GT as i64 => "GT",
                    i if i == Opcode::LE as i64 => "LE",
                    i if i == Opcode::GE as i64 => "GE",
                    i if i == Opcode::SHL as i64 => "SHL",
                    i if i == Opcode::SHR as i64 => "SHR",
                    i if i == Opcode::ADD as i64 => "ADD",
                    i if i == Opcode::SUB as i64 => "SUB",
                    i if i == Opcode::MUL as i64 => "MUL",
                    i if i == Opcode::DIV as i64 => "DIV",
                    i if i == Opcode::MOD as i64 => "MOD",
                    i if i == Opcode::OPEN as i64 => "OPEN",
                    i if i == Opcode::READ as i64 => "READ",
                    i if i == Opcode::CLOS as i64 => "CLOS",
                    i if i == Opcode::PRTF as i64 => "PRTF",
                    i if i == Opcode::MALC as i64 => "MALC",
                    i if i == Opcode::FREE as i64 => "FREE",
                    i if i == Opcode::MSET as i64 => "MSET",
                    i if i == Opcode::MCMP as i64 => "MCMP",
                    i if i == Opcode::EXIT as i64 => "EXIT",
                    _ => "???",
                };
                
                print!("{:>4}> {:8}", self.cycle, op_name);
                
                if op == Opcode::IMM as i64 || op == Opcode::JMP as i64 || op == Opcode::JSR as i64 || 
                   op == Opcode::BZ as i64 || op == Opcode::BNZ as i64 || op == Opcode::ENT as i64 || 
                   op == Opcode::ADJ as i64 {
                    if self.pc + 1 < self.code.len() {
                        println!(" {}", self.code[self.pc + 1]);
                    } else {
                        println!(" ???");
                    }
                } else {
                    println!();
                }
            }
            
            // Execute instruction
            match op {
                i if i == Opcode::EXIT as i64 => {
                    // Exit program with accumulator value
                    return Ok(self.ax);
                },
                _ => {
                    self.execute_instruction(op)?;
                }
            }
            
            // Check stack overflow/underflow
            if self.sp >= self.stack.len() {
                return Err(CompilerError::VMError(
                    format!("Stack pointer out of bounds: {}", self.sp)
                ));
            }
        }
    }
    
    fn execute_instruction(&mut self, op: i64) -> Result<(), CompilerError> {
        match op {
            i if i == Opcode::LEA as i64 => {
                // Load effective address
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                self.ax = self.bp as i64 + self.code[self.pc + 1];
                self.pc += 2;
            },
            i if i == Opcode::IMM as i64 => {
                // Load immediate value into accumulator
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                self.ax = self.code[self.pc + 1];
                self.pc += 2;
            },
            i if i == Opcode::JMP as i64 => {
                // Jump
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                self.pc = self.code[self.pc + 1] as usize;
            },
            i if i == Opcode::JSR as i64 => {
                // Jump to subroutine
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                if self.sp == 0 {
                    return Err(CompilerError::VMError("Stack overflow".to_string()));
                }
                self.sp -= 1;  // Decrement first
                self.stack[self.sp] = self.pc as i64 + 2;  // Then store return address
                self.pc = self.code[self.pc + 1] as usize;  // Jump to function
            },
            i if i == Opcode::BZ as i64 => {
                // Branch if zero
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                if self.ax == 0 {
                    self.pc = self.code[self.pc + 1] as usize;
                } else {
                    self.pc += 2;
                }
            },
            i if i == Opcode::BNZ as i64 => {
                // Branch if not zero
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                if self.ax != 0 {
                    self.pc = self.code[self.pc + 1] as usize;
                } else {
                    self.pc += 2;
                }
            },
            i if i == Opcode::ENT as i64 => {
                // Enter subroutine (setup new stack frame)
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                if self.sp == 0 {
                    return Err(CompilerError::VMError("Stack overflow".to_string()));
                }
                self.stack[self.sp] = self.bp as i64;
                self.sp -= 1;
                self.bp = self.sp;
                
                let locals = self.code[self.pc + 1] as usize;
                if self.sp < locals {
                    return Err(CompilerError::VMError("Stack overflow".to_string()));
                }
                self.sp -= locals;
                self.pc += 2;
            },
            i if i == Opcode::ADJ as i64 => {
                // Adjust stack
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                let count = self.code[self.pc + 1] as usize;
                self.sp += count;
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                self.pc += 2;
            },
            i if i == Opcode::LEV as i64 => {
                // Leave subroutine
                self.sp = self.bp;  // Restore stack pointer
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                self.bp = self.stack[self.sp] as usize;  // Restore base pointer
                if self.sp + 2 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                self.pc = self.stack[self.sp + 1] as usize;  // Restore program counter
            },
            i if i == Opcode::LI as i64 => {
                // Load integer
                if self.ax < 0 || self.ax as usize >= self.stack.len() {
                    return Err(CompilerError::VMError("Invalid memory access".to_string()));
                }
                self.ax = self.stack[self.ax as usize];
                self.pc += 1;
            },
            i if i == Opcode::LC as i64 => {
                // Load character
                if self.ax < 0 || self.ax as usize >= self.data.len() {
                    return Err(CompilerError::VMError("Invalid memory access".to_string()));
                }
                self.ax = self.data[self.ax as usize] as i64;
                self.pc += 1;
            },
            i if i == Opcode::SI as i64 => {
                // Store integer
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let addr = self.stack[self.sp];
                self.sp += 1;
                if addr < 0 || addr as usize >= self.stack.len() {
                    return Err(CompilerError::VMError("Invalid memory access".to_string()));
                }
                self.stack[addr as usize] = self.ax;
                self.pc += 1;
            },
            i if i == Opcode::SC as i64 => {
                // Store character
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let addr = self.stack[self.sp];
                self.sp += 1;
                if addr < 0 || addr as usize >= self.data.len() {
                    return Err(CompilerError::VMError("Invalid memory access".to_string()));
                }
                self.data[addr as usize] = self.ax as u8;
                self.pc += 1;
            },
            i if i == Opcode::PSH as i64 => {
                // Push accumulator onto stack
                if self.sp == 0 {
                    return Err(CompilerError::VMError("Stack overflow".to_string()));
                }
                self.sp -= 1;  // Decrement first
                self.stack[self.sp] = self.ax;  // Then store
                self.pc += 1;
            },
            i if i == Opcode::OR as i64 => {
                // Bitwise OR
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = a | self.ax;
                self.pc += 1;
            },
            i if i == Opcode::XOR as i64 => {
                // Bitwise XOR
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = a ^ self.ax;
                self.pc += 1;
            },
            i if i == Opcode::AND as i64 => {
                // Bitwise AND
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = a & self.ax;
                self.pc += 1;
            },
            i if i == Opcode::EQ as i64 => {
                // Equal
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = (a == self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::NE as i64 => {
                // Not equal
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = (a != self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::LT as i64 => {
                // Less than
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = (a < self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::GT as i64 => {
                // Greater than
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = (a > self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::LE as i64 => {
                // Less than or equal
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = (a <= self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::GE as i64 => {
                // Greater than or equal
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = (a >= self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::SHL as i64 => {
                // Shift left
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = a << self.ax;
                self.pc += 1;
            },
            i if i == Opcode::SHR as i64 => {
                // Shift right
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = a >> self.ax;
                self.pc += 1;
            },
            i if i == Opcode::ADD as i64 => {
                // Add
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = a + self.ax;
                self.pc += 1;
            },
            i if i == Opcode::SUB as i64 => {
                // Subtract
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = a - self.ax;
                self.pc += 1;
            },
            i if i == Opcode::MUL as i64 => {
                // Multiply
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                self.ax = a * self.ax;
                self.pc += 1;
            },
            i if i == Opcode::DIV as i64 => {
                // Divide
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                if self.ax == 0 {
                    return Err(CompilerError::VMError("Division by zero".to_string()));
                }
                self.ax = a / self.ax;
                self.pc += 1;
            },
            i if i == Opcode::MOD as i64 => {
                // Modulo
                if self.sp + 1 >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow".to_string()));
                }
                let a = self.stack[self.sp];
                self.sp += 1;
                if self.ax == 0 {
                    return Err(CompilerError::VMError("Division by zero".to_string()));
                }
                self.ax = a % self.ax;
                self.pc += 1;
            },
            _ => {
                return Err(CompilerError::VMError(format!("Unknown opcode: {}", op)));
            }
        }
        Ok(())
    }
    
    /// Load an integer from memory
    fn load_int(&self, addr: usize) -> Result<i64, CompilerError> {
        // Check if address is in data section
        if addr + std::mem::size_of::<i64>() <= self.data.len() {
            let mut value: i64 = 0;
            
            for i in 0..std::mem::size_of::<i64>() {
                value |= (self.data[addr + i] as i64) << (i * 8);
            }
            
            return Ok(value);
        }
        
        Err(CompilerError::VMError(
            format!("Invalid memory access at address {} for int load", addr)
        ))
    }
    
    /// Load a character from memory
    fn load_char(&self, addr: usize) -> Result<i64, CompilerError> {
        // Check if address is in data section
        if addr < self.data.len() {
            return Ok(self.data[addr] as i64);
        }
        
        Err(CompilerError::VMError(
            format!("Invalid memory access at address {} for char load", addr)
        ))
    }
    
    /// Store an integer to memory
    fn store_int(&mut self, addr: usize, value: i64) -> Result<(), CompilerError> {
        // Check if address is in data section
        if addr + std::mem::size_of::<i64>() <= self.data.len() {
            // Store each byte of the integer
            for i in 0..std::mem::size_of::<i64>() {
                self.data[addr + i] = ((value >> (i * 8)) & 0xFF) as u8;
            }
            Ok(())
        } else {
            Err(CompilerError::VMError(
                format!("Invalid memory access at address {} for int store", addr)
            ))
        }
    }
    
    /// Store a character to memory
    fn store_char(&mut self, addr: usize, value: u8) -> Result<(), CompilerError> {
        // Check if address is in data section
        if addr < self.data.len() {
            self.data[addr] = value;
            Ok(())
        } else {
            Err(CompilerError::VMError(
                format!("Invalid memory access at address {} for char store", addr)
            ))
        }
    }
    
    /// Read a null-terminated string from memory
    fn read_string(&self, addr: usize) -> Result<Vec<u8>, CompilerError> {
        let mut result = Vec::new();
        let mut current_addr = addr;
        
        while current_addr < self.data.len() {
            let byte = self.data[current_addr];
            if byte == 0 {
                break;
            }
            result.push(byte);
            current_addr += 1;
        }
        
        if current_addr >= self.data.len() {
            return Err(CompilerError::VMError(
                format!("String not null-terminated at address {}", addr)
            ));
        }
        
        Ok(result)
    }
}