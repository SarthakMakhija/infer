//! Unification constraint solver implementation.
//!
//! # End-to-End Type Inference Flow Example
//!
//! To understand how the Symbol Resolution, Type Inference, and Unification Solver
//! components interact, consider the following program:
//!
//! ```text
//! var x;
//! var y = x;
//! x = 10;
//! ```
//!
//! ## Phase 1: Symbol Resolution (Pass 1)
//!
//! The AST nodes are traversed from top to bottom. Declarations introduce new unique
//! `SymbolId`s, and reference nodes are resolved to point back to their respective declarations.
//!
//! 1. **`var x;`** (Declaration NodeId(1)):
//!    * Declares variable `x`. A new unique symbol `SymbolId(10)` is generated.
//!    * Maps: `NodeId(1) -> SymbolId(10)`.
//! 2. **`var y = x;`** (Declaration NodeId(2), Reference NodeId(3)):
//!    * Declares variable `y`. A new unique symbol `SymbolId(20)` is generated.
//!    * Maps: `NodeId(2) -> SymbolId(20)`.
//!    * The initializer references the identifier `x` (`NodeId(3)`). This resolves back to the
//!      already declared `SymbolId(10)`.
//!    * Maps: `NodeId(3) -> SymbolId(10)`.
//! 3. **`x = 10;`** (Reference NodeId(4), Literal NodeId(5)):
//!    * The assignment target references identifier `x` (`NodeId(4)`). This resolves back to
//!      `SymbolId(10)`.
//!    * Maps: `NodeId(4) -> SymbolId(10)`.
//!
//! **Final Resolution Table:**
//! ```text
//! NodeId(1) (declaration x)           -> SymbolId(10)
//! NodeId(2) (declaration y)           -> SymbolId(20)
//! NodeId(3) (reference x in y = x)    -> SymbolId(10)
//! NodeId(4) (reference x in x = 10)   -> SymbolId(10)
//! ```
//!
//! ## Phase 2: Type Inference & Constraint Generation (Pass 2)
//!
//! The `TypeInferenceVisitor` traverses the AST, generates fresh placeholder type variables
//! for unannotated declarations, and collects equality equations (constraints).
//!
//! 1. **`var x;`**:
//!    * Unannotated declaration. Generates a fresh placeholder type variable: `Type::Placeholder(1)`.
//!    * Records `SymbolId(10) -> Type::Placeholder(1)` in the `TypeTable`.
//! 2. **`var y = x;`**:
//!    * Unannotated declaration. Generates a fresh placeholder type variable: `Type::Placeholder(2)`.
//!    * Records `SymbolId(20) -> Type::Placeholder(2)` in the `TypeTable`.
//!    * Infers the initializer expression `x` (`NodeId(3)`): looks up `SymbolId(10)` in the `TypeTable`
//!      and retrieves `Type::Placeholder(1)`.
//!    * Generates an equality constraint equating `y`'s type to `x`'s type:
//!      `Type::Placeholder(2) == Type::Placeholder(1)`.
//! 3. **`x = 10;`**:
//!    * Infers target `x` (`NodeId(4)`): looks up `SymbolId(10)` in `TypeTable` -> `Type::Placeholder(1)`.
//!    * Infers source literal `10` (`NodeId(5)`): returns concrete type `Type::Int32`.
//!    * Generates an equality constraint equating `x`'s type to the literal's type:
//!      `Type::Placeholder(1) == Type::Int32`.
//!
//! **Final Type Table (Pre-Solving):**
//! ```text
//! SymbolId(10) (variable x) -> Type::Placeholder(1)
//! SymbolId(20) (variable y) -> Type::Placeholder(2)
//! ```
//!
//! **Final Constraints:**
//! 1. `Type::Placeholder(2) == Type::Placeholder(1)` (from declaration `var y = x`)
//! 2. `Type::Placeholder(1) == Type::Int32`        (from assignment `x = 10`)
//!
//! ## Phase 3: Constraint Solving & Unification
//!
//! The `Unifier` processes constraints one by one to build a substitution mapping,
//! followed by a flattening pass to perform path compression.
//!
//! 1. **Process Constraint 1** (`Type::Placeholder(2) == Type::Placeholder(1)`):
//!    * Resolves `Placeholder(2)` to `Placeholder(2)`.
//!    * Resolves `Placeholder(1)` to `Placeholder(1)`.
//!    * Since both are placeholders, binds the left variable to the right: `2 -> Type::Placeholder(1)`.
//!    * Substitutions: `{ 2: Type::Placeholder(1) }`.
//! 2. **Process Constraint 2** (`Type::Placeholder(1) == Type::Int32`):
//!    * Resolves `Placeholder(1)` to `Placeholder(1)`.
//!    * Resolves `Int32` to `Int32`.
//!    * Binds the placeholder variable to the concrete type: `1 -> Type::Int32`.
//!    * Substitutions: `{ 2: Type::Placeholder(1), 1: Type::Int32 }`.
//! 3. **Flattening Pass (Path Compression)**:
//!    * For Key `2`: `resolve(Placeholder(2))` recursive lookup resolves to `Type::Int32`. Updates mapping to `2 -> Type::Int32`.
//!    * For Key `1`: `resolve(Placeholder(1))` lookup resolves to `Type::Int32`. Updates mapping to `1 -> Type::Int32`.
//!
//! **Final Solved Substitutions Map:**
//! ```text
//! {
//!   1: Type::Int32,
//!   2: Type::Int32
//! }
//! ```

use crate::semantic::error::SemanticError;
use crate::semantic::inference::constraints::Constraints;
use crate::semantic::inference::Type;
use std::collections::HashMap;

/// A unification constraint solver that resolves type placeholders to concrete types.
pub(crate) struct Unifier {
    pub(crate) substitutions: HashMap<usize, Type>,
}

impl Unifier {
    /// Creates a new, empty unification solver.
    pub(crate) fn new() -> Self {
        Self {
            substitutions: HashMap::new(),
        }
    }

    /// Recursively resolves type placeholders to their final bound types.
    pub(crate) fn resolve(&self, type_node: Type) -> Type {
        match type_node {
            Type::Placeholder(placeholder_id) => {
                if let Some(next_type) = self.substitutions.get(&placeholder_id) {
                    self.resolve(*next_type)
                } else {
                    type_node
                }
            }
            _ => type_node,
        }
    }

    /// Solves all type constraints in the collection, updating substitutions and flattening them.
    pub(crate) fn solve(
        &mut self,
        constraints: Constraints,
    ) -> Result<HashMap<usize, Type>, SemanticError> {
        for constraint in constraints.entries {
            self.unify(constraint.left, constraint.right)?;
        }

        // Flatten all substitutions so that every placeholder maps directly to its resolved concrete type
        let placeholder_ids: Vec<usize> = self.substitutions.keys().copied().collect();
        for placeholder_id in placeholder_ids {
            let concrete_type = self.resolve(Type::Placeholder(placeholder_id));
            self.substitutions.insert(placeholder_id, concrete_type);
        }
        Ok(self.substitutions.clone())
    }

    /// Unifies two types, updating the substitutions map or returning a mismatch error.
    fn unify(&mut self, left_type: Type, right_type: Type) -> Result<(), SemanticError> {
        let left_type = self.resolve(left_type);
        let right_type = self.resolve(right_type);

        if left_type == right_type {
            return Ok(());
        }

        if let Type::Placeholder(placeholder_id) = left_type {
            self.substitutions.insert(placeholder_id, right_type);
            return Ok(());
        }

        if let Type::Placeholder(placeholder_id) = right_type {
            self.substitutions.insert(placeholder_id, left_type);
            return Ok(());
        }

        Err(SemanticError::TypeMismatch(left_type, right_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::inference::constraints::Constraint;

    #[test]
    fn unify_concrete_types() {
        let mut unifier = Unifier::new();
        let mut constraints = Constraints::new();
        constraints.add(Constraint::new(Type::Int32, Type::Int32));

        let result = unifier.solve(constraints);
        assert!(result.is_ok());
        assert!(unifier.substitutions.is_empty());
    }

    #[test]
    fn unify_concrete_types_mismatch() {
        let mut unifier = Unifier::new();
        let mut constraints = Constraints::new();
        constraints.add(Constraint::new(Type::Int32, Type::Bool));

        let result = unifier.solve(constraints);
        assert_eq!(
            result,
            Err(SemanticError::TypeMismatch(Type::Int32, Type::Bool))
        );
    }

    #[test]
    fn unify_placeholder_to_concrete_type() {
        let mut unifier = Unifier::new();
        let mut constraints = Constraints::new();
        constraints.add(Constraint::new(Type::Placeholder(1), Type::Int32));

        let result = unifier.solve(constraints);
        assert!(result.is_ok());
        assert_eq!(unifier.resolve(Type::Placeholder(1)), Type::Int32);
    }

    #[test]
    fn unify_placeholders_transitive() {
        let mut unifier = Unifier::new();
        let mut constraints = Constraints::new();
        constraints.add(Constraint::new(Type::Placeholder(1), Type::Placeholder(2)));
        constraints.add(Constraint::new(Type::Placeholder(2), Type::Int32));

        let result = unifier.solve(constraints);
        assert!(result.is_ok());

        let substitutions = result.unwrap();
        assert_eq!(*substitutions.get(&1).unwrap(), Type::Int32);
        assert_eq!(*substitutions.get(&2).unwrap(), Type::Int32);
    }

    #[test]
    fn unify_conflicting_placeholder_bindings_fails() {
        let mut unifier = Unifier::new();
        let mut constraints = Constraints::new();
        constraints.add(Constraint::new(Type::Placeholder(1), Type::Int32));
        constraints.add(Constraint::new(Type::Placeholder(1), Type::Bool));

        let result = unifier.solve(constraints);
        assert_eq!(
            result,
            Err(SemanticError::TypeMismatch(Type::Int32, Type::Bool))
        );
    }
}
