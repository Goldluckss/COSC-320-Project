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
}

/// Symbol table for managing variables and functions
pub struct SymbolTable {
    /// List of all symbols
    symbols: Vec<Symbol>,
    /// Stack of scope start indices
    scopes: Vec<usize>,
    /// Fast lookup by name (scope-prefixed)
    name_map: HashMap<String, usize>,
}

impl SymbolTable {
    /// Create a new symbol table
    pub fn new() -> Self {
        SymbolTable {
            symbols: Vec::new(),
            scopes: vec![0], // Start at global scope
            name_map: HashMap::new(),
        }
    }
    
    /// Add a symbol to the current scope
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
        let index = self.symbols.len();
        
        // Create the new symbol
        let symbol = Symbol {
            name: name.to_string(),
            class,
            typ,
            value,
        };
        
        // Add to table
        self.symbols.push(symbol);
        
        // Update the name map for current scope only
        if !self.scopes.is_empty() {
            let scope_prefix = if self.scopes.len() > 1 {
                // Use scope level as prefix for local variables
                format!("{}:{}", self.scopes.len() - 1, name)
            } else {
                // Global scope
                name.to_string()
            };
            
            self.name_map.insert(scope_prefix, index);
        }
        
        index
    }
    
    /// Find a symbol by name, searching from current scope up to global
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name to search for
    ///
    /// # Returns
    ///
    /// The symbol if found, None otherwise
    pub fn get(&self, name: &str) -> Option<Symbol> {
        // Start at current scope and work up
        for scope_level in (0..self.scopes.len()).rev() {
            let scope_prefix = if scope_level > 0 {
                // Local scope
                format!("{}:{}", scope_level, name)
            } else {
                // Global scope
                name.to_string()
            };
            
            if let Some(&index) = self.name_map.get(&scope_prefix) {
                return Some(self.symbols[index].clone());
            }
        }
        
        // Check for global without explicit scope prefix
        if let Some(&index) = self.name_map.get(name) {
            return Some(self.symbols[index].clone());
        }
        
        None
    }
    
    /// Check if a symbol exists in any accessible scope
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name to check
    ///
    /// # Returns
    ///
    /// True if the symbol exists in any accessible scope
    pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
    
    /// Check if a symbol exists in the current scope
    ///
    /// # Arguments
    ///
    /// * `name` - Symbol name to check
    ///
    /// # Returns
    ///
    /// True if the symbol exists in the current scope
    pub fn exists_in_current_scope(&self, name: &str) -> bool {
        if self.scopes.is_empty() {
            return false;
        }
        
        let scope_level = self.scopes.len() - 1;
        let scope_prefix = if scope_level > 0 {
            format!("{}:{}", scope_level, name)
        } else {
            name.to_string()
        };
        
        self.name_map.contains_key(&scope_prefix)
    }
    
    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(self.symbols.len());
    }
    
    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() <= 1 {
            // Don't exit global scope
            return;
        }
        
        // Remove all symbols in the current scope
        let start_index = self.scopes.pop().unwrap();
        
        // Remove scope entries from name map
        let scope_level = self.scopes.len();
        let keys_to_remove: Vec<String> = self.name_map.keys()
            .filter(|k| k.starts_with(&format!("{}:", scope_level)))
            .cloned()
            .collect();
            
        for key in keys_to_remove {
            self.name_map.remove(&key);
        }
        
        // Truncate symbols
        self.symbols.truncate(start_index);
    }
    
    /// Find the main function
    ///
    /// # Returns
    ///
    /// The main function symbol if found, None otherwise
    pub fn get_main(&self) -> Option<Symbol> {
        self.get("main")
    }
    
    /// Get all symbols
    ///
    /// # Returns
    ///
    /// A slice of all symbols
    pub fn get_symbols(&self) -> &[Symbol] {
        &self.symbols
    }
    
    /// Get symbol by index
    ///
    /// # Arguments
    ///
    /// * `index` - Symbol index
    ///
    /// # Returns
    ///
    /// The symbol at the given index, or None if out of bounds
    pub fn get_by_index(&self, index: usize) -> Option<&Symbol> {
        self.symbols.get(index)
    }
    
    /// Get the number of scopes
    ///
    /// # Returns
    ///
    /// The number of scopes
    pub fn get_scope_count(&self) -> usize {
        self.scopes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        
        // Add global symbols
        table.add("x", TokenType::Glo, Type::INT, 0);
        table.add("y", TokenType::Glo, Type::CHAR, 4);
        
        // Enter a function scope
        table.enter_scope();
        
        // Add local symbols
        table.add("a", TokenType::Loc, Type::INT, 0);
        table.add("b", TokenType::Loc, Type::INT, 1);
        
        // Enter a block scope
        table.enter_scope();
        
        // Add more locals
        table.add("c", TokenType::Loc, Type::INT, 2);
        
        // Shadow a global
        table.add("x", TokenType::Loc, Type::INT, 3);
        
        // Check symbol lookup
        let sym_c = table.get("c").unwrap();
        assert_eq!(sym_c.name, "c");
        assert_eq!(sym_c.class, TokenType::Loc);
        
        // The local x should shadow the global
        let sym_x = table.get("x").unwrap();
        assert_eq!(sym_x.class, TokenType::Loc);
        
        // Exit the block scope
        table.exit_scope();
        
        // c should no longer be accessible
        assert!(table.get("c").is_none());
        
        // x should now be the global
        let sym_x = table.get("x").unwrap();
        assert_eq!(sym_x.class, TokenType::Glo);
        
        // Exit the function scope
        table.exit_scope();
        
        // a and b should no longer be accessible
        assert!(table.get("a").is_none());
        assert!(table.get("b").is_none());
    }
    
    #[test]
    fn test_exist_in_current_scope() {
        let mut table = SymbolTable::new();
        
        // Add global symbols
        table.add("x", TokenType::Glo, Type::INT, 0);
        
        assert!(table.exists_in_current_scope("x"));
        assert!(!table.exists_in_current_scope("y"));
        
        // Enter a function scope
        table.enter_scope();
        
        assert!(!table.exists_in_current_scope("x")); // Not in current scope
        assert!(table.exists("x")); // But still accessible
        
        // Add local symbols
        table.add("a", TokenType::Loc, Type::INT, 0);
        
        assert!(table.exists_in_current_scope("a"));
    }
    
    #[test]
    fn test_function_symbol() {
        let mut table = SymbolTable::new();
        
        // Add a function
        let func_addr = 100;
        table.add("main", TokenType::Fun, Type::INT, func_addr);
        
        // Find main
        let main = table.get_main().unwrap();
        assert_eq!(main.name, "main");
        assert_eq!(main.class, TokenType::Fun);
        assert_eq!(main.value, func_addr);
    }
}