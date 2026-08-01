//! Type inference module.
//!
//! Implements monomorphic constraint-based type inference (Pass 2 of semantic analysis).
//! It walks the AST to generate type equations (constraints) and solves them
//! using unification to determine concrete types for all expressions.

use crate::ast::expr::{BinaryOperator, UnaryOperator};
use crate::semantic::error::SemanticError;
use std::cell::Cell;
use std::convert::TryFrom;

pub(crate) mod constraints;
pub(crate) mod type_inference;
pub(crate) mod type_table;

/// Represents a semantic type in the target language.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    /// 32-bit signed integer.
    Int32,
    /// Boolean flag (`true` or `false`).
    Bool,
    /// UTF-8 string literal.
    String,
    /// A unique type placeholder/variable used during inference.
    Placeholder(usize),
}

impl TryFrom<&str> for Type {
    type Error = SemanticError;

    /// Attempts to parse a raw string slice representation of a type (e.g. `"i32"`, `"bool"`, `"string"`)
    /// into a corresponding concrete `Type`.
    ///
    /// # Errors
    /// Returns `SemanticError::UnsupportedTypeError` if the type string is unrecognized.
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
    /// Extracts the inner unique identifier of a `Type::Placeholder`.
    ///
    /// # Panics
    /// Panics if called on a non-placeholder variant.
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

/// Represents the expected input and output types for a binary operator.
pub(crate) struct BinaryOperatorSignature {
    pub(crate) left: Type,
    pub(crate) right: Type,
    pub(crate) result: Type,
}

impl BinaryOperatorSignature {
    /// Returns the type signature of the given binary operator.
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

/// Represents the expected input and output types for a unary operator.
pub(crate) struct UnaryOperatorSignature {
    pub(crate) operand: Type,
    pub(crate) result: Type,
}

impl UnaryOperatorSignature {
    /// Returns the type signature of the given unary operator.
    pub(crate) fn of(operator: &UnaryOperator) -> Self {
        let (operand, result) = match operator {
            UnaryOperator::Minus => (Type::Int32, Type::Int32),
            UnaryOperator::Negation => (Type::Bool, Type::Bool),
        };
        Self { operand, result }
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

#[cfg(test)]
mod unary_operator_signature_tests {
    use super::*;

    #[test]
    fn minus_operator_returns_int32_operand_type() {
        let signature = UnaryOperatorSignature::of(&UnaryOperator::Minus);
        assert_eq!(signature.operand, Type::Int32);
    }

    #[test]
    fn minus_operator_returns_int32_result_type() {
        let signature = UnaryOperatorSignature::of(&UnaryOperator::Minus);
        assert_eq!(signature.result, Type::Int32);
    }

    #[test]
    fn negation_operator_returns_bool_operand_type() {
        let signature = UnaryOperatorSignature::of(&UnaryOperator::Negation);
        assert_eq!(signature.operand, Type::Bool);
    }

    #[test]
    fn negation_operator_returns_bool_result_type() {
        let signature = UnaryOperatorSignature::of(&UnaryOperator::Negation);
        assert_eq!(signature.result, Type::Bool);
    }
}
