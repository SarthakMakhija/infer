use crate::lexer::error::LexError;
use crate::lexer::token::TokenType;
use crate::parser::ast::error::ExpressionError;

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

impl From<ExpressionError> for ParseError {
    fn from(error: ExpressionError) -> Self {
        ParseError::ExpressionError(error)
    }
}
