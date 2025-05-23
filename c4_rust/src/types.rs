/// Token types for the lexer and parser
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TokenType {
    // EOF sentinel
    Eof,
    
    // Keywords
    Char,
    Else,
    Enum,
    If,
    Int,
    Return,
    Sizeof,
    While,
    Void,   // Added to match C4.c
    
    // Variable/function classes
    Num,
    Fun,
    Sys,
    Glo,
    Loc,
    Id,
    
    // Operators (in precedence order)
    Assign,  // =
    Cond,    // ?
    Lor,     // ||
    Lan,     // &&
    Or,      // |
    Xor,     // ^
    And,     // &
    Eq,      // ==
    Ne,      // !=
    Lt,      // <
    Gt,      // >
    Le,      // <=
    Ge,      // >=
    Shl,     // <<
    Shr,     // >>
    Add,     // +
    Sub,     // -
    Mul,     // *
    Div,     // /
    Mod,     // %
    Inc,     // ++
    Dec,     // --
    Brak,    // [
    
    // Single character tokens
    Semicolon,  // ;
    LBrace,     // {
    RBrace,     // }
    LParen,     // (
    RParen,     // )
    RBracket,   // ]
    Comma,      // ,
    Colon,      // :
    Tilde,      // ~
}

impl TokenType {
    /// Get the precedence level of an operator token
    /// 
    /// Higher values mean higher precedence
    pub fn precedence(&self) -> usize {
        match self {
            TokenType::Assign => 2,
            TokenType::Cond => 4,
            TokenType::Lor => 6,
            TokenType::Lan => 8,
            TokenType::Or => 10,
            TokenType::Xor => 12,
            TokenType::And => 14,
            TokenType::Eq | TokenType::Ne => 16,
            TokenType::Lt | TokenType::Gt | TokenType::Le | TokenType::Ge => 18,
            TokenType::Shl | TokenType::Shr => 20,
            TokenType::Add | TokenType::Sub => 22,
            TokenType::Mul | TokenType::Div | TokenType::Mod => 24,
            TokenType::Inc | TokenType::Dec => 26,
            TokenType::Brak => 28,
            _ => 0,
        }
    }
}

impl PartialOrd for TokenType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.precedence().partial_cmp(&other.precedence())
    }
}

/// VM operation codes
/// 
/// These are the instructions understood by the VM,
/// directly mapping to C4's opcodes
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
    LEA,    // Load effective address
    IMM,    // Load immediate value
    JMP,    // Jump
    JSR,    // Jump to subroutine
    BZ,     // Branch if zero
    BNZ,    // Branch if not zero
    ENT,    // Enter subroutine
    ADJ,    // Adjust stack
    LEV,    // Leave subroutine
    LI,     // Load int
    LC,     // Load char
    SI,     // Store int
    SC,     // Store char
    PSH,    // Push
    
    // Arithmetic and logical operations
    OR,     // Bitwise OR
    XOR,    // Bitwise XOR
    AND,    // Bitwise AND
    EQ,     // Equal
    NE,     // Not equal
    LT,     // Less than
    GT,     // Greater than
    LE,     // Less than or equal
    GE,     // Greater than or equal
    SHL,    // Shift left
    SHR,    // Shift right
    ADD,    // Add
    SUB,    // Subtract
    MUL,    // Multiply
    DIV,    // Divide
    MOD,    // Modulo
    NEG,    // Negate
    
    // System calls
    OPEN,   // Open file
    READ,   // Read from file
    CLOS,   // Close file
    PRTF,   // Printf
    MALC,   // Malloc
    FREE,   // Free
    MSET,   // Memset
    MCMP,   // Memcmp
    EXIT,   // Exit
}

impl Opcode {
    /// Convert opcode to string representation for debugging
    pub fn to_string(&self) -> &'static str {
        match self {
            Opcode::LEA => "LEA", Opcode::IMM => "IMM", Opcode::JMP => "JMP",
            Opcode::JSR => "JSR", Opcode::BZ => "BZ", Opcode::BNZ => "BNZ", 
            Opcode::ENT => "ENT", Opcode::ADJ => "ADJ", Opcode::LEV => "LEV", 
            Opcode::LI => "LI", Opcode::LC => "LC", Opcode::SI => "SI", 
            Opcode::SC => "SC", Opcode::PSH => "PSH", Opcode::OR => "OR", 
            Opcode::XOR => "XOR", Opcode::AND => "AND", Opcode::EQ => "EQ", 
            Opcode::NE => "NE", Opcode::LT => "LT", Opcode::GT => "GT", 
            Opcode::LE => "LE", Opcode::GE => "GE", Opcode::SHL => "SHL", 
            Opcode::SHR => "SHR", Opcode::ADD => "ADD", Opcode::SUB => "SUB", 
            Opcode::MUL => "MUL", Opcode::DIV => "DIV", Opcode::MOD => "MOD", 
            Opcode::NEG => "NEG", Opcode::OPEN => "OPEN", Opcode::READ => "READ", 
            Opcode::CLOS => "CLOS", Opcode::PRTF => "PRTF", Opcode::MALC => "MALC", 
            Opcode::FREE => "FREE", Opcode::MSET => "MSET", Opcode::MCMP => "MCMP", 
            Opcode::EXIT => "EXIT",
        }
    }
}

/// Type system
/// 
/// The C4 compiler handles char, int, and pointer types
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Type {
    CHAR = 0,   // Character type (8-bit)
    INT = 1,    // Integer type (64-bit)
    PTR = 2,    // Pointer type (starts at 2 and increments for each level of indirection)
}

impl Type {
    /// Create a pointer to this type
    pub fn to_ptr(self) -> Self {
        match self {
            Type::CHAR => Type::PTR,
            Type::INT => Type::PTR,
            Type::PTR => {
                // Create a pointer to pointer (PTR + 1)
                // This mimics C4's behavior where pointer types are represented by integers
                let ptr_level = self as u8 + 1;
                unsafe { std::mem::transmute::<u8, Type>(ptr_level) }
            }
        }
    }
    
    /// Check if this is a pointer type
    pub fn is_ptr(self) -> bool {
        match self {
            Type::PTR => true,
            _ => (self as i32) > Type::PTR as i32,
        }
    }
    
    /// Get the size of this type in bytes
    pub fn size(self) -> usize {
        match self {
            Type::CHAR => 1,
            _ => std::mem::size_of::<i64>(), // Use i64 for INT and PTR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_precedence() {
        assert!(TokenType::Mul > TokenType::Add);
        assert!(TokenType::Add > TokenType::Eq);
        assert!(TokenType::Eq > TokenType::And);
        assert!(TokenType::And > TokenType::Or);
    }
    
    #[test]
    fn test_type_ptr() {
        let int_t = Type::INT;
        let ptr_t = int_t.to_ptr();
        
        assert_eq!(ptr_t, Type::PTR);
        assert!(ptr_t.is_ptr());
        
        let ptr_to_ptr_t = ptr_t.to_ptr();
        assert!(ptr_to_ptr_t.is_ptr());
        assert!(ptr_to_ptr_t as i32 > ptr_t as i32);
    }
    
    #[test]
    fn test_type_size() {
        assert_eq!(Type::CHAR.size(), 1);
        assert_eq!(Type::INT.size(), std::mem::size_of::<i64>());
        assert_eq!(Type::PTR.size(), std::mem::size_of::<i64>());
    }
}