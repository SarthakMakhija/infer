use crate::lexer::error::LexError;
use crate::lexer::token::TokenType;
use crate::parser::ast::error::ExpressionError;

/// Represents all syntactic and structural errors encountered during the parsing phase.
#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
    /// Expected a certain token type, but found a different one.
    ///
    /// The first `TokenType` is the expected token, and the second is the actual token found.
    UnexpectedTokenType(TokenType, TokenType),

    /// The parser reached the end of the input stream unexpectedly while parsing a construct.
    UnexpectedEof,

    /// A scanning error propagated from the lexical analysis phase.
    LexError(LexError),

    /// An error encountered while parsing or validating an individual expression.
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
