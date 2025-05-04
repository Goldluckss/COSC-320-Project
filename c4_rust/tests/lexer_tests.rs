use c4_rust::error::CompilerError;
use c4_rust::lexer::{Lexer, Token};
use c4_rust::types::TokenType;

/// Test basic tokenization
#[test]
fn test_basic_tokenization() -> Result<(), CompilerError> {
    let source = "int main() { return 42; }";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    // Expect tokens: Int, Id("main"), LParen, RParen, LBrace, Return, Num(42), Semicolon, RBrace
    let expected_types = vec![
        TokenType::Int,
        TokenType::Id,
        TokenType::LParen,
        TokenType::RParen,
        TokenType::LBrace,
        TokenType::Return,
        TokenType::Num,
        TokenType::Semicolon,
        TokenType::RBrace,
        TokenType::Eof,
    ];
    
    let expected_values = vec![
        None,
        None,
        None,
        None,
        None,
        None,
        Some(42),
        None,
        None,
        None,
    ];
    
    let expected_names = vec![
        None,
        Some("main".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ];
    
    for i in 0..expected_types.len() {
        let token = lexer.next_token()?;
        assert_eq!(token.token_type, expected_types[i], "Token {}: Expected {:?}, got {:?}", i, expected_types[i], token.token_type);
        assert_eq!(token.value, expected_values[i], "Token {}: Value mismatch", i);
        assert_eq!(token.name, expected_names[i], "Token {}: Name mismatch", i);
    }
    
    Ok(())
}

/// Test keywords
#[test]
fn test_keywords() -> Result<(), CompilerError> {
    let source = "int char if else while return sizeof enum void";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    let expected_types = vec![
        TokenType::Int,
        TokenType::Char,
        TokenType::If,
        TokenType::Else,
        TokenType::While,
        TokenType::Return,
        TokenType::Sizeof,
        TokenType::Enum,
        TokenType::Void,
        TokenType::Eof,
    ];
    
    for i in 0..expected_types.len() {
        let token = lexer.next_token()?;
        assert_eq!(token.token_type, expected_types[i], "Token {}: Expected {:?}, got {:?}", i, expected_types[i], token.token_type);
    }
    
    Ok(())
}

/// Test identifiers
#[test]
fn test_identifiers() -> Result<(), CompilerError> {
    let source = "main _underscore camelCase var123";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    let expected_names = vec![
        "main",
        "_underscore",
        "camelCase",
        "var123",
    ];
    
    for i in 0..expected_names.len() {
        let token = lexer.next_token()?;
        assert_eq!(token.token_type, TokenType::Id, "Token {} should be an Id", i);
        assert_eq!(token.name, Some(expected_names[i].to_string()), "Token {}: Expected name '{}', got {:?}", i, expected_names[i], token.name);
    }
    
    Ok(())
}

/// Test numeric literals
#[test]
fn test_numeric_literals() -> Result<(), CompilerError> {
    let source = "123 0 0x1A 077";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    let expected_values = vec![
        123,    // Decimal
        0,      // Zero
        26,     // Hex (0x1A)
        63,     // Octal (077)
    ];
    
    for i in 0..expected_values.len() {
        let token = lexer.next_token()?;
        assert_eq!(token.token_type, TokenType::Num, "Token {} should be a Num", i);
        assert_eq!(token.value, Some(expected_values[i]), "Token {}: Expected value {}, got {:?}", i, expected_values[i], token.value);
    }
    
    Ok(())
}

/// Test character and string literals
#[test]
fn test_char_string_literals() -> Result<(), CompilerError> {
    let source = "'A' '\\n' \"Hello, World!\"";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    // Character 'A'
    let token = lexer.next_token()?;
    assert_eq!(token.token_type, TokenType::Num);
    assert_eq!(token.value, Some('A' as i64));
    
    // Character '\n'
    let token = lexer.next_token()?;
    assert_eq!(token.token_type, TokenType::Num);
    assert_eq!(token.value, Some('\n' as i64));
    
    // String "Hello, World!"
    let token = lexer.next_token()?;
    assert_eq!(token.token_type, TokenType::Num); // String literal address
    assert!(token.name.is_some());
    assert_eq!(token.name.unwrap(), "Hello, World!");
    
    Ok(())
}

/// Test operators
#[test]
fn test_operators() -> Result<(), CompilerError> {
    let source = "+ - * / % == != < > <= >= << >> & | ^ && || = ? ~ ! ++ --";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    let expected_types = vec![
        TokenType::Add,
        TokenType::Sub,
        TokenType::Mul,
        TokenType::Div,
        TokenType::Mod,
        TokenType::Eq,
        TokenType::Ne,
        TokenType::Lt,
        TokenType::Gt,
        TokenType::Le,
        TokenType::Ge,
        TokenType::Shl,
        TokenType::Shr,
        TokenType::And,
        TokenType::Or,
        TokenType::Xor,
        TokenType::Lan,
        TokenType::Lor,
        TokenType::Assign,
        TokenType::Cond,
        TokenType::Tilde,
        TokenType::Tilde, // '!' is parsed as TokenType::Tilde
        TokenType::Inc,
        TokenType::Dec,
        TokenType::Eof,
    ];
    
    for i in 0..expected_types.len() {
        let token = lexer.next_token()?;
        assert_eq!(token.token_type, expected_types[i], 
            "Token {}: Expected {:?}, got {:?}", i, expected_types[i], token.token_type);
    }
    
    Ok(())
}

/// Test delimiters
#[test]
fn test_delimiters() -> Result<(), CompilerError> {
    let source = "{ } ( ) [ ] , ; :";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    let expected_types = vec![
        TokenType::LBrace,
        TokenType::RBrace,
        TokenType::LParen,
        TokenType::RParen, 
        TokenType::Brak,
        TokenType::RBracket,
        TokenType::Comma,
        TokenType::Semicolon,
        TokenType::Colon,
        TokenType::Eof,
    ];
    
    for i in 0..expected_types.len() {
        let token = lexer.next_token()?;
        assert_eq!(token.token_type, expected_types[i], 
            "Token {}: Expected {:?}, got {:?}", i, expected_types[i], token.token_type);
    }
    
    Ok(())
}

/// Test comments
#[test]
fn test_comments() -> Result<(), CompilerError> {
    let source = "int a; // This is a comment\nint b; /* This is a block comment */ int c;";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    // Expected tokens: Int, Id("a"), Semicolon, Int, Id("b"), Semicolon, Int, Id("c"), Semicolon
    let expected_names = vec![
        None,             // Int
        Some("a".to_string()),  // Id
        None,             // Semicolon
        None,             // Int
        Some("b".to_string()),  // Id
        None,             // Semicolon 
        None,             // Int
        Some("c".to_string()),  // Id
        None,             // Semicolon
        None,             // Eof
    ];
    
    for i in 0..expected_names.len() {
        let token = lexer.next_token()?;
        if let Some(expected_name) = &expected_names[i] {
            assert_eq!(token.name, Some(expected_name.clone()), 
                "Token {}: Expected name {:?}, got {:?}", i, expected_name, token.name);
        }
    }
    
    Ok(())
}

/// Test source code with error
#[test]
fn test_lexer_error() {
    let source = "int main() { @invalid }";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    // Consume tokens until we hit the error
    while lexer.next_token().is_ok() {
        if lexer.current_token().token_type == TokenType::Eof {
            panic!("Expected lexer error, but reached EOF");
        }
    }
    
    // We should have encountered an error at '@'
    assert!(lexer.line() > 0, "Line number should be tracked");
}

/// Test line and column tracking
#[test]
fn test_location_tracking() -> Result<(), CompilerError> {
    let source = "int main() {\n    return 42;\n}";
    let mut lexer = Lexer::new(source.to_string(), false);
    
    // Skip to 'return'
    lexer.next_token()?; // int
    lexer.next_token()?; // main
    lexer.next_token()?; // (
    lexer.next_token()?; // )
    lexer.next_token()?; // {
    
    // Check line and column for 'return'
    let return_token = lexer.next_token()?;
    assert_eq!(return_token.token_type, TokenType::Return);
    assert_eq!(lexer.line(), 2); // Line 2 (1-based)
    assert!(lexer.column() > 1); // Should be at column > 1
    
    Ok(())
}

/// Test that our lexer correctly handles the sample from C4
#[test]
fn test_c4_sample() -> Result<(), CompilerError> {
    // A small snippet from the original C4.c
    let source = r#"
        int main(int argc, char **argv) {
            int fd, bt, ty, poolsz, *idmain;
            int *pc, *sp, *bp, a, cycle; // vm registers
            int i, *t; // temps
            
            return 0;
        }
    "#;
    
    let mut lexer = Lexer::new(source.to_string(), false);
    
    // Just verify we can tokenize the whole thing without errors
    while lexer.next_token()?.token_type != TokenType::Eof {}
    
    Ok(())
}