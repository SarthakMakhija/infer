use std::cell::Cell;

pub(crate) mod analyzer;
pub(crate) mod error;
pub(crate) mod resolution;
pub(crate) mod scope;
pub(crate) mod state;
pub(crate) mod visitor;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub(crate) struct SymbolId(pub usize);

thread_local! {
    static ID: Cell<SymbolId> = const { Cell::new(SymbolId(0)) };
}

pub(crate) fn next_symbol_id() -> SymbolId {
    ID.with(|id| {
        let current = id.get();
        let next = SymbolId(current.0 + 1);
        id.set(next);
        next
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_id() {
        let first = next_symbol_id();

        assert_eq!(first, SymbolId(1));
    }

    #[test]
    fn next_symbol_id_generates_consecutive_symbols() {
        let first = next_symbol_id();
        let second = next_symbol_id();

        assert_eq!(first, SymbolId(1));
        assert_eq!(second, SymbolId(2));
    }
}
