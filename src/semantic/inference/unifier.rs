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
