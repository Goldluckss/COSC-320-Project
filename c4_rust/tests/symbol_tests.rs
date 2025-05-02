use c4_rust::symbol::SymbolTable;
use c4_rust::types::{TokenType, Type};

#[test]
fn test_nested_scopes() {
    let mut table = SymbolTable::new();
    
    // Global scope
    table.add("globalVar", TokenType::Glo, Type::INT, 0);
    
    // Function scope
    table.enter_scope();
    table.add("param1", TokenType::Loc, Type::INT, 0);
    table.add("param2", TokenType::Loc, Type::INT, 1);
    table.add("localVar", TokenType::Loc, Type::INT, 2);
    
    // Block scope 1
    table.enter_scope();
    table.add("blockVar1", TokenType::Loc, Type::INT, 3);
    
    // Can access all variables
    assert!(table.exists("globalVar"));
    assert!(table.exists("param1"));
    assert!(table.exists("param2"));
    assert!(table.exists("localVar"));
    assert!(table.exists("blockVar1"));
    
    // Block scope 2 (nested)
    table.enter_scope();
    table.add("blockVar2", TokenType::Loc, Type::INT, 4);
    
    // Shadow a variable
    table.add("globalVar", TokenType::Loc, Type::INT, 5);
    
    // Shadowed variable should be local now
    let shadowed = table.get("globalVar").unwrap();
    assert_eq!(shadowed.class, TokenType::Loc);
    assert_eq!(shadowed.value, 5);
    
    // Exit block scope 2
    table.exit_scope();
    
    // blockVar2 should be gone
    assert!(!table.exists("blockVar2"));
    
    // globalVar should be the global one again
    let global = table.get("globalVar").unwrap();
    assert_eq!(global.class, TokenType::Glo);
    assert_eq!(global.value, 0);
    
    // Exit block scope 1
    table.exit_scope();
    
    // blockVar1 should be gone
    assert!(!table.exists("blockVar1"));
    
    // Exit function scope
    table.exit_scope();
    
    // Only global variables should be accessible
    assert!(table.exists("globalVar"));
    assert!(!table.exists("param1"));
    assert!(!table.exists("param2"));
    assert!(!table.exists("localVar"));
}

#[test]
fn test_function_symbols() {
    let mut table = SymbolTable::new();
    
    // Add some system functions
    table.add("printf", TokenType::Sys, Type::INT, 0);
    table.add("malloc", TokenType::Sys, Type::INT, 1);
    
    // Add a user function
    let func_addr = 100;
    table.add("add", TokenType::Fun, Type::INT, func_addr);
    
    // Check function lookup
    let printf = table.get("printf").unwrap();
    assert_eq!(printf.name, "printf");
    assert_eq!(printf.class, TokenType::Sys);
    
    let add = table.get("add").unwrap();
    assert_eq!(add.name, "add");
    assert_eq!(add.class, TokenType::Fun);
    assert_eq!(add.value, func_addr);
    
    // Add main function
    table.add("main", TokenType::Fun, Type::INT, 200);
    
    // Get main function
    let main = table.get_main().unwrap();
    assert_eq!(main.name, "main");
    assert_eq!(main.class, TokenType::Fun);
    assert_eq!(main.value, 200);
}

#[test]
fn test_pointer_types() {
    let mut table = SymbolTable::new();
    
    // Add variables with different types
    table.add("charVar", TokenType::Glo, Type::CHAR, 0);
    table.add("intVar", TokenType::Glo, Type::INT, 4);
    table.add("charPtr", TokenType::Glo, Type::CHAR.to_ptr(), 8);
    table.add("intPtr", TokenType::Glo, Type::INT.to_ptr(), 16);
    table.add("ptrPtr", TokenType::Glo, Type::INT.to_ptr().to_ptr(), 24);
    
    // Check variable types
    let char_var = table.get("charVar").unwrap();
    assert_eq!(char_var.typ, Type::CHAR);
    assert!(!char_var.typ.is_ptr());
    
    let int_var = table.get("intVar").unwrap();
    assert_eq!(int_var.typ, Type::INT);
    assert!(!int_var.typ.is_ptr());
    
    let char_ptr = table.get("charPtr").unwrap();
    assert_eq!(char_ptr.typ, Type::PTR);
    assert!(char_ptr.typ.is_ptr());
    
    let int_ptr = table.get("intPtr").unwrap();
    assert_eq!(int_ptr.typ, Type::PTR);
    assert!(int_ptr.typ.is_ptr());
    
    let ptr_ptr = table.get("ptrPtr").unwrap();
    assert!(ptr_ptr.typ.is_ptr());
    assert!(ptr_ptr.typ as i32 > Type::PTR as i32); // Higher value for pointer-to-pointer
}

#[test]
fn test_enum_constants() {
    let mut table = SymbolTable::new();
    
    // Add enum constants
    table.add("RED", TokenType::Num, Type::INT, 0);
    table.add("GREEN", TokenType::Num, Type::INT, 1);
    table.add("BLUE", TokenType::Num, Type::INT, 2);
    
    // Check enum values
    let red = table.get("RED").unwrap();
    assert_eq!(red.class, TokenType::Num);
    assert_eq!(red.value, 0);
    
    let green = table.get("GREEN").unwrap();
    assert_eq!(green.value, 1);
    
    let blue = table.get("BLUE").unwrap();
    assert_eq!(blue.value, 2);
}