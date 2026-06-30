use crate::ast::expr::ExpressionKind;
use crate::lexer::token::Token;
use crate::parser::error::ParseError;
use crate::parser::expr::PrefixParser;

/// A prefix parser that converts an `Identifier` token into an [`ExpressionKind::Identifier`].
pub(crate) struct IdentifierParser;

impl<'src> PrefixParser<'src> for IdentifierParser {
    fn parse(&mut self, token: &Token<'src>) -> Result<ExpressionKind, ParseError> {
        Ok(ExpressionKind::identifier(token.value().to_owned()))
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
        assert_eq!(
            expression,
            ExpressionKind::identifier("first_name".to_string())
        );
    }
}
