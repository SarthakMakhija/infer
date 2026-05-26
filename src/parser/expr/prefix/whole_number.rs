use crate::ast::expr::{Expression, ExpressionError};
use crate::lexer::token::Token;
use crate::parser::error::ParseError;
use crate::parser::expr::PrefixParser;
use std::str::FromStr;

pub(crate) struct WholeNumberParser;

impl<'src> PrefixParser<'src> for WholeNumberParser {
    fn parse(&mut self, token: &Token<'src>) -> Result<Expression, ParseError> {
        let value = i32::from_str(token.value()).map_err(|_| {
            ParseError::ExpressionError(ExpressionError::ParseIntError(
                token.value().to_string(),
                token.line,
            ))
        })?;
        Ok(Expression::I32(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::TokenType;

    #[test]
    fn parse_valid_whole_number() {
        let token = Token::new(TokenType::WholeNumber, 0..3, 1, "123");
        let mut parser = WholeNumberParser;

        let expression = parser.parse(&token).unwrap();
        assert_eq!(expression, Expression::I32(123));
    }

    #[test]
    fn parse_whole_number_overflow() {
        let token = Token::new(TokenType::WholeNumber, 0..12, 1, "999999999999");
        let mut parser = WholeNumberParser;

        let result = parser.parse(&token);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ParseError::ExpressionError(ExpressionError::ParseIntError(
                "999999999999".to_string(),
                1
            ))
        );
    }
}
