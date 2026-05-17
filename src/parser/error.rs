use crate::lexer::error::LexError;
use crate::lexer::token::TokenType;

#[derive(Debug, PartialEq)]
pub(crate) enum ExpressionError {
    UnsupportedTokenType(TokenType),
    ParseIntError(String),
}

#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
    UnexpectedTokenType(TokenType, TokenType),
    UnexpectedEof,
    LexError(LexError),
    ExpressionError(ExpressionError),
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        ParseError::LexError(error)
    }
}
