use crate::lexer::token::{Token, TokenType};
use crate::parser::error::{ExpressionError, ParseError};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    I32(i32),
    String(String),
    Identifier(String),
}

impl<'a> TryFrom<Token<'a>> for Expression {
    type Error = ParseError;

    fn try_from(token: Token<'a>) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Identifier => Ok(Expression::Identifier(token.value().to_string())),
            TokenType::WholeNumber => {
                let value = i32::from_str(token.value()).map_err(|_| {
                    ParseError::ExpressionError(ExpressionError::ParseIntError(
                        token.value().to_string(),
                    ))
                })?;
                Ok(Expression::I32(value))
            }
            TokenType::StringLiteral => Ok(Expression::String(token.value().to_string())),
            other => Err(ParseError::ExpressionError(
                ExpressionError::UnsupportedTokenType(other),
            )),
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
        let expr = Expression::try_from(token).unwrap();
        assert_eq!(expr, Expression::Identifier("name".to_string()));
    }

    #[test]
    fn try_from_whole_number() {
        let token = Token::new(TokenType::WholeNumber, 0..3, 1, "456");
        let expr = Expression::try_from(token).unwrap();
        assert_eq!(expr, Expression::I32(456));
    }

    #[test]
    fn try_from_invalid_whole_number() {
        let token = Token::new(TokenType::WholeNumber, 0..12, 1, "999999999999");
        let expr = Expression::try_from(token);
        assert_eq!(
            expr.err().unwrap(),
            ParseError::ExpressionError(ExpressionError::ParseIntError("999999999999".to_string()))
        );
    }

    #[test]
    fn try_from_string_literal() {
        let token = Token::new(TokenType::StringLiteral, 1..6, 1, "\"infer\"");
        let expr = Expression::try_from(token).unwrap();
        assert_eq!(expr, Expression::String("infer".to_string()));
    }

    #[test]
    fn try_from_invalid_token_type() {
        let token = Token::new(TokenType::Var, 0..3, 1, "var");
        let expr = Expression::try_from(token);
        assert_eq!(
            expr.err().unwrap(),
            ParseError::ExpressionError(ExpressionError::UnsupportedTokenType(TokenType::Var))
        );
    }
}
