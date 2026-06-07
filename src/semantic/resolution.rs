use crate::ast::statement::NodeId;
use crate::semantic::SymbolId;
use std::collections::HashMap;

pub(crate) struct ResolutionTable {
    symbol_id_by_node_id: HashMap<NodeId, SymbolId>,
}

impl ResolutionTable {
    pub(crate) fn new() -> Self {
        Self {
            symbol_id_by_node_id: HashMap::new(),
        }
    }

    pub(crate) fn resolve(&mut self, node_id: NodeId, symbol_id: SymbolId) {
        self.symbol_id_by_node_id.insert(node_id, symbol_id);
    }

    pub(crate) fn get(&self, node_id: &NodeId) -> Option<SymbolId> {
        self.symbol_id_by_node_id.get(node_id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_table_resolves_and_retrieves_symbol_by_node_id() {
        let mut table = ResolutionTable::new();
        let node_id = NodeId(42);
        let symbol_id = SymbolId(10);

        table.resolve(node_id, symbol_id);

        assert_eq!(table.get(&node_id), Some(symbol_id));
    }

    #[test]
    fn resolution_table_returns_none_for_an_unresolved_node_id() {
        let table = ResolutionTable::new();
        assert_eq!(table.get(&NodeId(42)), None);
    }
}
