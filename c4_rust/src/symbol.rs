use crate::types::{TokenType, Type};
use std::collections::HashMap;

/// Symbol representation
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Name of the symbol
    pub name: String,
    /// Symbol class (Fun, Glo, Loc, Num, etc.)
    pub class: TokenType,
    /// Symbol type (INT, CHAR, PTR)
    pub typ: Type,
    /// Value or address
    pub value: i64,
    
    // Fields for saving local symbol state when entering a new scope
    pub h_class: Option<TokenType>,
    pub h_type: Option<Type>,
    pub h_value: Option<i64>,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(name: &str, class: TokenType, typ: Type, value: i64) -> Self {
        Symbol {
            name: name.to_string(),
            class,
            typ,
            value,
            h_class: None,
            h_type: None,
            h_value: None,
        }
    }
    
    /// Save the current state of the symbol
    pub fn save_state(&mut self) {
        self.h_class = Some(self.class);
        self.h_type = Some(self.typ);
        self.h_value = Some(self.value);
    }
    
    /// Restore the saved state
    pub fn restore_state(&mut self) {
        if let Some(class) = self.h_class {
            self.class = class;
        }
        if let Some(typ) = self.h_type {
            self.typ = typ;
        }
        if let Some(value) = self.h_value {
            self.value = value;
        }
        
        // Clear saved state
        self.h_class = None;
        self.h_type = None;
        self.h_value = None;
    }
}

/// Symbol table for managing variables and functions
pub struct SymbolTable {
    /// List of all symbols
    symbols: Vec<Symbol>,
    /// Map of symbol names to indices (for fast lookup)
    name_map: HashMap<String, usize>,
    /// Current scope level (0 = global)
    scope_level: usize,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        SymbolTable {
            symbols: Vec::new(),
            name_map: HashMap::new(),
            scope_level: 0, // Start at global scope
        }
    }
    
    /// Add a symbol to the symbol table
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name
    /// * `class` - Symbol class (Fun, Glo, Loc, etc.)
    /// * `typ` - Symbol type (INT, CHAR, PTR)
    /// * `value` - Symbol value or address
    ///
    /// # Returns
    ///
    /// The index of the added symbol
    pub fn add(&mut self, name: &str, class: TokenType, typ: Type, value: i64) -> usize {
        // Create a new symbol
        let symbol = Symbol::new(name, class, typ, value);
        let index = self.symbols.len();
        
        // Add to the lookup map
        self.name_map.insert(name.to_string(), index);
        
        // Add to the table
        self.symbols.push(symbol);
        
        index
    }
    
    /// Get a symbol by name
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name
    ///
    /// # Returns
    ///
    /// The symbol if found, or None
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.name_map.get(name).map(|&index| &self.symbols[index])
    }
    
    /// Get a mutable reference to a symbol
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name
    ///
    /// # Returns
    ///
    /// Mutable reference to the symbol if found, or None
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        if let Some(&index) = self.name_map.get(name) {
            Some(&mut self.symbols[index])
        } else {
            None
        }
    }
    
    /// Get a symbol by index
    ///
    /// # Arguments
    ///
    /// * `index` - Symbol index
    ///
    /// # Returns
    ///
    /// The symbol at the given index, or None
    pub fn get_by_index(&self, index: usize) -> Option<&Symbol> {
        self.symbols.get(index)
    }
    
    /// Get a mutable reference to a symbol by index
    ///
    /// # Arguments
    ///
    /// * `index` - Symbol index
    ///
    /// # Returns
    ///
    /// Mutable reference to the symbol at the given index, or None
    pub fn get_by_index_mut(&mut self, index: usize) -> Option<&mut Symbol> {
        self.symbols.get_mut(index)
    }
    
    /// Check if a symbol exists
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name
    ///
    /// # Returns
    ///
    /// True if the symbol exists, false otherwise
    pub fn exists(&self, name: &str) -> bool {
        self.name_map.contains_key(name)
    }
    
    /// Enter a new scope level
    pub fn enter_scope(&mut self) {
        self.scope_level += 1;
    }
    
    /// Exit the current scope level
    pub fn exit_scope(&mut self) {
        if self.scope_level > 0 {
            self.scope_level -= 1;
        }
    }
    
    /// Get the current scope level
    pub fn current_scope_level(&self) -> usize {
        self.scope_level
    }
    
    /// Get the number of symbols in the table
    pub fn len(&self) -> usize {
        self.symbols.len()
    }
    
    /// Check if the symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
    
    /// Get an iterator over all symbols
    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.iter()
    }
    
    /// Iterate over all symbols with mutable access
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Symbol> {
        self.symbols.iter_mut()
    }
    
    /// Get the main function 
    pub fn get_main(&self) -> Option<&Symbol> {
        self.get("main")
    }
    
    /// Get the current symbol being processed (last added)
    pub fn current_symbol(&self) -> Option<&Symbol> {
        self.symbols.last()
    }
    
    /// Get a mutable reference to the current symbol
    pub fn current_symbol_mut(&mut self) -> Option<&mut Symbol> {
        self.symbols.last_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new("test", TokenType::Glo, Type::INT, 42);
        
        assert_eq!(symbol.name, "test");
        assert_eq!(symbol.class, TokenType::Glo);
        assert_eq!(symbol.typ, Type::INT);
        assert_eq!(symbol.value, 42);
        assert_eq!(symbol.h_class, None);
        assert_eq!(symbol.h_type, None);
        assert_eq!(symbol.h_value, None);
    }
    
    #[test]
    fn test_symbol_state() {
        let mut symbol = Symbol::new("test", TokenType::Glo, Type::INT, 42);
        
        // Save state
        symbol.save_state();
        
        // Change values
        symbol.class = TokenType::Loc;
        symbol.typ = Type::CHAR;
        symbol.value = 100;
        
        // Check that saved state is stored
        assert_eq!(symbol.h_class, Some(TokenType::Glo));
        assert_eq!(symbol.h_type, Some(Type::INT));
        assert_eq!(symbol.h_value, Some(42));
        
        // Restore state
        symbol.restore_state();
        
        // Check restored values
        assert_eq!(symbol.class, TokenType::Glo);
        assert_eq!(symbol.typ, Type::INT);
        assert_eq!(symbol.value, 42);
        
        // Check that saved state is cleared
        assert_eq!(symbol.h_class, None);
        assert_eq!(symbol.h_type, None);
        assert_eq!(symbol.h_value, None);
    }
    
    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        
        // Add symbols
        let idx1 = table.add("var1", TokenType::Glo, Type::INT, 10);
        let idx2 = table.add("var2", TokenType::Glo, Type::CHAR, 20);
        
        // Check indices
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        
        // Check get by name
        let sym1 = table.get("var1").unwrap();
        assert_eq!(sym1.name, "var1");
        assert_eq!(sym1.value, 10);
        
        // Check get by index
        let sym2 = table.get_by_index(1).unwrap();
        assert_eq!(sym2.name, "var2");
        assert_eq!(sym2.value, 20);
        
        // Check exists
        assert!(table.exists("var1"));
        assert!(table.exists("var2"));
        assert!(!table.exists("var3"));
        
        // Check length
        assert_eq!(table.len(), 2);
        assert!(!table.is_empty());
        
        // Test scope levels
        assert_eq!(table.current_scope_level(), 0);
        table.enter_scope();
        assert_eq!(table.current_scope_level(), 1);
        table.enter_scope();
        assert_eq!(table.current_scope_level(), 2);
        table.exit_scope();
        assert_eq!(table.current_scope_level(), 1);
        table.exit_scope();
        assert_eq!(table.current_scope_level(), 0);
    }
    
    #[test]
    fn test_symbol_modification() {
        let mut table = SymbolTable::new();
        
        // Add a symbol
        table.add("var", TokenType::Glo, Type::INT, 10);
        
        // Modify the symbol
        {
            let sym = table.get_mut("var").unwrap();
            sym.value = 20;
        }
        
        // Check the modification
        let sym = table.get("var").unwrap();
        assert_eq!(sym.value, 20);
    }
    
    #[test]
    fn test_main_function() {
        let mut table = SymbolTable::new();
        
        // Add some symbols
        table.add("var", TokenType::Glo, Type::INT, 10);
        table.add("func", TokenType::Fun, Type::INT, 100);
        
        // No main function yet
        assert!(table.get_main().is_none());
        
        // Add main function
        table.add("main", TokenType::Fun, Type::INT, 200);
        
        // Now we have a main function
        let main = table.get_main().unwrap();
        assert_eq!(main.name, "main");
        assert_eq!(main.class, TokenType::Fun);
        assert_eq!(main.value, 200);
    }
}