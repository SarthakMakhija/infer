use crate::ast::expr::Expression;
use crate::lexer::token::Token;
use crate::parser::error::ParseError;
use crate::parser::expr::PrefixParser;

/// A prefix parser that converts an `Identifier` token into an [`Expression::Identifier`].
pub(crate) struct IdentifierParser;

impl<'src> PrefixParser<'src> for IdentifierParser {
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
        let mut identifier = IdentifierParser;

        let expression = identifier.parse(&token).unwrap();
        assert_eq!(expression, Expression::Identifier("first_name".to_string()));
    }
}
