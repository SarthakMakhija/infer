use crate::semantic::inference::Type;
use crate::semantic::SymbolId;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub(crate) struct TypeTable {
    entries: HashMap<SymbolId, Type>,
}

impl TypeTable {
    pub(crate) fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, symbol_id: SymbolId, ty: Type) {
        self.entries.insert(symbol_id, ty);
    }

    #[cfg(test)]
    fn get(&self, symbol_id: &SymbolId) -> Option<Type> {
        self.entries.get(symbol_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_type_table() {
        let table = TypeTable::new();
        assert_eq!(table.entries.len(), 0);
    }

    #[test]
    fn attempt_to_get_type_for_a_non_existent_symbol() {
        let table = TypeTable::new();
        assert_eq!(table.get(&SymbolId(1)), None);
    }

    #[test]
    fn add_symbol_inserts_type_successfully() {
        let mut table = TypeTable::new();
        table.add(SymbolId(1), Type::Int32);

        assert_eq!(table.get(&SymbolId(1)), Some(Type::Int32));
    }

    #[test]
    fn add_symbol_retains_different_inserted_types() {
        let mut table = TypeTable::new();
        table.add(SymbolId(1), Type::Int32);
        table.add(SymbolId(2), Type::Bool);

        assert_eq!(table.get(&SymbolId(2)), Some(Type::Bool));
    }
}
