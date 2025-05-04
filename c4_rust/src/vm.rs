use crate::error::CompilerError;
use crate::types::Opcode;
use std::io::{self, Read, Write};
use std::process::exit;

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
        
        // Initialize stack pointer at the end of stack (like C4.c)
        let sp = stack_size;
        
        VirtualMachine {
            pc: 0,
            sp,
            bp: sp,
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
        // Setup stack for main() - matching C4.c's setup
        self.pc = entry_point;
        
        // Setup for EXIT when main returns
        self.sp -= 1;
        self.stack[self.sp] = Opcode::EXIT as i64;
        
        // Save stack pointer location for args setup
        let t = self.sp;
        
        // Push argc (number of arguments)
        self.sp -= 1;
        self.stack[self.sp] = args.len() as i64;
        
        // Push argv pointer (simplified - in real C4, this would be more involved)
        self.sp -= 1;
        self.stack[self.sp] = 0; // Not fully implementing argv handling for simplicity
        
        // Push address for EXIT location
        self.sp -= 1;
        self.stack[self.sp] = t as i64;
        
        // Main execution loop
        loop {
            self.cycle += 1;
            
            // Check if PC is out of bounds
            if self.pc >= self.code.len() {
                return Err(CompilerError::VMError {
                    message: format!("Program counter out of bounds: {}", self.pc),
                    instruction: None,
                    cycle: Some(self.cycle),
                });
            }
            
            // Fetch instruction
            let op = self.code[self.pc];
            
            // Debug output
            if self.debug {
                self.print_debug_info(op);
            }
            
            // Execute instruction
            match op as usize {
                i if i == Opcode::LEA as usize => {
                    // Load effective address
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = (self.bp as i64) + self.code[self.pc + 1];
                    self.pc += 2;
                },
                i if i == Opcode::IMM as usize => {
                    // Load immediate value
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.code[self.pc + 1];
                    self.pc += 2;
                },
                i if i == Opcode::JMP as usize => {
                    // Jump
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    let target = self.code[self.pc + 1] as usize;
                    if target >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: format!("Jump target out of bounds: {}", target),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.pc = target;
                },
                i if i == Opcode::JSR as usize => {
                    // Jump to subroutine
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.sp -= 1;
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack overflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.stack[self.sp] = (self.pc + 2) as i64;
                    
                    let target = self.code[self.pc + 1] as usize;
                    if target >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: format!("Jump target out of bounds: {}", target),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.pc = target;
                },
                i if i == Opcode::BZ as usize => {
                    // Branch if zero
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    if self.ax == 0 {
                        let target = self.code[self.pc + 1] as usize;
                        if target >= self.code.len() {
                            return Err(CompilerError::VMError {
                                message: format!("Branch target out of bounds: {}", target),
                                instruction: None,
                                cycle: Some(self.cycle),
                            });
                        }
                        self.pc = target;
                    } else {
                        self.pc += 2;
                    }
                },
                i if i == Opcode::BNZ as usize => {
                    // Branch if not zero
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    if self.ax != 0 {
                        let target = self.code[self.pc + 1] as usize;
                        if target >= self.code.len() {
                            return Err(CompilerError::VMError {
                                message: format!("Branch target out of bounds: {}", target),
                                instruction: None,
                                cycle: Some(self.cycle),
                            });
                        }
                        self.pc = target;
                    } else {
                        self.pc += 2;
                    }
                },
                i if i == Opcode::ENT as usize => {
                    // Enter subroutine
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.sp -= 1;
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack overflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.stack[self.sp] = self.bp as i64;
                    self.bp = self.sp;
                    self.sp -= self.code[self.pc + 1] as usize;
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack overflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.pc += 2;
                },
                i if i == Opcode::ADJ as usize => {
                    // Adjust stack
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.sp += self.code[self.pc + 1] as usize;
                    if self.sp > self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.pc += 2;
                },
                i if i == Opcode::LEV as usize => {
                    // Leave subroutine
                    self.sp = self.bp;
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack pointer out of bounds".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.bp = self.stack[self.sp] as usize;
                    self.sp += 1;
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack pointer out of bounds".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.pc = self.stack[self.sp] as usize;
                    self.sp += 1;
                },
                i if i == Opcode::LI as usize => {
                    // Load int
                    if self.ax as usize >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: format!("Memory access out of bounds: {}", self.ax),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.ax as usize];
                    self.pc += 1;
                },
                i if i == Opcode::LC as usize => {
                    // Load char
                    if self.ax as usize >= self.data.len() {
                        return Err(CompilerError::VMError {
                            message: format!("Memory access out of bounds: {}", self.ax),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.data[self.ax as usize] as i64;
                    self.pc += 1;
                },
                i if i == Opcode::SI as usize => {
                    // Store int
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    let addr = self.stack[self.sp] as usize;
                    self.sp += 1;
                    if addr >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: format!("Memory access out of bounds: {}", addr),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.stack[addr] = self.ax;
                    self.pc += 1;
                },
                i if i == Opcode::SC as usize => {
                    // Store char
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    let addr = self.stack[self.sp] as usize;
                    self.sp += 1;
                    if addr >= self.data.len() {
                        // Grow data segment if necessary
                        if addr < 1_000_000 { // Reasonable limit to prevent OOM
                            self.data.resize(addr + 1, 0);
                        } else {
                            return Err(CompilerError::VMError {
                                message: format!("Memory access out of bounds: {}", addr),
                                instruction: None,
                                cycle: Some(self.cycle),
                            });
                        }
                    }
                    self.data[addr] = self.ax as u8;
                    self.pc += 1;
                },
                i if i == Opcode::PSH as usize => {
                    // Push value onto stack
                    self.sp -= 1;
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack overflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.stack[self.sp] = self.ax;
                    self.pc += 1;
                },
                i if i == Opcode::OR as usize => {
                    // Bitwise OR
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] | self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::XOR as usize => {
                    // Bitwise XOR
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] ^ self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::AND as usize => {
                    // Bitwise AND
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] & self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::EQ as usize => {
                    // Equal
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = (self.stack[self.sp] == self.ax) as i64;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::NE as usize => {
                    // Not equal
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = (self.stack[self.sp] != self.ax) as i64;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::LT as usize => {
                    // Less than
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = (self.stack[self.sp] < self.ax) as i64;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::GT as usize => {
                    // Greater than
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = (self.stack[self.sp] > self.ax) as i64;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::LE as usize => {
                    // Less than or equal
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = (self.stack[self.sp] <= self.ax) as i64;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::GE as usize => {
                    // Greater than or equal
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = (self.stack[self.sp] >= self.ax) as i64;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::SHL as usize => {
                    // Shift left
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] << self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::SHR as usize => {
                    // Shift right
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] >> self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::ADD as usize => {
                    // Add
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] + self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::SUB as usize => {
                    // Subtract
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] - self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::MUL as usize => {
                    // Multiply
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] * self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::DIV as usize => {
                    // Divide
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    if self.ax == 0 {
                        return Err(CompilerError::VMError {
                            message: "Division by zero".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] / self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::MOD as usize => {
                    // Modulo
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    if self.ax == 0 {
                        return Err(CompilerError::VMError {
                            message: "Division by zero in modulo".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    self.ax = self.stack[self.sp] % self.ax;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::OPEN as usize => {
                    // Open file - simplified for cross-platform compatibility
                    if self.sp + 1 >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    let path_ptr = self.stack[self.sp + 1] as usize;
                    let mode = self.stack[self.sp] as i32;
                    
                    // Read null-terminated string from data segment
                    let mut path = Vec::new();
                    let mut ptr = path_ptr;
                    while ptr < self.data.len() && self.data[ptr] != 0 {
                        path.push(self.data[ptr]);
                        ptr += 1;
                    }
                    
                    let path_str = match std::str::from_utf8(&path) {
                        Ok(s) => s,
                        Err(_) => return Err(CompilerError::VMError {
                            message: "Invalid path string".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        }),
                    };
                    
                    // Simple file open implementation
                    match std::fs::OpenOptions::new()
                        .read(mode & 0o1 != 0)
                        .write(mode & 0o2 != 0)
                        .open(path_str) {
                        Ok(_) => self.ax = 3, // Simplified: always return fd 3 (real C4 would track file handles)
                        Err(_) => self.ax = -1,
                    }
                    
                    self.sp += 2;
                    self.pc += 1;
                },
                i if i == Opcode::READ as usize => {
                    // Read from file - simplified
                    if self.sp + 2 >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    let fd = self.stack[self.sp + 2] as i32;
                    let buf_ptr = self.stack[self.sp + 1] as usize;
                    let count = self.stack[self.sp] as usize;
                    
                    // Ensure data segment is large enough
                    if buf_ptr + count > self.data.len() {
                        self.data.resize(buf_ptr + count, 0);
                    }
                    
                    // Simplified read implementation (just read from stdin)
                    if fd == 0 {
                        let mut input = io::stdin();
                        let bytes_read = match input.read(&mut self.data[buf_ptr..buf_ptr + count]) {
                            Ok(n) => n as i64,
                            Err(_) => -1,
                        };
                        self.ax = bytes_read;
                    } else {
                        self.ax = -1; // Simplified: not implementing file reads
                    }
                    
                    self.sp += 3;
                    self.pc += 1;
                },
                i if i == Opcode::CLOS as usize => {
                    // Close file - simplified
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    // Just return success
                    self.ax = 0;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::PRTF as usize => {
                    // Printf - simplified implementation
                    if self.pc + 1 >= self.code.len() {
                        return Err(CompilerError::VMError {
                            message: "Unexpected end of code".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    let arg_count = self.code[self.pc + 1] as usize;
                    if self.sp + arg_count > self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    // The format string is the first argument
                    let fmt_ptr = self.stack[self.sp] as usize;
                    
                    // Read format string from memory
                    let mut fmt = Vec::new();
                    let mut ptr = fmt_ptr;
                    while ptr < self.data.len() && self.data[ptr] != 0 {
                        fmt.push(self.data[ptr]);
                        ptr += 1;
                    }
                    
                    let fmt_str = match std::str::from_utf8(&fmt) {
                        Ok(s) => s,
                        Err(_) => return Err(CompilerError::VMError {
                            message: "Invalid format string".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        }),
                    };
                    
                    // Very simplified printf implementation - just print the format string
                    print!("{}", fmt_str);
                    io::stdout().flush().unwrap();
                    
                    self.ax = fmt_str.len() as i64;
                    self.sp += arg_count;
                    self.pc += 2;
                },
                i if i == Opcode::MALC as usize => {
                    // Malloc - simplified implementation
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    let size = self.stack[self.sp] as usize;
                    
                    // Simplified: allocate from the end of the data segment
                    let addr = self.data.len();
                    self.data.resize(addr + size, 0);
                    
                    self.ax = addr as i64;
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::FREE as usize => {
                    // Free - no-op in this simplified implementation
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    self.sp += 1;
                    self.pc += 1;
                },
                i if i == Opcode::MSET as usize => {
                    // Memset
                    if self.sp + 2 >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    let dst_ptr = self.stack[self.sp + 2] as usize;
                    let value = self.stack[self.sp + 1] as u8;
                    let count = self.stack[self.sp] as usize;
                    
                    // Ensure data segment is large enough
                    if dst_ptr + count > self.data.len() {
                        self.data.resize(dst_ptr + count, 0);
                    }
                    
                    // Set memory
                    for i in 0..count {
                        self.data[dst_ptr + i] = value;
                    }
                    
                    self.ax = dst_ptr as i64;
                    self.sp += 3;
                    self.pc += 1;
                },
                i if i == Opcode::MCMP as usize => {
                    // Memcmp
                    if self.sp + 2 >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    let s1_ptr = self.stack[self.sp + 2] as usize;
                    let s2_ptr = self.stack[self.sp + 1] as usize;
                    let count = self.stack[self.sp] as usize;
                    
                    // Check bounds
                    if s1_ptr + count > self.data.len() || s2_ptr + count > self.data.len() {
                        return Err(CompilerError::VMError {
                            message: "Memory access out of bounds".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    // Compare memory
                    for i in 0..count {
                        let a = self.data[s1_ptr + i];
                        let b = self.data[s2_ptr + i];
                        if a != b {
                            self.ax = (a as i64) - (b as i64);
                            self.sp += 3;
                            self.pc += 1;
                            return Ok(0); // Return 0 to continue execution
                        }
                    }
                    
                    self.ax = 0; // Equal
                    self.sp += 3;
                    self.pc += 1;
                },
                i if i == Opcode::EXIT as usize => {
                    // Exit
                    if self.sp >= self.stack.len() {
                        return Err(CompilerError::VMError {
                            message: "Stack underflow".to_string(),
                            instruction: None,
                            cycle: Some(self.cycle),
                        });
                    }
                    
                    if self.debug {
                        println!("exit({}) cycle = {}", self.stack[self.sp], self.cycle);
                    }
                    
                    return Ok(self.stack[self.sp]);
                },
                _ => {
                    return Err(CompilerError::VMError {
                        message: format!("Unknown opcode: {}", op),
                        instruction: None,
                        cycle: Some(self.cycle),
                    });
                }
            }
        }
    }
    
    /// Print debugging information for the current instruction
    fn print_debug_info(&self, op: i64) {
        let opcode_str = match op as usize {
            i if i == Opcode::LEA as usize => "LEA",
            i if i == Opcode::IMM as usize => "IMM",
            i if i == Opcode::JMP as usize => "JMP",
            i if i == Opcode::JSR as usize => "JSR",
            i if i == Opcode::BZ as usize => "BZ",
            i if i == Opcode::BNZ as usize => "BNZ",
            i if i == Opcode::ENT as usize => "ENT",
            i if i == Opcode::ADJ as usize => "ADJ",
            i if i == Opcode::LEV as usize => "LEV",
            i if i == Opcode::LI as usize => "LI",
            i if i == Opcode::LC as usize => "LC",
            i if i == Opcode::SI as usize => "SI",
            i if i == Opcode::SC as usize => "SC",
            i if i == Opcode::PSH as usize => "PSH",
            i if i == Opcode::OR as usize => "OR",
            i if i == Opcode::XOR as usize => "XOR",
            i if i == Opcode::AND as usize => "AND",
            i if i == Opcode::EQ as usize => "EQ",
            i if i == Opcode::NE as usize => "NE",
            i if i == Opcode::LT as usize => "LT",
            i if i == Opcode::GT as usize => "GT",
            i if i == Opcode::LE as usize => "LE",
            i if i == Opcode::GE as usize => "GE",
            i if i == Opcode::SHL as usize => "SHL",
            i if i == Opcode::SHR as usize => "SHR",
            i if i == Opcode::ADD as usize => "ADD",
            i if i == Opcode::SUB as usize => "SUB",
            i if i == Opcode::MUL as usize => "MUL",
            i if i == Opcode::DIV as usize => "DIV",
            i if i == Opcode::MOD as usize => "MOD",
            i if i == Opcode::OPEN as usize => "OPEN",
            i if i == Opcode::READ as usize => "READ",
            i if i == Opcode::CLOS as usize => "CLOS",
            i if i == Opcode::PRTF as usize => "PRTF",
            i if i == Opcode::MALC as usize => "MALC",
            i if i == Opcode::FREE as usize => "FREE",
            i if i == Opcode::MSET as usize => "MSET",
            i if i == Opcode::MCMP as usize => "MCMP",
            i if i == Opcode::EXIT as usize => "EXIT",
            _ => "???",
        };
        
        print!("{:4}> {:8}", self.cycle, opcode_str);
        
        // Print operand for instructions that have one
        if op as usize == Opcode::IMM as usize || 
           op as usize == Opcode::LEA as usize || 
           op as usize == Opcode::JMP as usize || 
           op as usize == Opcode::JSR as usize || 
           op as usize == Opcode::BZ as usize || 
           op as usize == Opcode::BNZ as usize || 
           op as usize == Opcode::ENT as usize || 
           op as usize == Opcode::ADJ as usize {
            if self.pc + 1 < self.code.len() {
                println!(" {}", self.code[self.pc + 1]);
            } else {
                println!(" ???");
            }
        } else {
            println!();
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vm_basic() {
        // Basic program: return 42
        let code = vec![
            Opcode::IMM as i64, 42,
            Opcode::EXIT as i64,
        ];
        
        let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
        let result = vm.run(0, &[]).unwrap();
        
        assert_eq!(result, 42);
    }
    
    #[test]
    fn test_vm_arithmetic() {
        // Test arithmetic operations
        let code = vec![
            // Load 10
            Opcode::IMM as i64, 10,
            // Push 10
            Opcode::PSH as i64,
            // Load 5
            Opcode::IMM as i64, 5,
            // Add: 10 + 5 = 15
            Opcode::ADD as i64,
            // Push 15
            Opcode::PSH as i64,
            // Load 3
            Opcode::IMM as i64, 3,
            // Multiply: 15 * 3 = 45
            Opcode::MUL as i64,
            // Push 45
            Opcode::PSH as i64,
            // Load 5
            Opcode::IMM as i64, 5,
            // Divide: 45 / 5 = 9
            Opcode::DIV as i64,
            // Exit with 9
            Opcode::EXIT as i64,
        ];
        
        let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
        let result = vm.run(0, &[]).unwrap();
        
        assert_eq!(result, 9);
    }
    
    #[test]
    fn test_vm_conditional_branch() {
        // Test conditional branching
        let code = vec![
            // Load 10
            Opcode::IMM as i64, 10,
            // Push 10
            Opcode::PSH as i64,
            // Load 5
            Opcode::IMM as i64, 5,
            // Greater than: 10 > 5 = 1
            Opcode::GT as i64,
            // Branch if zero (not taken)
            Opcode::BZ as i64, 12,
            // Load 42 (this branch is taken)
            Opcode::IMM as i64, 42,
            // Exit with 42
            Opcode::EXIT as i64,
            // Load 24 (not reached)
            Opcode::IMM as i64, 24,
            // Exit with 24 (not reached)
            Opcode::EXIT as i64,
        ];
        
        let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
        let result = vm.run(0, &[]).unwrap();
        
        assert_eq!(result, 42);
    }
    
    #[test]
    fn test_vm_function_call() {
        // Test function calls
        let code = vec![
            // Jump to main
            Opcode::JMP as i64, 10,
            
            // Function: double(x) -> x * 2
            // Set up stack frame
            Opcode::ENT as i64, 0,
            // Load parameter (bp+2)
            Opcode::LEA as i64, 2,
            // Get value
            Opcode::LI as i64,
            // Push value
            Opcode::PSH as i64,
            // Load 2
            Opcode::IMM as i64, 2,
            // Multiply
            Opcode::MUL as i64,
            // Return
            Opcode::LEV as i64,
            
            // Main function
            // Load 21
            Opcode::IMM as i64, 21,
            // Push argument
            Opcode::PSH as i64,
            // Call double()
            Opcode::JSR as i64, 2,
            // Remove argument
            Opcode::ADJ as i64, 1,
            // Exit with result (42)
            Opcode::EXIT as i64,
        ];
        
        let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
        let result = vm.run(10, &[]).unwrap();
        
        assert_eq!(result, 42);
    }
}