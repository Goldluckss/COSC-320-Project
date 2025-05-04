use c4_rust::symbol::SymbolTable;
use c4_rust::types::{TokenType, Type};
use pretty_assertions::assert_eq;

#[test]
fn test_symbol_table_basic() {
    let mut table = SymbolTable::new();
    
    // Add global symbol
    table.add("x", TokenType::Glo, Type::INT, 0);
    
    // Check if symbol exists
    assert!(table.exists("x"));
    
    // Get symbol and check its properties
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.name, "x");
    assert_eq!(symbol.class, TokenType::Glo);
    assert_eq!(symbol.typ, Type::INT);
    assert_eq!(symbol.value, 0);
    
    // Check non-existent symbol
    assert!(!table.exists("y"));
    assert!(table.get("y").is_none());
}

#[test]
fn test_symbol_table_scopes() {
    let mut table = SymbolTable::new();
    
    // Global scope (level 0)
    table.add("global_var", TokenType::Glo, Type::INT, 0);
    assert!(table.exists("global_var"));
    assert_eq!(table.current_scope_level(), 0);
    
    // Enter function scope (level 1)
    table.enter_scope();
    assert_eq!(table.current_scope_level(), 1);
    
    table.add("local_var", TokenType::Loc, Type::INT, 1);
    assert!(table.exists("global_var")); // Global still accessible
    assert!(table.exists("local_var"));
    
    // Enter block scope (level 2)
    table.enter_scope();
    assert_eq!(table.current_scope_level(), 2);
    
    table.add("block_var", TokenType::Loc, Type::INT, 2);
    assert!(table.exists("global_var")); // Global still accessible
    assert!(table.exists("local_var"));  // Parent local still accessible
    assert!(table.exists("block_var"));
    
    // Check scope-specific existence
    assert!(table.exists_in_current_scope("block_var"));
    assert!(!table.exists_in_current_scope("local_var"));
    assert!(!table.exists_in_current_scope("global_var"));
    
    // Leave block scope
    table.exit_scope();
    assert_eq!(table.current_scope_level(), 1);
    
    assert!(table.exists("global_var")); // Global still accessible
    assert!(table.exists("local_var"));  // Local still accessible
    assert!(!table.exists("block_var")); // Block variable gone
    
    // Leave function scope
    table.exit_scope();
    assert_eq!(table.current_scope_level(), 0);
    
    assert!(table.exists("global_var")); // Global still accessible
    assert!(!table.exists("local_var")); // Local gone
}

#[test]
fn test_symbol_table_shadowing() {
    let mut table = SymbolTable::new();
    
    // Add global variable
    table.add("x", TokenType::Glo, Type::INT, 0);
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.class, TokenType::Glo);
    
    // Enter function scope
    table.enter_scope();
    
    // Add shadowing local variable
    table.add("x", TokenType::Loc, Type::CHAR, 1);
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.class, TokenType::Loc);
    assert_eq!(symbol.typ, Type::CHAR);
    
    // Exit function scope
    table.exit_scope();
    
    // Global should be visible again
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.class, TokenType::Glo);
    assert_eq!(symbol.typ, Type::INT);
}

#[test]
fn test_symbol_table_function_lookup() {
    let mut table = SymbolTable::new();
    
    // Add function
    let func_addr = 100;
    table.add("add", TokenType::Fun, Type::INT, func_addr);
    
    // Find function
    let function = table.get("add").unwrap();
    assert_eq!(function.name, "add");
    assert_eq!(function.class, TokenType::Fun);
    assert_eq!(function.value, func_addr);
    
    // Add main function and test get_main()
    table.add("main", TokenType::Fun, Type::INT, 200);
    let main = table.get_main().unwrap();
    assert_eq!(main.name, "main");
    assert_eq!(main.value, 200);
}

#[test]
fn test_symbol_table_multiple_symbols() {
    let mut table = SymbolTable::new();
    
    // Add multiple symbols
    table.add("var1", TokenType::Glo, Type::INT, 0);
    table.add("var2", TokenType::Glo, Type::CHAR, 4);
    table.add("func", TokenType::Fun, Type::INT, 100);
    
    // Get all symbols
    let symbols = table.get_symbols();
    assert_eq!(symbols.len(), 3);
    
    // Get symbol by index
    let symbol1 = table.get_by_index(0).unwrap();
    assert_eq!(symbol1.name, "var1");
    
    let symbol2 = table.get_by_index(1).unwrap();
    assert_eq!(symbol2.name, "var2");
    
    let symbol3 = table.get_by_index(2).unwrap();
    assert_eq!(symbol3.name, "func");
    
    // Test out of bounds
    assert!(table.get_by_index(3).is_none());
}

#[test]
fn test_symbol_table_nested_scopes() {
    let mut table = SymbolTable::new();
    
    // Global scope
    table.add("global", TokenType::Glo, Type::INT, 0);
    
    // Scope level 1
    table.enter_scope();
    table.add("level1_a", TokenType::Loc, Type::INT, 1);
    table.add("level1_b", TokenType::Loc, Type::INT, 2);
    
    // Scope level 2
    table.enter_scope();
    table.add("level2_a", TokenType::Loc, Type::INT, 3);
    
    // Scope level 3
    table.enter_scope();
    table.add("level3_a", TokenType::Loc, Type::INT, 4);
    
    // Check accessibility at deepest scope
    assert!(table.exists("global"));
    assert!(table.exists("level1_a"));
    assert!(table.exists("level1_b"));
    assert!(table.exists("level2_a"));
    assert!(table.exists("level3_a"));
    
    // Exit back to level 2
    table.exit_scope();
    assert!(table.exists("global"));
    assert!(table.exists("level1_a"));
    assert!(table.exists("level1_b"));
    assert!(table.exists("level2_a"));
    assert!(!table.exists("level3_a"));
    
    // Exit back to level 1
    table.exit_scope();
    assert!(table.exists("global"));
    assert!(table.exists("level1_a"));
    assert!(table.exists("level1_b"));
    assert!(!table.exists("level2_a"));
    
    // Exit back to global
    table.exit_scope();
    assert!(table.exists("global"));
    assert!(!table.exists("level1_a"));
    assert!(!table.exists("level1_b"));
    
    // Verify scope count
    assert_eq!(table.get_scope_count(), 1); // Global scope only
}

#[test]
fn test_symbol_table_type_handling() {
    let mut table = SymbolTable::new();
    
    // Test different types
    table.add("var_int", TokenType::Glo, Type::INT, 0);
    table.add("var_char", TokenType::Glo, Type::CHAR, 1);
    table.add("var_ptr", TokenType::Glo, Type::PTR, 2);
    
    // Check types
    let symbol_int = table.get("var_int").unwrap();
    assert_eq!(symbol_int.typ, Type::INT);
    
    let symbol_char = table.get("var_char").unwrap();
    assert_eq!(symbol_char.typ, Type::CHAR);
    
    let symbol_ptr = table.get("var_ptr").unwrap();
    assert_eq!(symbol_ptr.typ, Type::PTR);
}

#[test]
fn test_symbol_table_prevent_global_exit() {
    let mut table = SymbolTable::new();
    
    // Add symbol to global scope
    table.add("global", TokenType::Glo, Type::INT, 0);
    
    // Try to exit global scope (should be prevented)
    table.exit_scope();
    
    // Global scope should still exist
    assert_eq!(table.get_scope_count(), 1);
    assert!(table.exists("global"));
}