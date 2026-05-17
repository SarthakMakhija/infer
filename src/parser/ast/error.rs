use crate::lexer::token::TokenType;

/// Represents errors encountered while parsing tokens into AST expressions.
#[derive(Debug, PartialEq)]
pub(crate) enum ExpressionError {
    /// The parser encountered a token type that cannot be converted into or used as an expression.
    UnsupportedTokenType(TokenType),

    /// Failed to parse a numeric literal string slice into a concrete integer value.
    ///
    /// Stores the invalid string that failed parsing.
    ParseIntError(String),
}
