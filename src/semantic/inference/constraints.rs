use crate::semantic::inference::Type;

/// A collection of type equations (constraints) collected during type inference.
#[derive(Debug, PartialEq)]
pub(crate) struct Constraints {
    pub(crate) entries: Vec<Constraint>,
}

/// Represents a single type constraint asserting that two types must be equal (left == right).
#[derive(Debug, PartialEq)]
pub(crate) struct Constraint {
    pub(crate) left: Type,
    pub(crate) right: Type,
}

impl Constraint {
    /// Creates a new type equality constraint (left == right).
    pub(crate) fn new(left: Type, right: Type) -> Self {
        Self { left, right }
    }
}

impl Constraints {
    /// Creates a new, empty set of constraints.
    pub(crate) fn new() -> Self {
        Self { entries: vec![] }
    }

    /// Adds a type constraint to the collection.
    pub(crate) fn add(&mut self, constraint: Constraint) {
        self.entries.push(constraint);
    }

    #[cfg(test)]
    pub(crate) fn entry_at(&self, index: usize) -> &Constraint {
        &self.entries[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_constraints() {
        let constraints = Constraints::new();
        assert_eq!(constraints.entries.len(), 0);
    }

    #[test]
    fn add_constraint() {
        let mut constraints = Constraints::new();
        constraints.add(Constraint::new(Type::Int32, Type::Int32));

        assert_eq!(constraints.entries.len(), 1);
    }

    #[test]
    fn added_constraint_retains_left_type() {
        let mut constraints = Constraints::new();
        constraints.add(Constraint::new(Type::Int32, Type::Bool));

        assert_eq!(constraints.entries[0].left, Type::Int32);
    }

    #[test]
    fn added_constraint_retains_right_type() {
        let mut constraints = Constraints::new();
        constraints.add(Constraint::new(Type::Int32, Type::Bool));

        assert_eq!(constraints.entries[0].right, Type::Bool);
    }
}
