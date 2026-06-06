use crate::semantic::SymbolId;
use std::collections::HashMap;

pub(crate) struct ResolutionTable {
    symbol_id_by_node_id: HashMap<usize, SymbolId>,
}

impl ResolutionTable {
    pub(crate) fn new() -> Self {
        Self {
            symbol_id_by_node_id: HashMap::new(),
        }
    }

    pub(crate) fn resolve(&mut self, node_id: usize, symbol_id: SymbolId) {
        self.symbol_id_by_node_id.insert(node_id, symbol_id);
    }

    pub(crate) fn get(&self, node_id: usize) -> Option<SymbolId> {
        self.symbol_id_by_node_id.get(&node_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_table_resolves_and_retrieves_symbol_by_node_id() {
        let mut table = ResolutionTable::new();
        let node_id = 42;
        let symbol_id = SymbolId(10);

        table.resolve(node_id, symbol_id);

        assert_eq!(table.get(node_id), Some(symbol_id));
    }

    #[test]
    fn resolution_table_returns_none_for_an_unresolved_node_id() {
        let table = ResolutionTable::new();
        assert_eq!(table.get(999), None);
    }
}
