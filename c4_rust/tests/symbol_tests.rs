use c4_rust::symbol::{Symbol, SymbolTable};
use c4_rust::types::{TokenType, Type};

/// Test basic symbol table functionality
#[test]
fn test_basic_symbol_table() {
    let mut table = SymbolTable::new();
    
    // Add a symbol
    let index = table.add("x", TokenType::Glo, Type::INT, 0);
    
    // Verify it was added
    assert_eq!(index, 0);
    assert!(table.exists("x"));
    
    // Check symbol properties
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.name, "x");
    assert_eq!(symbol.class, TokenType::Glo);
    assert_eq!(symbol.typ, Type::INT);
    assert_eq!(symbol.value, 0);
    
    // Add another symbol
    let index = table.add("y", TokenType::Glo, Type::CHAR, 8);
    
    // Verify it was added
    assert_eq!(index, 1);
    assert!(table.exists("y"));
    
    // Check symbol properties
    let symbol = table.get("y").unwrap();
    assert_eq!(symbol.name, "y");
    assert_eq!(symbol.class, TokenType::Glo);
    assert_eq!(symbol.typ, Type::CHAR);
    assert_eq!(symbol.value, 8);
}

/// Test symbol table scoping
#[test]
fn test_symbol_scoping() {
    let mut table = SymbolTable::new();
    
    // Add global symbol
    table.add("x", TokenType::Glo, Type::INT, 0);
    
    // Check scope level
    assert_eq!(table.current_scope_level(), 0);
    
    // Enter function scope
    table.enter_scope();
    assert_eq!(table.current_scope_level(), 1);
    
    // Add local in function scope
    table.add("y", TokenType::Loc, Type::INT, 1);
    
    // Should be able to see both x and y
    assert!(table.exists("x"));
    assert!(table.exists("y"));
    
    // Enter block scope
    table.enter_scope();
    assert_eq!(table.current_scope_level(), 2);
    
    // Add local in block scope
    table.add("z", TokenType::Loc, Type::INT, 2);
    
    // Should be able to see x, y, and z
    assert!(table.exists("x"));
    assert!(table.exists("y"));
    assert!(table.exists("z"));
    
    // Exit block scope
    table.exit_scope();
    assert_eq!(table.current_scope_level(), 1);
    
    // Should still see x and y, but not z
    assert!(table.exists("x"));
    assert!(table.exists("y"));
    assert!(!table.exists("z"));
    
    // Exit function scope
    table.exit_scope();
    assert_eq!(table.current_scope_level(), 0);
    
    // Should see only x
    assert!(table.exists("x"));
    assert!(!table.exists("y"));
    assert!(!table.exists("z"));
}

/// Test symbol table variable shadowing
#[test]
fn test_symbol_shadowing() {
    let mut table = SymbolTable::new();
    
    // Add global symbol
    table.add("x", TokenType::Glo, Type::INT, 0);
    
    // Check global x
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.class, TokenType::Glo);
    assert_eq!(symbol.value, 0);
    
    // Enter function scope
    table.enter_scope();
    
    // Add local that shadows global
    table.add("x", TokenType::Loc, Type::INT, 1);
    
    // Check local x (should shadow global)
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.class, TokenType::Loc);
    assert_eq!(symbol.value, 1);
    
    // Enter block scope
    table.enter_scope();
    
    // Add another local that shadows function local
    table.add("x", TokenType::Loc, Type::CHAR, 2);
    
    // Check innermost local x
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.class, TokenType::Loc);
    assert_eq!(symbol.typ, Type::CHAR);
    assert_eq!(symbol.value, 2);
    
    // Exit block scope
    table.exit_scope();
    
    // Should see function local x
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.class, TokenType::Loc);
    assert_eq!(symbol.typ, Type::INT);
    assert_eq!(symbol.value, 1);
    
    // Exit function scope
    table.exit_scope();
    
    // Should see global x
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.class, TokenType::Glo);
    assert_eq!(symbol.value, 0);
}

/// Test function symbol handling
#[test]
fn test_function_symbols() {
    let mut table = SymbolTable::new();
    
    // Add a function
    table.add("main", TokenType::Fun, Type::INT, 100);
    
    // Check function properties
    let symbol = table.get("main").unwrap();
    assert_eq!(symbol.name, "main");
    assert_eq!(symbol.class, TokenType::Fun);
    assert_eq!(symbol.typ, Type::INT);
    assert_eq!(symbol.value, 100);
    
    // Check main function lookup
    let main = table.get_main().unwrap();
    assert_eq!(main.name, "main");
    assert_eq!(main.value, 100);
    
    // Add another function
    table.add("add", TokenType::Fun, Type::INT, 200);
    
    // Check it was added
    let symbol = table.get("add").unwrap();
    assert_eq!(symbol.name, "add");
    assert_eq!(symbol.value, 200);
}

/// Test enum symbol handling
#[test]
fn test_enum_symbols() {
    let mut table = SymbolTable::new();
    
    // Add enum values
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
    
    // Test using enum values
    assert_eq!(red.value + green.value + blue.value, 3);
}

/// Test symbol state saving and restoring
#[test]
fn test_symbol_state() {
    // Create a symbol
    let mut symbol = Symbol::new("x", TokenType::Glo, Type::INT, 10);
    
    // Check initial state
    assert_eq!(symbol.class, TokenType::Glo);
    assert_eq!(symbol.typ, Type::INT);
    assert_eq!(symbol.value, 10);
    assert_eq!(symbol.h_class, None);
    assert_eq!(symbol.h_type, None);
    assert_eq!(symbol.h_value, None);
    
    // Save state
    symbol.save_state();
    
    // Check saved state
    assert_eq!(symbol.h_class, Some(TokenType::Glo));
    assert_eq!(symbol.h_type, Some(Type::INT));
    assert_eq!(symbol.h_value, Some(10));
    
    // Change current state
    symbol.class = TokenType::Loc;
    symbol.typ = Type::CHAR;
    symbol.value = 20;
    
    // Check changed state
    assert_eq!(symbol.class, TokenType::Loc);
    assert_eq!(symbol.typ, Type::CHAR);
    assert_eq!(symbol.value, 20);
    
    // Saved state should still be the original
    assert_eq!(symbol.h_class, Some(TokenType::Glo));
    assert_eq!(symbol.h_type, Some(Type::INT));
    assert_eq!(symbol.h_value, Some(10));
    
    // Restore state
    symbol.restore_state();
    
    // Check restored state
    assert_eq!(symbol.class, TokenType::Glo);
    assert_eq!(symbol.typ, Type::INT);
    assert_eq!(symbol.value, 10);
    
    // Saved state should be cleared
    assert_eq!(symbol.h_class, None);
    assert_eq!(symbol.h_type, None);
    assert_eq!(symbol.h_value, None);
}

/// Test symbol table iteration
#[test]
fn test_symbol_iteration() {
    let mut table = SymbolTable::new();
    
    // Add several symbols
    table.add("x", TokenType::Glo, Type::INT, 0);
    table.add("y", TokenType::Glo, Type::INT, 8);
    table.add("z", TokenType::Glo, Type::INT, 16);
    
    // Check table size
    assert_eq!(table.len(), 3);
    
    // Iterate and check all symbols
    let mut names = Vec::new();
    let mut values = Vec::new();
    
    for symbol in table.iter() {
        names.push(symbol.name.clone());
        values.push(symbol.value);
    }
    
    assert_eq!(names, vec!["x", "y", "z"]);
    assert_eq!(values, vec![0, 8, 16]);
    
    // Check specific symbols by index
    let symbol0 = table.get_by_index(0).unwrap();
    assert_eq!(symbol0.name, "x");
    
    let symbol1 = table.get_by_index(1).unwrap();
    assert_eq!(symbol1.name, "y");
    
    let symbol2 = table.get_by_index(2).unwrap();
    assert_eq!(symbol2.name, "z");
    
    // Out of bounds index should return None
    assert!(table.get_by_index(3).is_none());
}

/// Test mutable symbol access
#[test]
fn test_mutable_symbol_access() {
    let mut table = SymbolTable::new();
    
    // Add a symbol
    table.add("x", TokenType::Glo, Type::INT, 10);
    
    // Get mutable reference and modify
    {
        let symbol = table.get_mut("x").unwrap();
        symbol.value = 20;
    }
    
    // Check that modification worked
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.value, 20);
    
    // Get mutable reference by index and modify
    {
        let symbol = table.get_by_index_mut(0).unwrap();
        symbol.typ = Type::CHAR;
    }
    
    // Check that modification worked
    let symbol = table.get("x").unwrap();
    assert_eq!(symbol.typ, Type::CHAR);
}

/// Test system function symbols
#[test]
fn test_system_function_symbols() {
    let mut table = SymbolTable::new();
    
    // Add system functions (similar to how the parser would)
    table.add("open", TokenType::Sys, Type::INT, 30);  // OPEN opcode value
    table.add("printf", TokenType::Sys, Type::INT, 33); // PRTF opcode value
    
    // Check system function properties
    let open_fn = table.get("open").unwrap();
    assert_eq!(open_fn.class, TokenType::Sys);
    assert_eq!(open_fn.value, 30);
    
    let printf_fn = table.get("printf").unwrap();
    assert_eq!(printf_fn.class, TokenType::Sys);
    assert_eq!(printf_fn.value, 33);
}

/// Test handling of pointer types in symbols
#[test]
fn test_pointer_types() {
    let mut table = SymbolTable::new();
    
    // Add variables with pointer types
    table.add("x", TokenType::Glo, Type::INT, 0);
    table.add("p", TokenType::Glo, Type::INT.to_ptr(), 8);
    table.add("pp", TokenType::Glo, Type::INT.to_ptr().to_ptr(), 16);
    
    // Check basic pointer type
    let p = table.get("p").unwrap();
    assert!(p.typ.is_ptr());
    assert_eq!(p.typ, Type::PTR);
    
    // Check pointer to pointer type
    let pp = table.get("pp").unwrap();
    assert!(pp.typ.is_ptr());
    assert!(pp.typ as i32 > Type::PTR as i32);
}

/// Test current symbol access
#[test]
fn test_current_symbol() {
    let mut table = SymbolTable::new();
    
    // Add a symbol
    table.add("x", TokenType::Glo, Type::INT, 10);
    
    // Current symbol should be the last added
    let current = table.current_symbol().unwrap();
    assert_eq!(current.name, "x");
    
    // Add another symbol
    table.add("y", TokenType::Glo, Type::INT, 20);
    
    // Current symbol should now be y
    let current = table.current_symbol().unwrap();
    assert_eq!(current.name, "y");
    
    // Modify current symbol
    {
        let current = table.current_symbol_mut().unwrap();
        current.value = 30;
    }
    
    // Check modification
    let y = table.get("y").unwrap();
    assert_eq!(y.value, 30);
}

/// Test edge cases for the symbol table
#[test]
fn test_symbol_table_edge_cases() {
    let mut table = SymbolTable::new();
    
    // Empty table
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
    assert!(table.get("nonexistent").is_none());
    assert!(table.current_symbol().is_none());
    
    // Check that exiting global scope does nothing
    table.exit_scope();
    assert_eq!(table.current_scope_level(), 0);
    
    // Check behavior of empty names (should be valid, though unusual)
    table.add("", TokenType::Glo, Type::INT, 0);
    assert!(table.exists(""));
    assert_eq!(table.get("").unwrap().name, "");
    
    // Ensure table works with many symbols
    for i in 0..100 {
        let name = format!("var{}", i);
        table.add(&name, TokenType::Glo, Type::INT, i);
    }
    
    assert_eq!(table.len(), 101);  // 100 new ones + the empty name
    
    // Check that all symbols are accessible
    for i in 0..100 {
        let name = format!("var{}", i);
        let var = table.get(&name).unwrap();
        assert_eq!(var.value, i);
    }
}

/// Test the symbol table with a simulated C program
#[test]
fn test_simulated_c_program() {
    let mut table = SymbolTable::new();
    
    // Global variables
    table.add("g_count", TokenType::Glo, Type::INT, 0);
    table.add("g_message", TokenType::Glo, Type::CHAR.to_ptr(), 8);
    
    // Enum
    table.add("RED", TokenType::Num, Type::INT, 0);
    table.add("GREEN", TokenType::Num, Type::INT, 1);
    table.add("BLUE", TokenType::Num, Type::INT, 2);
    
    // Function: int add(int a, int b)
    table.add("add", TokenType::Fun, Type::INT, 100);
    
    // Enter function scope
    table.enter_scope();
    
    // Function parameters
    table.add("a", TokenType::Loc, Type::INT, 1);
    table.add("b", TokenType::Loc, Type::INT, 2);
    
    // Local variables
    table.add("result", TokenType::Loc, Type::INT, -1);
    
    // Check all symbols are accessible
    assert!(table.exists("g_count"));
    assert!(table.exists("g_message"));
    assert!(table.exists("RED"));
    assert!(table.exists("add"));
    assert!(table.exists("a"));
    assert!(table.exists("b"));
    assert!(table.exists("result"));
    
    // Exit function scope
    table.exit_scope();
    
    // Function parameters and locals should no longer be accessible
    assert!(!table.exists("a"));
    assert!(!table.exists("b"));
    assert!(!table.exists("result"));
    
    // Globals should still be accessible
    assert!(table.exists("g_count"));
    assert!(table.exists("g_message"));
    assert!(table.exists("RED"));
    assert!(table.exists("add"));
    
    // Function: main
    table.add("main", TokenType::Fun, Type::INT, 200);
    
    // Check main function
    let main = table.get_main().unwrap();
    assert_eq!(main.name, "main");
    assert_eq!(main.value, 200);
}