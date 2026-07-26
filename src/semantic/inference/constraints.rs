use crate::semantic::inference::Type;

#[derive(Debug, PartialEq)]
pub(crate) struct Constraints {
    entries: Vec<Constraint>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Constraint {
    left: Type,
    right: Type,
}

impl Constraint {
    pub(crate) fn new(left: Type, right: Type) -> Self {
        Self { left, right }
    }
}

impl Constraints {
    pub(crate) fn new() -> Self {
        Self { entries: vec![] }
    }

    pub(crate) fn add(&mut self, constraint: Constraint) {
        self.entries.push(constraint);
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
