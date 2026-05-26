use crate::ast::expr::Expression;
use crate::lexer::token::Token;
use crate::parser::error::ParseError;
use crate::parser::expr::PrefixRule;

pub(crate) struct String;

impl<'src> PrefixRule<'src> for String {
    fn parse(&mut self, token: &'src Token) -> Result<Expression, ParseError> {
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
        let mut rule = String;

        let expression = rule.parse(&token).unwrap();
        assert_eq!(expression, Expression::String("infer".to_string()));
    }
}
