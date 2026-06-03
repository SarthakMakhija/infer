use crate::lexer::token::{Token, TokenType};
use std::fmt;

/// Represents errors encountered while parsing tokens into AST expressions.
#[derive(Debug, PartialEq)]
pub(crate) enum ExpressionError {
    /// Failed to parse a numeric literal string slice into a concrete integer value.
    ///
    /// Stores the invalid string that failed parsing and the line number where it occurred.
    ParseIntError(String, usize),

    /// The parser encountered a token type that cannot be converted into or used as an operator.
    ///
    /// Stores the unsupported `TokenType` and the line number where it occurred.
    UnsupportedOperator(TokenType, usize),
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionError::ParseIntError(val, line) => {
                write!(
                    formatter,
                    "failed to parse integer '{}' on line {}",
                    val, line
                )
            }
            ExpressionError::UnsupportedOperator(token_type, line) => {
                write!(
                    formatter,
                    "unsupported operator '{:?}' on line {}",
                    token_type, line
                )
            }
        }
    }
}

impl std::error::Error for ExpressionError {}

/// Represents a parsed expression in the AST.
///
/// Expressions evaluate to values and can appear on the right-hand side
/// of declarations, in conditions, as function arguments, and more.
#[derive(Debug, PartialEq)]
pub enum Expression {
    /// A 32-bit signed integer literal (e.g., `42`).
    I32(i32),
    /// A string literal (e.g., `"hello"`).
    String(String),
    /// A reference to a named variable or function (e.g., `score`).
    Identifier(String),
    /// A boolean literal: `true` or `false`.
    Boolean(bool),
    /// A unary expression applying an operator to a single operand (e.g., `-x`, `!flag`).
    Unary(Box<Expression>, UnaryOperator),
    /// A binary expression applying an operator to a left and right operand (e.g., `a + b`).
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
    /// A parenthesised expression that controls evaluation order (e.g., `(a + b)`).
    Grouped(Box<Expression>),
    /// A function call expression with a callee expression and a list of argument expressions.
    FunctionCall(Box<Expression>, Vec<Expression>),
}

impl Expression {
    /// Recursively unwraps nested `Grouped` expressions, returning the innermost non-grouped expression.
    pub(crate) fn unwrap_grouped(&self) -> &Expression {
        match self {
            Expression::Grouped(inner) => inner.unwrap_grouped(),
            _ => self,
        }
    }

    /// Factory method to construct an `Expression::Identifier`.
    pub fn identifier(name: String) -> Self {
        Expression::Identifier(name)
    }

    /// Factory method to construct an `Expression::FunctionCall`.
    pub fn function_call(callee: Expression, arguments: Vec<Expression>) -> Self {
        Expression::FunctionCall(Box::new(callee), arguments)
    }
}

/// Represents a binary operator used between two expressions.
///
/// Arithmetic operators follow standard mathematical precedence:
/// multiplication and division bind tighter than addition and subtraction.
/// Comparison and logical operators bind at a lower level.
#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    /// The `+` addition operator.
    Plus,
    /// The `-` subtraction operator.
    Minus,
    /// The `*` multiplication operator.
    Multiply,
    /// The `/` division operator.
    Divide,
    /// The `>` comparison operator.
    GreaterThan,
    /// The `<` comparison operator.
    LessThan,
    /// The `>=` comparison operator.
    GreaterThanEquals,
    /// The `<=` comparison operator.
    LessThanEquals,
    /// The `==` equality operator.
    EqualsEquals,
    /// The `!=` inequality operator.
    NotEquals,
    /// The `and` logical conjunction operator.
    And,
    /// The `or` logical disjunction operator.
    Or,
}

impl TryFrom<&Token<'_>> for BinaryOperator {
    type Error = ExpressionError;

    /// Attempts to convert a `Token` reference into a `BinaryOperator`.
    ///
    /// Returns `Err(ExpressionError::UnsupportedOperator)` if the token's type
    /// cannot be mapped to any known binary operator.
    fn try_from(token: &Token<'_>) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Plus => Ok(BinaryOperator::Plus),
            TokenType::Minus => Ok(BinaryOperator::Minus),
            TokenType::Star => Ok(BinaryOperator::Multiply),
            TokenType::Slash => Ok(BinaryOperator::Divide),
            TokenType::GreaterThan => Ok(BinaryOperator::GreaterThan),
            TokenType::LessThan => Ok(BinaryOperator::LessThan),
            TokenType::GreaterThanEquals => Ok(BinaryOperator::GreaterThanEquals),
            TokenType::LessThanEquals => Ok(BinaryOperator::LessThanEquals),
            TokenType::EqualsEquals => Ok(BinaryOperator::EqualsEquals),
            TokenType::BangEquals => Ok(BinaryOperator::NotEquals),
            TokenType::And => Ok(BinaryOperator::And),
            TokenType::Or => Ok(BinaryOperator::Or),
            _ => Err(ExpressionError::UnsupportedOperator(
                token.token_type,
                token.line,
            )),
        }
    }
}

impl BinaryOperator {
    /// Returns `true` if this operator is a relational or equality comparison
    /// (`>`, `>=`, `<`, `<=`, `==`, `!=`).
    ///
    /// Used to detect chained comparisons (e.g., `a < b < c`) which are not supported.
    pub(crate) fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOperator::GreaterThan
                | BinaryOperator::GreaterThanEquals
                | BinaryOperator::LessThan
                | BinaryOperator::LessThanEquals
                | BinaryOperator::EqualsEquals
                | BinaryOperator::NotEquals
        )
    }
}

/// Represents a unary operator applied to a single operand.
#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    /// The unary `-` negation operator (e.g., `-10`).
    Minus,
    /// The logical NOT `!` operator (e.g., `!active`).
    Negation,
}

impl TryFrom<&Token<'_>> for UnaryOperator {
    type Error = ExpressionError;

    /// Attempts to convert a `Token` reference into a `UnaryOperator`.
    ///
    /// Returns `Err(ExpressionError::UnsupportedOperator)` if the token's type
    /// cannot be mapped to a known unary operator.
    fn try_from(token: &Token<'_>) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Minus => Ok(UnaryOperator::Minus),
            TokenType::Bang => Ok(UnaryOperator::Negation),
            _ => Err(ExpressionError::UnsupportedOperator(
                token.token_type,
                token.line,
            )),
        }
    }
}

#[cfg(test)]
mod binary_operator_tests {
    use super::*;

    #[test]
    fn try_from_plus_token_to_plus_operator() {
        let token = Token::new(TokenType::Plus, 0..1, 1, "+");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Plus);
    }

    #[test]
    fn try_from_minus_token_to_minus_operator() {
        let token = Token::new(TokenType::Minus, 0..1, 1, "-");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Minus);
    }

    #[test]
    fn try_from_star_token_to_multiply_operator() {
        let token = Token::new(TokenType::Star, 0..1, 1, "*");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Multiply);
    }

    #[test]
    fn try_from_slash_token_to_divide_operator() {
        let token = Token::new(TokenType::Slash, 0..1, 1, "/");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Divide);
    }

    #[test]
    fn try_from_invalid_token_type_to_unsupported_operator_error() {
        let token = Token::new(TokenType::Identifier, 0..4, 2, "name");
        let result = BinaryOperator::try_from(&token);
        assert_eq!(
            result.err().unwrap(),
            ExpressionError::UnsupportedOperator(TokenType::Identifier, 2)
        );
    }

    #[test]
    fn try_from_greater_than_token_to_greater_than_operator() {
        let token = Token::new(TokenType::GreaterThan, 0..1, 1, ">");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::GreaterThan);
    }

    #[test]
    fn try_from_less_than_token_to_less_than_operator() {
        let token = Token::new(TokenType::LessThan, 0..1, 1, "<");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::LessThan);
    }

    #[test]
    fn try_from_greater_than_equals_token_to_greater_than_equals_operator() {
        let token = Token::new(TokenType::GreaterThanEquals, 0..2, 1, ">=");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::GreaterThanEquals);
    }

    #[test]
    fn try_from_less_than_equals_token_to_less_than_equals_operator() {
        let token = Token::new(TokenType::LessThanEquals, 0..2, 1, "<=");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::LessThanEquals);
    }

    #[test]
    fn try_from_equals_equals_token_to_equals_equals_operator() {
        let token = Token::new(TokenType::EqualsEquals, 0..2, 1, "==");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::EqualsEquals);
    }

    #[test]
    fn try_from_bang_equals_token_to_not_equals_operator() {
        let token = Token::new(TokenType::BangEquals, 0..2, 1, "!=");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::NotEquals);
    }

    #[test]
    fn try_from_and_token_to_and_operator() {
        let token = Token::new(TokenType::And, 0..3, 1, "and");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::And);
    }

    #[test]
    fn try_from_or_token_to_or_operator() {
        let token = Token::new(TokenType::Or, 0..2, 1, "or");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Or);
    }

    #[test]
    fn greater_than_is_comparison() {
        assert!(BinaryOperator::GreaterThan.is_comparison());
    }

    #[test]
    fn greater_than_equals_is_comparison() {
        assert!(BinaryOperator::GreaterThanEquals.is_comparison());
    }

    #[test]
    fn less_than_is_comparison() {
        assert!(BinaryOperator::LessThan.is_comparison());
    }

    #[test]
    fn less_than_equals_is_comparison() {
        assert!(BinaryOperator::LessThanEquals.is_comparison());
    }

    #[test]
    fn equals_equals_is_comparison() {
        assert!(BinaryOperator::EqualsEquals.is_comparison());
    }

    #[test]
    fn not_equals_is_comparison() {
        assert!(BinaryOperator::NotEquals.is_comparison());
    }

    #[test]
    fn plus_is_not_comparison() {
        assert!(!BinaryOperator::Plus.is_comparison());
    }
}

#[cfg(test)]
mod expression_tests {
    use super::*;

    #[test]
    fn unwrap_grouped_returns_inner_expression() {
        let inner = Expression::I32(42);
        let grouped = Expression::Grouped(Box::new(inner));
        assert_eq!(grouped.unwrap_grouped(), &Expression::I32(42));
    }

    #[test]
    fn unwrap_grouped_returns_self_for_non_grouped() {
        let expr = Expression::I32(42);
        assert_eq!(expr.unwrap_grouped(), &Expression::I32(42));
    }

    #[test]
    fn unwrap_grouped_unwraps_multiple_levels() {
        let inner = Expression::I32(42);
        let nested = Expression::Grouped(Box::new(Expression::Grouped(Box::new(inner))));
        assert_eq!(nested.unwrap_grouped(), &Expression::I32(42));
    }
}

#[cfg(test)]
mod unary_operator_tests {
    use super::*;

    #[test]
    fn try_from_minus_token_to_minus_operator() {
        let token = Token::new(TokenType::Minus, 0..1, 1, "-");
        let operator = UnaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, UnaryOperator::Minus);
    }

    #[test]
    fn try_from_bang_token_to_negation_operator() {
        let token = Token::new(TokenType::Bang, 0..1, 1, "!");
        let operator = UnaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, UnaryOperator::Negation);
    }

    #[test]
    fn try_from_invalid_token_type_to_unsupported_operator_error() {
        let token = Token::new(TokenType::Identifier, 0..4, 2, "name");
        let result = UnaryOperator::try_from(&token);
        assert_eq!(
            result.err().unwrap(),
            ExpressionError::UnsupportedOperator(TokenType::Identifier, 2)
        );
    }
}
