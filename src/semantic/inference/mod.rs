use std::cell::Cell;

pub(crate) mod constraints;
pub(crate) mod type_table;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    Int32,
    Bool,
    String,
    Placeholder(usize), // Unique type variable placeholder (e.g. 0, 1)
}

impl Type {
    fn placeholder_type_id(&self) -> usize {
        match self {
            Type::Placeholder(type_id) => *type_id,
            _ => panic!("must be a placeholder type"),
        }
    }
}

thread_local! {
    static ID: Cell<Type> = const { Cell::new(Type::Placeholder(0)) };
}

/// Generates a new, thread-safe, globally unique `Type::Placeholder`.
pub(crate) fn next_type_id() -> Type {
    ID.with(|id| {
        let current = id.get();
        let next = Type::Placeholder(current.placeholder_type_id() + 1);
        id.set(next);
        next
    })
}

#[cfg(test)]
mod type_tests {
    use crate::semantic::inference::Type;

    #[test]
    fn placeholder_type_id_extracts_id_from_placeholder() {
        let placeholder = Type::Placeholder(42);
        assert_eq!(placeholder.placeholder_type_id(), 42);
    }

    #[test]
    #[should_panic(expected = "must be a placeholder type")]
    fn placeholder_type_id_panics_on_non_placeholder() {
        Type::Int32.placeholder_type_id();
    }
}

#[cfg(test)]
mod next_type_id_tests {
    use super::*;

    #[test]
    fn next_type_id_generates_consecutive_ids() {
        let first = next_type_id();
        let second = next_type_id();

        let id1 = first.placeholder_type_id();
        let id2 = second.placeholder_type_id();

        assert_eq!(id2, id1 + 1);
    }
}
