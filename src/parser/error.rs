use crate::lexer::error::LexError;
use crate::lexer::token::TokenType;

#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
    UnexpectedTokenType(TokenType, TokenType),
    UnexpectedEof,
    LexError(LexError),
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        ParseError::LexError(error)
    }
}
