use crate::ast::expr::BinaryOperator;
use crate::semantic::error::SemanticError;
use std::cell::Cell;
use std::convert::TryFrom;

pub(crate) mod constraints;
pub(crate) mod type_inference;
pub(crate) mod type_table;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    Int32,
    Bool,
    String,
    Placeholder(usize), // Unique type variable placeholder (e.g. 0, 1)
}

impl TryFrom<&str> for Type {
    type Error = SemanticError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "i32" => Ok(Type::Int32),
            "bool" => Ok(Type::Bool),
            "string" => Ok(Type::String),
            _ => Err(SemanticError::UnsupportedTypeError(value.to_string())),
        }
    }
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

pub(crate) struct BinaryOperatorSignature {
    pub(crate) left: Type,
    pub(crate) right: Type,
    pub(crate) result: Type,
}

impl BinaryOperatorSignature {
    pub(crate) fn of(operator: &BinaryOperator) -> Self {
        let (left, right, result) = match operator {
            BinaryOperator::Plus
            | BinaryOperator::Minus
            | BinaryOperator::Multiply
            | BinaryOperator::Divide => (Type::Int32, Type::Int32, Type::Int32),

            BinaryOperator::GreaterThan
            | BinaryOperator::LessThan
            | BinaryOperator::GreaterThanEquals
            | BinaryOperator::LessThanEquals
            | BinaryOperator::EqualsEquals
            | BinaryOperator::NotEquals => (Type::Int32, Type::Int32, Type::Bool),

            BinaryOperator::And | BinaryOperator::Or => (Type::Bool, Type::Bool, Type::Bool),
        };
        Self {
            left,
            right,
            result,
        }
    }
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

#[cfg(test)]
mod parse_type_tests {
    use super::*;

    #[test]
    fn parse_i32_type() {
        let parsed = Type::try_from("i32");
        assert_eq!(parsed, Ok(Type::Int32));
    }

    #[test]
    fn parse_bool_type() {
        let parsed = Type::try_from("bool");
        assert_eq!(parsed, Ok(Type::Bool));
    }

    #[test]
    fn parse_string_type() {
        let parsed = Type::try_from("string");
        assert_eq!(parsed, Ok(Type::String));
    }

    #[test]
    fn parse_invalid_type_returns_unsupported_error() {
        let parsed = Type::try_from("i64");
        assert_eq!(
            parsed,
            Err(SemanticError::UnsupportedTypeError("i64".to_string()))
        );
    }
}

#[cfg(test)]
mod operator_signature_tests {
    use super::*;

    #[test]
    fn plus_operator_returns_int32_left_operand_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::Plus);
        assert_eq!(signature.left, Type::Int32);
    }

    #[test]
    fn plus_operator_returns_int32_right_operand_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::Plus);
        assert_eq!(signature.right, Type::Int32);
    }

    #[test]
    fn plus_operator_returns_int32_result_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::Plus);
        assert_eq!(signature.result, Type::Int32);
    }

    #[test]
    fn greater_than_operator_returns_int32_left_operand_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::GreaterThan);
        assert_eq!(signature.left, Type::Int32);
    }

    #[test]
    fn greater_than_operator_returns_int32_right_operand_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::GreaterThan);
        assert_eq!(signature.right, Type::Int32);
    }

    #[test]
    fn greater_than_operator_returns_bool_result_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::GreaterThan);
        assert_eq!(signature.result, Type::Bool);
    }

    #[test]
    fn and_operator_returns_bool_left_operand_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::And);
        assert_eq!(signature.left, Type::Bool);
    }

    #[test]
    fn and_operator_returns_bool_right_operand_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::And);
        assert_eq!(signature.right, Type::Bool);
    }

    #[test]
    fn and_operator_returns_bool_result_type() {
        let signature = BinaryOperatorSignature::of(&BinaryOperator::And);
        assert_eq!(signature.result, Type::Bool);
    }
}
