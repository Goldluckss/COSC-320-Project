use crate::error::CompilerError;
use crate::types::Opcode;

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
        let mut stack = vec![0; stack_size];
        
        // Initialize stack pointer at the end of stack (like C4.c)
        let sp = stack_size;
        
        VirtualMachine {
            pc: 0,
            sp,  // Initialize at end of stack
            bp: sp,  // Initialize at end of stack
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
        
        // Setup argc, argv - matching C4.c's stack setup
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
        }
    }
    
    fn execute_instruction(&mut self, op: i64) -> Result<(), CompilerError> {
        match op {
            i if i == Opcode::LEA as i64 => {
                // Load effective address - matching C4.c's implementation
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
                // Jump - matching C4.c's implementation
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                self.pc = self.code[self.pc + 1] as usize;
            },
            i if i == Opcode::JSR as i64 => {
                // Jump to subroutine - matching C4.c's implementation
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                self.sp -= 1;
                self.stack[self.sp] = (self.pc + 2) as i64;
                self.pc = self.code[self.pc + 1] as usize;
            },
            i if i == Opcode::BZ as i64 => {
                // Branch if zero - matching C4.c's implementation
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
                // Branch if not zero - matching C4.c's implementation
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
                // Enter subroutine - matching C4.c's implementation
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                self.sp -= 1;
                self.stack[self.sp] = self.bp as i64;
                self.bp = self.sp;
                self.sp -= self.code[self.pc + 1] as usize;
                self.pc += 2;
            },
            i if i == Opcode::ADJ as i64 => {
                // Adjust stack - matching C4.c's implementation
                if self.pc + 1 >= self.code.len() {
                    return Err(CompilerError::VMError("Unexpected end of code".to_string()));
                }
                self.sp += self.code[self.pc + 1] as usize;
                self.pc += 2;
            },
            i if i == Opcode::LEV as i64 => {
                // Leave subroutine - matching C4.c's implementation
                self.sp = self.bp;
                self.bp = self.stack[self.sp] as usize;
                self.sp += 1;
                self.pc = self.stack[self.sp] as usize;
                self.sp += 1;
            },
            i if i == Opcode::LI as i64 => {
                // Load integer - matching C4.c's implementation
                self.ax = self.stack[self.ax as usize];
                self.pc += 1;
            },
            i if i == Opcode::LC as i64 => {
                // Load character - matching C4.c's implementation
                self.ax = self.data[self.ax as usize] as i64;
                self.pc += 1;
            },
            i if i == Opcode::SI as i64 => {
                // Store integer - matching C4.c's implementation
                let addr = self.stack[self.sp] as usize;
                self.sp += 1;
                self.stack[addr] = self.ax;
                self.pc += 1;
            },
            i if i == Opcode::SC as i64 => {
                // Store character - matching C4.c's implementation
                self.data[self.stack[self.sp] as usize] = self.ax as u8;
                self.sp += 1;
                self.pc += 1;
            },
            i if i == Opcode::PSH as i64 => {
                // Push - matching C4.c's implementation
                self.sp -= 1;
                self.stack[self.sp] = self.ax;
                self.pc += 1;
            },
            i if i == Opcode::OR as i64 => {
                // Bitwise OR: stack top | AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in OR".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = value | self.ax;
                self.pc += 1;
            },
            i if i == Opcode::XOR as i64 => {
                // Bitwise XOR: stack top ^ AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in XOR".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = value ^ self.ax;
                self.pc += 1;
            },
            i if i == Opcode::AND as i64 => {
                // Bitwise AND: stack top & AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in AND".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = value & self.ax;
                self.pc += 1;
            },
            i if i == Opcode::EQ as i64 => {
                // Equal: stack top == AX -> AX (1 if true, 0 if false)
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in EQ".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = (value == self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::NE as i64 => {
                // Not Equal: stack top != AX -> AX (1 if true, 0 if false)
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in NE".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = (value != self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::LT as i64 => {
                // Less Than: stack top < AX -> AX (1 if true, 0 if false)
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in LT".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = (value < self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::GT as i64 => {
                // Greater Than: stack top > AX -> AX (1 if true, 0 if false)
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in GT".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = (value > self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::LE as i64 => {
                // Less Than or Equal: stack top <= AX -> AX (1 if true, 0 if false)
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in LE".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = (value <= self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::GE as i64 => {
                // Greater Than or Equal: stack top >= AX -> AX (1 if true, 0 if false)
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in GE".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = (value >= self.ax) as i64;
                self.pc += 1;
            },
            i if i == Opcode::SHL as i64 => {
                // Shift Left: stack top << AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in SHL".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = value << self.ax;
                self.pc += 1;
            },
            i if i == Opcode::SHR as i64 => {
                // Shift Right: stack top >> AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in SHR".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = value >> self.ax;
                self.pc += 1;
            },
            i if i == Opcode::ADD as i64 => {
                // Add: stack top + AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in ADD".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = value + self.ax;
                self.pc += 1;
            },
            i if i == Opcode::SUB as i64 => {
                // Subtract: stack top - AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in SUB".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = value - self.ax;
                self.pc += 1;
            },
            i if i == Opcode::MUL as i64 => {
                // Multiply: stack top * AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in MUL".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                self.ax = value * self.ax;
                self.pc += 1;
            },
            i if i == Opcode::DIV as i64 => {
                // Divide: stack top / AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in DIV".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                if self.ax == 0 {
                    return Err(CompilerError::VMError("Division by zero".to_string()));
                }
                
                self.ax = value / self.ax;
                self.pc += 1;
            },
            i if i == Opcode::MOD as i64 => {
                // Modulo: stack top % AX -> AX
                if self.sp >= self.stack.len() {
                    return Err(CompilerError::VMError("Stack underflow in MOD".to_string()));
                }
                
                let value = self.stack[self.sp];
                self.sp += 1;
                
                if self.ax == 0 {
                    return Err(CompilerError::VMError("Division by zero in modulo".to_string()));
                }
                
                self.ax = value % self.ax;
                self.pc += 1;
            },
            i if i == Opcode::PRTF as i64 => {
                // Simple printf implementation - not fully compatible with C
                println!("PRTF: {}", self.ax);
                self.pc += 1;
            },
            _ => {
                return Err(CompilerError::VMError(format!("Unknown opcode: {}", op)));
            }
        }
        Ok(())
    }
    
    #[allow(dead_code)]
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
    
    #[allow(dead_code)]
    fn load_char(&self, addr: usize) -> Result<i64, CompilerError> {
        // Check if address is in data section
        if addr < self.data.len() {
            return Ok(self.data[addr] as i64);
        }
        
        Err(CompilerError::VMError(
            format!("Invalid memory access at address {} for char load", addr)
        ))
    }
    
    #[allow(dead_code)]
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
    
    #[allow(dead_code)]
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
    
    #[allow(dead_code)]
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