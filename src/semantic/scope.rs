use crate::semantic::SymbolId;
use std::collections::HashMap;

pub(crate) struct Scopes {
    entries: Vec<Scope>,
}

struct Scope {
    symbols: HashMap<String, SymbolId>,
}

impl Scope {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, id: SymbolId) {
        self.symbols.insert(name, id);
    }

    fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    fn get(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).cloned()
    }
}

impl Scopes {
    pub(crate) fn new() -> Self {
        Self {
            entries: vec![Scope::new()],
        }
    }

    pub(crate) fn begin_scope(&mut self) {
        self.entries.push(Scope::new())
    }

    pub(crate) fn end_scope(&mut self) {
        self.entries.pop();
    }

    pub(crate) fn define(&mut self, name: String, id: SymbolId) {
        let scope = self.current_scope_mut();
        scope.define(name, id);
    }

    pub(crate) fn contains(&self, name: &str) -> bool {
        for scope in self.entries.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }

    pub(crate) fn contains_locally(&self, name: &str) -> bool {
        let scope = self.current_scope();
        scope.contains(name)
    }

    pub(crate) fn get(&self, name: &str) -> Option<SymbolId> {
        for scope in self.entries.iter().rev() {
            if let Some(symbol_id) = scope.get(name) {
                return Some(symbol_id);
            }
        }
        None
    }

    fn current_scope(&self) -> &Scope {
        self.entries.last().expect("No active scope is defined")
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.entries.last_mut().expect("No active scope is defined")
    }
}

#[cfg(test)]
mod scope_tests {
    use super::*;

    #[test]
    fn scope_defines_a_symbol() {
        let mut scope = Scope::new();
        scope.define("username".to_string(), SymbolId(1));

        assert!(scope.contains("username"));
    }

    #[test]
    fn scope_does_not_contain_a_symbol() {
        let scope = Scope::new();

        assert!(!scope.contains("username"));
    }

    #[test]
    fn scope_get_a_defined_symbol() {
        let mut scope = Scope::new();
        scope.define("username".to_string(), SymbolId(1));

        assert_eq!(scope.get("username"), Some(SymbolId(1)));
    }

    #[test]
    fn scope_returns_none_for_an_undefined_symbol() {
        let scope = Scope::new();

        assert_eq!(scope.get("username"), None);
    }
}

#[cfg(test)]
mod scopes_tests {
    use super::*;

    #[test]
    fn scope_contains_a_defined_symbol() {
        let mut scopes = Scopes::new();
        scopes.define("username".to_string(), SymbolId(1));

        assert!(scopes.contains("username"));
    }

    #[test]
    fn scope_contains_symbols_from_enclosing_scopes() {
        let mut scopes = Scopes::new();
        scopes.define("username".to_string(), SymbolId(1));

        scopes.begin_scope();
        scopes.define("user_age".to_string(), SymbolId(2));

        assert!(scopes.contains("username"));
        assert!(scopes.contains("user_age"));

        scopes.end_scope();
        assert!(scopes.contains("username"));
        assert!(!scopes.contains("user_age"));
    }

    #[test]
    fn innermost_scope_shadows_symbols_in_outer_scopes() {
        let mut scopes = Scopes::new();
        scopes.define("username".to_string(), SymbolId(1));

        scopes.begin_scope();
        // Inner scope contains "username"
        scopes.define("username".to_string(), SymbolId(2));
        assert!(scopes.contains("username"));

        scopes.end_scope();
        assert!(scopes.contains("username"));
    }

    #[test]
    fn scopes_contains_locally() {
        let mut scopes = Scopes::new();
        scopes.define("username".to_string(), SymbolId(1));

        assert!(scopes.contains_locally("username"));
    }

    #[test]
    fn scopes_contains_locally_only_checks_innermost_scope() {
        let mut scopes = Scopes::new();
        scopes.define("username".to_string(), SymbolId(1));

        // In the root scope, "username" is local
        assert!(scopes.contains_locally("username"));

        scopes.begin_scope();
        assert!(!scopes.contains_locally("username"));

        scopes.define("user_age".to_string(), SymbolId(2));
        assert!(scopes.contains_locally("user_age"));

        scopes.end_scope();
    }

    #[test]
    #[should_panic(expected = "No active scope is defined")]
    fn defining_a_symbol_without_active_scope_panics() {
        let mut scopes = Scopes::new();
        // Remove the default root scope
        scopes.end_scope();
        scopes.define("username".to_string(), SymbolId(1));
    }

    #[test]
    fn scopes_gets_the_defined_symbol() {
        let mut scopes = Scopes::new();
        scopes.define("username".to_string(), SymbolId(1));

        assert_eq!(scopes.get("username"), Some(SymbolId(1)));
    }

    #[test]
    fn scopes_get_symbol_by_scanning_innermost_to_outermost() {
        let mut scopes = Scopes::new();
        scopes.define("username".to_string(), SymbolId(1));

        scopes.begin_scope();

        // Shadows "username"
        scopes.define("username".to_string(), SymbolId(2));
        assert_eq!(scopes.get("username"), Some(SymbolId(2)));

        scopes.end_scope();
        // Restores to original resolution
        assert_eq!(scopes.get("username"), Some(SymbolId(1)));
    }

    #[test]
    fn scopes_returns_none_for_undefined_symbol() {
        let scopes = Scopes::new();
        assert_eq!(scopes.get("username"), None);
    }
}
