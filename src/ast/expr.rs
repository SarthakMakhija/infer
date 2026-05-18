use crate::lexer::token::{Token, TokenType};
use std::str::FromStr;

/// Represents errors encountered while parsing tokens into AST expressions.
#[derive(Debug, PartialEq)]
pub(crate) enum ExpressionError {
    /// The parser encountered a token type that cannot be converted into or used as an expression.
    UnsupportedTokenType(TokenType),

    /// Failed to parse a numeric literal string slice into a concrete integer value.
    ///
    /// Stores the invalid string that failed parsing.
    ParseIntError(String),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    I32(i32),
    String(String),
    Identifier(String),
    Boolean(bool),
}

impl<'a> TryFrom<Token<'a>> for Expression {
    type Error = ExpressionError;

    fn try_from(token: Token<'a>) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Identifier => Ok(Expression::Identifier(token.value().to_string())),
            TokenType::WholeNumber => {
                let value = i32::from_str(token.value())
                    .map_err(|_| ExpressionError::ParseIntError(token.value().to_string()))?;
                Ok(Expression::I32(value))
            }
            TokenType::StringLiteral => Ok(Expression::String(token.string_value().to_string())),
            TokenType::True => Ok(Expression::Boolean(true)),
            TokenType::False => Ok(Expression::Boolean(false)),
            other => Err(ExpressionError::UnsupportedTokenType(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::Token;

    #[test]
    fn try_from_identifier() {
        let token = Token::new(TokenType::Identifier, 0..4, 1, "name");
        let expression = Expression::try_from(token).unwrap();
        assert_eq!(expression, Expression::Identifier("name".to_string()));
    }

    #[test]
    fn try_from_whole_number() {
        let token = Token::new(TokenType::WholeNumber, 0..3, 1, "456");
        let expression = Expression::try_from(token).unwrap();
        assert_eq!(expression, Expression::I32(456));
    }

    #[test]
    fn try_from_invalid_whole_number() {
        let token = Token::new(TokenType::WholeNumber, 0..12, 1, "999999999999");
        let expression = Expression::try_from(token);
        assert_eq!(
            expression.err().unwrap(),
            ExpressionError::ParseIntError("999999999999".to_string())
        );
    }

    #[test]
    fn try_from_string_literal() {
        let token = Token::new(TokenType::StringLiteral, 1..6, 1, "\"infer\"");
        let expression = Expression::try_from(token).unwrap();
        assert_eq!(expression, Expression::String("infer".to_string()));
    }

    #[test]
    fn try_from_invalid_token_type() {
        let token = Token::new(TokenType::Var, 0..3, 1, "var");
        let expression = Expression::try_from(token);
        assert_eq!(
            expression.err().unwrap(),
            ExpressionError::UnsupportedTokenType(TokenType::Var)
        );
    }

    #[test]
    fn try_from_boolean_true() {
        let token = Token::new(TokenType::True, 0..4, 1, "true");
        let expression = Expression::try_from(token).unwrap();
        assert_eq!(expression, Expression::Boolean(true));
    }

    #[test]
    fn try_from_boolean_false() {
        let token = Token::new(TokenType::False, 0..5, 1, "false");
        let expression = Expression::try_from(token).unwrap();
        assert_eq!(expression, Expression::Boolean(false));
    }
}
