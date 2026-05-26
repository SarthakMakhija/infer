use crate::ast::expr::Expression;
use crate::lexer::token::Token;
use crate::parser::error::ParseError;
use crate::parser::expr::PrefixParser;

pub(crate) struct StringParser;

impl<'src> PrefixParser<'src> for StringParser {
    fn parse(&mut self, token: &Token<'src>) -> Result<Expression, ParseError> {
        Ok(Expression::String(token.string_value().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::TokenType;

    #[test]
    fn parse_string_literal() {
        let token = Token::new(TokenType::StringLiteral, 0..7, 1, "\"infer\"");
        let mut parser = StringParser;

        let expression = parser.parse(&token).unwrap();
        assert_eq!(expression, Expression::String("infer".to_string()));
    }
}
