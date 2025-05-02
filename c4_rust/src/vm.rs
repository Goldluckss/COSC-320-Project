use crate::error::CompilerError;
use crate::types::Opcode;
use std::io::{self, Read, Write};
use std::process;

/// Virtual Machine for executing compiled C4 code
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
    pub fn new(code: Vec<i64>, data: Vec<u8>, stack_size: usize, debug: bool) -> Self {
        let mut stack = vec![0; stack_size];
        
        VirtualMachine {
            pc: 0,
            sp: stack_size - 1,
            bp: stack_size - 1,
            ax: 0,
            code,
            stack,
            data,
            debug,
            cycle: 0,
        }
    }
    
    /// Run the VM starting at the specified entry point
    pub fn run(&mut self, entry_point: usize, args: &[String]) -> Result<i64, CompilerError> {
        // Setup stack for main()
        self.pc = entry_point;
        
        // Setup argc, argv
        self.stack[self.sp] = args.len() as i64;
        self.sp -= 1;
        
        // Push argv pointer (dummy for now)
        self.stack[self.sp] = 0;
        self.sp -= 1;
        
        // Setup return address (EXIT)
        self.stack[self.sp] = Opcode::EXIT as i64;
        self.sp -= 1;
        
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
            self.pc += 1;
            
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
                    if self.pc < self.code.len() {
                        println!(" {}", self.code[self.pc]);
                    } else {
                        println!(" ???");
                    }
                } else {
                    println!();
                }
            }
            
            // Execute instruction
            match op {
                i if i == Opcode::LEA as i64 => {
                    // Load effective address
                    self.ax = self.bp as i64 + self.code[self.pc];
                    self.pc += 1;
                },
                i if i == Opcode::IMM as i64 => {
                    // Load immediate value into accumulator
                    self.ax = self.code[self.pc];
                    self.pc += 1;
                },
                i if i == Opcode::JMP as i64 => {
                    // Jump
                    self.pc = self.code[self.pc] as usize;
                },
                i if i == Opcode::JSR as i64 => {
                    // Jump to subroutine
                    self.stack[self.sp] = self.pc as i64 + 1;
                    self.sp -= 1;
                    self.pc = self.code[self.pc] as usize;
                },
                i if i == Opcode::BZ as i64 => {
                    // Branch if zero
                    if self.ax == 0 {
                        self.pc = self.code[self.pc] as usize;
                    } else {
                        self.pc += 1;
                    }
                },
                i if i == Opcode::BNZ as i64 => {
                    // Branch if not zero
                    if self.ax != 0 {
                        self.pc = self.code[self.pc] as usize;
                    } else {
                        self.pc += 1;
                    }
                },
                i if i == Opcode::ENT as i64 => {
                    // Enter subroutine
                    self.stack[self.sp] = self.bp as i64;
                    self.sp -= 1;
                    self.bp = self.sp;
                    self.sp -= self.code[self.pc] as usize;
                    self.pc += 1;
                },
                i if i == Opcode::ADJ as i64 => {
                    // Adjust stack
                    self.sp += self.code[self.pc] as usize;
                    self.pc += 1;
                },
                i if i == Opcode::LEV as i64 => {
                    // Leave subroutine
                    self.sp = self.bp;
                    self.bp = self.stack[self.sp + 1] as usize;
                    self.pc = self.stack[self.sp + 2] as usize;
                },
                i if i == Opcode::LI as i64 => {
                    // Load int
                    self.ax = self.load_int(self.ax as usize)?;
                },
                i if i == Opcode::LC as i64 => {
                    // Load char
                    self.ax = self.load_char(self.ax as usize)?;
                },
                i if i == Opcode::SI as i64 => {
                    // Store int
                    let addr = self.stack[self.sp + 1] as usize;
                    self.store_int(addr, self.ax)?;
                    self.sp += 1;
                },
                i if i == Opcode::SC as i64 => {
                    // Store char
                    let addr = self.stack[self.sp + 1] as usize;
                    self.store_char(addr, self.ax as u8)?;
                    self.sp += 1;
                },
                i if i == Opcode::PSH as i64 => {
                    // Push accumulator onto stack
                    self.stack[self.sp] = self.ax;
                    self.sp -= 1;
                },
                i if i == Opcode::OR as i64 => {
                    // Bitwise OR
                    self.ax = self.stack[self.sp + 1] | self.ax;
                    self.sp += 1;
                },
                i if i == Opcode::XOR as i64 => {
                    // Bitwise XOR
                    self.ax = self.stack[self.sp + 1] ^ self.ax;
                    self.sp += 1;
                },
                i if i == Opcode::AND as i64 => {
                    // Bitwise AND
                    self.ax = self.stack[self.sp + 1] & self.ax;
                    self.sp += 1;
                },
                i if i == Opcode::EQ as i64 => {
                    // Equal
                    self.ax = (self.stack[self.sp + 1] == self.ax) as i64;
                    self.sp += 1;
                },
                i if i == Opcode::NE as i64 => {
                    // Not equal
                    self.ax = (self.stack[self.sp + 1] != self.ax) as i64;
                    self.sp += 1;
                },
                i if i == Opcode::LT as i64 => {
                    // Less than
                    self.ax = (self.stack[self.sp + 1] < self.ax) as i64;
                    self.sp += 1;
                },
                i if i == Opcode::GT as i64 => {
                    // Greater than
                    self.ax = (self.stack[self.sp + 1] > self.ax) as i64;
                    self.sp += 1;
                },
                i if i == Opcode::LE as i64 => {
                    // Less than or equal
                    self.ax = (self.stack[self.sp + 1] <= self.ax) as i64;
                    self.sp += 1;
                },
                i if i == Opcode::GE as i64 => {
                    // Greater than or equal
                    self.ax = (self.stack[self.sp + 1] >= self.ax) as i64;
                    self.sp += 1;
                },
                i if i == Opcode::SHL as i64 => {
                    // Shift left
                    self.ax = self.stack[self.sp + 1] << self.ax;
                    self.sp += 1;
                },
                i if i == Opcode::SHR as i64 => {
                    // Shift right
                    self.ax = self.stack[self.sp + 1] >> self.ax;
                    self.sp += 1;
                },
                i if i == Opcode::ADD as i64 => {
                    // Add
                    let b = self.stack[self.sp + 1];
                    let a = self.stack[self.sp + 2];
                    self.ax = a + b;
                    self.sp += 1;  // Pop one operand, keep result in accumulator
                },
                i if i == Opcode::SUB as i64 => {
                    // Subtract
                    let b = self.stack[self.sp + 1];
                    let a = self.stack[self.sp + 2];
                    self.ax = a - b;
                    self.sp += 1;  // Pop one operand, keep result in accumulator
                },
                i if i == Opcode::MUL as i64 => {
                    // Multiply
                    let b = self.stack[self.sp + 1];
                    let a = self.stack[self.sp + 2];
                    self.ax = a * b;
                    self.sp += 1;  // Pop one operand, keep result in accumulator
                },
                i if i == Opcode::DIV as i64 => {
                    // Divide
                    let b = self.stack[self.sp + 1];
                    let a = self.stack[self.sp + 2];
                    if b == 0 {
                        return Err(CompilerError::VMError("Division by zero".to_string()));
                    }
                    self.ax = a / b;
                    self.sp += 1;  // Pop one operand, keep result in accumulator
                },
                i if i == Opcode::MOD as i64 => {
                    // Modulo
                    let b = self.stack[self.sp + 1];
                    let a = self.stack[self.sp + 2];
                    if b == 0 {
                        return Err(CompilerError::VMError("Modulo by zero".to_string()));
                    }
                    self.ax = a % b;
                    self.sp += 1;  // Pop one operand, keep result in accumulator
                },
                i if i == Opcode::OPEN as i64 => {
                    // Open file - not implemented in this version
                    self.ax = -1; // Return error
                },
                i if i == Opcode::READ as i64 => {
                    // Read from file - not implemented in this version
                    self.ax = 0; // Return 0 bytes read
                },
                i if i == Opcode::CLOS as i64 => {
                    // Close file - not implemented in this version
                    self.ax = 0; // Return success
                },
                i if i == Opcode::PRTF as i64 => {
                    // Printf - simplified implementation
                    let fmt_addr = self.stack[self.sp + 1] as usize;
                    let fmt = self.read_string(fmt_addr)?;
                    
                    // Parse format string
                    let mut result = String::new();
                    let mut i = 0;
                    let mut arg_index = 2;
                    
                    while i < fmt.len() {
                        if fmt[i] == b'%' {
                            i += 1;
                            if i >= fmt.len() {
                                break;
                            }
                            
                            match fmt[i] {
                                b'd' => {
                                    // Integer
                                    if self.sp + arg_index < self.stack.len() {
                                        result.push_str(&self.stack[self.sp + arg_index].to_string());
                                        arg_index += 1;
                                    }
                                },
                                b'c' => {
                                    // Character
                                    if self.sp + arg_index < self.stack.len() {
                                        let c = self.stack[self.sp + arg_index] as u8;
                                        result.push(c as char);
                                        arg_index += 1;
                                    }
                                },
                                b's' => {
                                    // String
                                    if self.sp + arg_index < self.stack.len() {
                                        let str_addr = self.stack[self.sp + arg_index] as usize;
                                        let s = self.read_string(str_addr)?;
                                        result.push_str(&String::from_utf8_lossy(&s));
                                        arg_index += 1;
                                    }
                                },
                                b'%' => {
                                    // Literal %
                                    result.push('%');
                                },
                                _ => {
                                    // Unsupported format specifier
                                    result.push('%');
                                    result.push(fmt[i] as char);
                                }
                            }
                        } else {
                            result.push(fmt[i] as char);
                        }
                        
                        i += 1;
                    }
                    
                    // Print the result
                    print!("{}", result);
                    io::stdout().flush().unwrap();
                    
                    // Return number of characters printed
                    self.ax = result.len() as i64;
                },
                i if i == Opcode::MALC as i64 => {
                    // Malloc - not implemented properly in this version
                    // Just allocate in the data section (not safe for real use)
                    let size = self.stack[self.sp + 1] as usize;
                    let addr = self.data.len();
                    self.data.resize(addr + size, 0);
                    self.ax = addr as i64;
                },
                i if i == Opcode::FREE as i64 => {
                    // Free - not implemented in this version
                    // Do nothing, memory is never freed
                },
                i if i == Opcode::MSET as i64 => {
                    // Memset
                    let addr = self.stack[self.sp + 3] as usize;
                    let val = self.stack[self.sp + 2] as u8;
                    let count = self.stack[self.sp + 1] as usize;
                    
                    if addr + count <= self.data.len() {
                        for i in 0..count {
                            self.data[addr + i] = val;
                        }
                    }
                    
                    self.ax = addr as i64;
                },
                i if i == Opcode::MCMP as i64 => {
                    // Memcmp
                    let addr1 = self.stack[self.sp + 3] as usize;
                    let addr2 = self.stack[self.sp + 2] as usize;
                    let count = self.stack[self.sp + 1] as usize;
                    
                    let mut result = 0;
                    
                    if addr1 + count <= self.data.len() && addr2 + count <= self.data.len() {
                        for i in 0..count {
                            let v1 = self.data[addr1 + i];
                            let v2 = self.data[addr2 + i];
                            
                            if v1 != v2 {
                                result = v1 as i64 - v2 as i64;
                                break;
                            }
                        }
                    }
                    
                    self.ax = result;
                },
                i if i == Opcode::EXIT as i64 => {
                    // Exit
                    if self.debug {
                        println!("exit({}) cycle = {}", self.ax, self.cycle);
                    }
                    return Ok(self.ax);
                },
                _ => {
                    return Err(CompilerError::VMError(
                        format!("Unknown instruction: {}", op)
                    ));
                }
            }
            
            // Check stack overflow/underflow
            if self.sp < 0 || self.sp >= self.stack.len() {
                return Err(CompilerError::VMError(
                    format!("Stack pointer out of bounds: {}", self.sp)
                ));
            }
        }
    }
    
    /// Load an integer from memory
    fn load_int(&self, addr: usize) -> Result<i64, CompilerError> {
        // Check if address is in stack
        if addr >= self.stack.as_ptr() as usize && 
           addr < (self.stack.as_ptr() as usize) + (self.stack.len() * std::mem::size_of::<i64>()) {
            let index = (addr - (self.stack.as_ptr() as usize)) / std::mem::size_of::<i64>();
            if index < self.stack.len() {
                return Ok(self.stack[index]);
            }
        }
        
        // Check if address is in data section
        if addr < self.data.len() {
            let mut value: i64 = 0;
            let bytes = addr.min(self.data.len() - std::mem::size_of::<i64>());
            
            for i in 0..std::mem::size_of::<i64>() {
                if bytes + i < self.data.len() {
                    value |= (self.data[bytes + i] as i64) << (i * 8);
                }
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
        // Check if address is in stack
        if addr >= self.stack.as_ptr() as usize && 
           addr < (self.stack.as_ptr() as usize) + (self.stack.len() * std::mem::size_of::<i64>()) {
            let index = (addr - (self.stack.as_ptr() as usize)) / std::mem::size_of::<i64>();
            if index < self.stack.len() {
                self.stack[index] = value;
                return Ok(());
            }
        }
        
        // Check if address is in data section
        if addr < self.data.len() && addr + std::mem::size_of::<i64>() <= self.data.len() {
            for i in 0..std::mem::size_of::<i64>() {
                self.data[addr + i] = ((value >> (i * 8)) & 0xFF) as u8;
            }
            return Ok(());
        }
        
        Err(CompilerError::VMError(
            format!("Invalid memory access at address {} for int store", addr)
        ))
    }
    
    /// Store a character to memory
    fn store_char(&mut self, addr: usize, value: u8) -> Result<(), CompilerError> {
        // Check if address is in data section
        if addr < self.data.len() {
            self.data[addr] = value;
            return Ok(());
        }
        
        Err(CompilerError::VMError(
            format!("Invalid memory access at address {} for char store", addr)
        ))
    }
    
    /// Read a null-terminated string from memory
    fn read_string(&self, addr: usize) -> Result<Vec<u8>, CompilerError> {
        if addr >= self.data.len() {
            return Err(CompilerError::VMError(
                format!("Invalid memory access at address {} for string read", addr)
            ));
        }
        
        let mut result = Vec::new();
        let mut i = addr;
        
        while i < self.data.len() {
            let c = self.data[i];
            if c == 0 {
                break;
            }
            result.push(c);
            i += 1;
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vm_simple_program() -> Result<(), CompilerError> {
        // Simple program: return 42
        let code = vec![
            Opcode::IMM as i64, 42,     // Load 42
            Opcode::EXIT as i64,        // Exit
        ];
        
        let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
        let result = vm.run(0, &[])?;
        
        assert_eq!(result, 42);
        
        Ok(())
    }
    
    #[test]
    fn test_vm_arithmetic() -> Result<(), CompilerError> {
        // Program: (2 + 3) * 4
        let code = vec![
            Opcode::IMM as i64, 2,      // Load 2
            Opcode::PSH as i64,         // Push it
            Opcode::IMM as i64, 3,      // Load 3
            Opcode::ADD as i64,         // Add
            Opcode::PSH as i64,         // Push result
            Opcode::IMM as i64, 4,      // Load 4
            Opcode::MUL as i64,         // Multiply
            Opcode::EXIT as i64,        // Exit
        ];
        
        let mut vm = VirtualMachine::new(code, Vec::new(), 1024, false);
        let result = vm.run(0, &[])?;
        
        assert_eq!(result, 20);
        
        Ok(())
    }
}