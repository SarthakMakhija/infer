use crate::ast::expr::ExpressionKind;
use crate::lexer::token::{Token, TokenType};
use crate::parser::error::ParseError;
use crate::parser::expr::PrefixParser;

/// A prefix parser that converts a `BooleanLiteral` token into an [`ExpressionKind::Boolean`].
pub(crate) struct BooleanParser;

impl<'src> PrefixParser<'src> for BooleanParser {
    fn parse(&mut self, token: &Token<'src>) -> Result<ExpressionKind, ParseError> {
        if let TokenType::BooleanLiteral(val) = token.token_type {
            Ok(ExpressionKind::Boolean(val))
        } else {
            unreachable!("Boolean parser only handles BooleanLiteral tokens")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_with_true() {
        let token = Token::new(TokenType::BooleanLiteral(true), 0..4, 1, "true");
        let mut parser = BooleanParser;

        let expression = parser.parse(&token).unwrap();
        assert_eq!(expression, ExpressionKind::Boolean(true));
    }

    #[test]
    fn parse_boolean_with_false() {
        let token = Token::new(TokenType::BooleanLiteral(false), 0..5, 1, "false");
        let mut parser = BooleanParser;

        let expression = parser.parse(&token).unwrap();
        assert_eq!(expression, ExpressionKind::Boolean(false));
    }
}
