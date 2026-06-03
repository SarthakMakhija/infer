use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SymbolId(pub usize);

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
        let scope = self.entries.last_mut().expect("No active scope is defined");
        scope.define(name, id);
    }

    pub fn contains(&self, name: &str) -> bool {
        for scope in self.entries.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
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
    fn scope_resolves_symbols_from_enclosing_scopes() {
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
    #[should_panic(expected = "No active scope is defined")]
    fn defining_a_symbol_without_active_scope_panics() {
        let mut scopes = Scopes::new();
        // Remove the default root scope
        scopes.end_scope();
        scopes.define("username".to_string(), SymbolId(1));
    }
}
