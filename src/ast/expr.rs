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

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    I32(i32),
    String(String),
    Identifier(String),
    Boolean(bool),
    BinaryExpression(Box<Expression>, Operator, Box<Expression>),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl TryFrom<&Token<'_>> for Operator {
    type Error = ExpressionError;

    fn try_from(token: &Token<'_>) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Plus => Ok(Operator::Plus),
            TokenType::Minus => Ok(Operator::Minus),
            TokenType::Star => Ok(Operator::Multiply),
            TokenType::Slash => Ok(Operator::Divide),
            _ => Err(ExpressionError::UnsupportedOperator(
                token.token_type,
                token.line,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_plus_token_to_plus_operator() {
        let token = Token::new(TokenType::Plus, 0..1, 1, "+");
        let operator = Operator::try_from(&token).unwrap();
        assert_eq!(operator, Operator::Plus);
    }

    #[test]
    fn try_from_minus_token_to_minus_operator() {
        let token = Token::new(TokenType::Minus, 0..1, 1, "-");
        let operator = Operator::try_from(&token).unwrap();
        assert_eq!(operator, Operator::Minus);
    }

    #[test]
    fn try_from_star_token_to_multiply_operator() {
        let token = Token::new(TokenType::Star, 0..1, 1, "*");
        let operator = Operator::try_from(&token).unwrap();
        assert_eq!(operator, Operator::Multiply);
    }

    #[test]
    fn try_from_slash_token_to_divide_operator() {
        let token = Token::new(TokenType::Slash, 0..1, 1, "/");
        let operator = Operator::try_from(&token).unwrap();
        assert_eq!(operator, Operator::Divide);
    }

    #[test]
    fn try_from_invalid_token_type_to_unsupported_operator_error() {
        let token = Token::new(TokenType::Identifier, 0..4, 2, "name");
        let result = Operator::try_from(&token);
        assert_eq!(
            result.err().unwrap(),
            ExpressionError::UnsupportedOperator(TokenType::Identifier, 2)
        );
    }
}
