use crate::lexer::token::TokenType;

#[derive(Debug, PartialEq)]
pub(crate) enum ExpressionError {
    UnsupportedTokenType(TokenType),
    ParseIntError(String),
}
