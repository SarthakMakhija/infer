use crate::ast::expr::Expression;
use crate::lexer::token::Token;
use crate::parser::error::ParseError;
use crate::parser::expr::PrefixRule;

pub(crate) struct Identifier;

impl<'src> PrefixRule<'src> for Identifier {
    fn parse(&mut self, token: &Token<'src>) -> Result<Expression, ParseError> {
        Ok(Expression::Identifier(token.value().to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::TokenType;

    #[test]
    fn parse_identifier() {
        let token = Token::new(TokenType::Identifier, 0..10, 1, "first_name");
        let mut identifier = Identifier;

        let expression = identifier.parse(&token).unwrap();
        assert_eq!(expression, Expression::Identifier("first_name".to_string()));
    }
}
