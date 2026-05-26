use crate::ast::expr::Expression;
use crate::lexer::token::{Token, TokenType};
use crate::parser::error::ParseError;
use crate::parser::expr::PrefixRule;

pub(crate) struct Boolean;

impl<'src> PrefixRule<'src> for Boolean {
    fn parse(&mut self, token: &'src Token) -> Result<Expression, ParseError> {
        if let TokenType::BooleanLiteral(val) = token.token_type {
            Ok(Expression::Boolean(val))
        } else {
            unreachable!("Boolean prefix rule only handles BooleanLiteral tokens")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_boolean_with_true() {
        let token = Token::new(TokenType::BooleanLiteral(true), 0..4, 1, "true");
        let mut rule = Boolean;

        let expression = rule.parse(&token).unwrap();
        assert_eq!(expression, Expression::Boolean(true));
    }

    #[test]
    fn parse_boolean_with_false() {
        let token = Token::new(TokenType::BooleanLiteral(false), 0..5, 1, "false");
        let mut rule = Boolean;

        let expression = rule.parse(&token).unwrap();
        assert_eq!(expression, Expression::Boolean(false));
    }
}
