use crate::ast::statement::NodeId;
use crate::semantic::SymbolId;
use std::collections::HashMap;

/// A lookup table mapping AST node identifiers (`NodeId`) to their semantically resolved `SymbolId`.
///
/// # Rationale & Purpose
/// During lexical analysis and parsing, variable and function references are represented
/// simply as raw string names (e.g., `"x"` or `"score"`). However, in languages supporting block scoping,
/// shadowing, and nested function definitions, a single string name can refer to multiple
/// distinct variables depending on context.
///
/// To resolve this ambiguity, the symbol resolution phase (Sema Pass 1) walks the AST and assigns
/// a globally unique `SymbolId` to every variable declaration, function parameter, and function definition.
/// The `ResolutionTable` maps the `NodeId` of every identifier reference or assignment back to its
/// specific declaration's `SymbolId`. This decouples the variable's semantic identity from its string name.
///
/// ## Why `SymbolId` instead of Symbol Name?
/// If we mapped `NodeId` to the symbol's string name (e.g., mapping a reference to `"x"`), the type checker
/// would still suffer from ambiguity in the presence of shadowing. Multiple active scopes can define different
/// variables with the same string name `"x"`.
/// Using a numeric, globally unique `SymbolId` guarantees that two variables with the same string name
/// are treated as completely independent entities, allowing downstream compiler passes to differentiate them.
///
/// # Purpose at Type Inference Time (with Scoping Example)
/// Consider the following source code:
/// ```text
/// let x = 10;          // [VarDecl NodeId(1)] -> Declares x with SymbolId(1)
/// {
///     let x = "hello"; // [VarDecl NodeId(2)] -> Shadows and declares x with SymbolId(2)
///     print(x);        // [Identifier NodeId(3)] -> Reference to innermost x
/// }
/// print(x);            // [Identifier NodeId(4)] -> Reference to outermost x
/// ```
///
/// Without the `ResolutionTable`, a type checker would walk the AST, encounter two variables named `"x"`,
/// and get confused when resolving types (since `"x"` cannot simultaneously be `i32` and `string`).
///
/// With the `ResolutionTable`, the map resolves the nodes as follows:
/// - `NodeId(1)` (outer `VarDecl`) maps to `SymbolId(1)`
/// - `NodeId(2)` (inner `VarDecl`) maps to `SymbolId(2)`
/// - `NodeId(3)` (inner print argument) maps to `SymbolId(2)`
/// - `NodeId(4)` (outer print argument) maps to `SymbolId(1)`
///
/// During the type inference pass:
/// 1. The compiler assigns a distinct type variable to each `SymbolId` (e.g., `T1` for `SymbolId(1)`, `T2` for `SymbolId(2)`).
/// 2. When type-checking `print(x)` at `NodeId(3)`, it queries the table and learns it refers to `SymbolId(2)`. It writes the constraint:
///    `Type(SymbolId(2)) = string`
/// 3. When type-checking `print(x)` at `NodeId(4)`, it queries the table and learns it refers to `SymbolId(1)`. It writes the constraint:
///    `Type(SymbolId(1)) = i32`
///
/// In this way, type inference completely bypasses scoping logic and resolves constraints cleanly.
///
/// # General Compiler-Level Role
/// In production compilers (such as Rust's `rustc` or Java's frontend), this is the result of the
/// "Name Resolution" pass. It builds a flat, fast lookup structure that bridges the hierarchical AST
/// (syntax) with the Symbol Table (semantics). Downstream passes (type checking, borrow checking,
/// lints, optimization, and code generation) can query symbol metadata, lifetimes, and types in constant time
/// using AST node IDs, rather than constantly performing expensive scope traversal lookups.
pub(crate) struct ResolutionTable {
    symbol_id_by_node_id: HashMap<NodeId, SymbolId>,
}

impl ResolutionTable {
    /// Creates a new, empty `ResolutionTable`.
    pub(crate) fn new() -> Self {
        Self {
            symbol_id_by_node_id: HashMap::new(),
        }
    }

    /// Records the resolved `SymbolId` for a given AST `NodeId`.
    pub(crate) fn resolve(&mut self, node_id: NodeId, symbol_id: SymbolId) {
        self.symbol_id_by_node_id.insert(node_id, symbol_id);
    }

    /// Retrieves the resolved `SymbolId` for a given `NodeId`, if it exists.
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
