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

    pub(crate) fn get_or_panic(&self, symbol_id: &SymbolId) -> Type {
        self.entries
            .get(symbol_id)
            .copied()
            .unwrap_or_else(|| panic!("type not found for symbol {:?}", symbol_id))
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
    #[should_panic]
    fn attempt_to_get_type_for_a_non_existent_symbol() {
        let table = TypeTable::new();
        table.get_or_panic(&SymbolId(1));
    }

    #[test]
    fn add_symbol_inserts_type_successfully() {
        let mut table = TypeTable::new();
        table.add(SymbolId(1), Type::Int32);

        assert_eq!(table.get_or_panic(&SymbolId(1)), Type::Int32);
    }

    #[test]
    fn add_symbol_retains_different_inserted_types() {
        let mut table = TypeTable::new();
        table.add(SymbolId(1), Type::Int32);
        table.add(SymbolId(2), Type::Bool);

        assert_eq!(table.get_or_panic(&SymbolId(2)), Type::Bool);
    }

    #[test]
    fn get_or_panic_returns_type_when_present() {
        let mut table = TypeTable::new();
        table.add(SymbolId(1), Type::Int32);
        assert_eq!(table.get_or_panic(&SymbolId(1)), Type::Int32);
    }

    #[test]
    #[should_panic(expected = "type not found for symbol SymbolId(1)")]
    fn get_or_panic_panics_when_missing() {
        let table = TypeTable::new();
        table.get_or_panic(&SymbolId(1));
    }
}
