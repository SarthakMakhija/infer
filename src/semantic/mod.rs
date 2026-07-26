pub(crate) mod error;
pub(crate) mod inference;
pub(crate) mod resolution_table;
pub(crate) mod scoping;
pub(crate) mod visitor;

/// Represents a unique identifier for a resolved symbol (variable, parameter, or function).
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub(crate) struct SymbolId(pub usize);
