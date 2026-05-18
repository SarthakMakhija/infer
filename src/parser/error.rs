use crate::ast::expr::ExpressionError;
use crate::lexer::error::LexError;
use crate::lexer::token::TokenType;
use std::fmt;

/// Represents all syntactic and structural errors encountered during the parsing phase.
#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
    /// Expected a certain token type, but found a different one.
    ///
    /// The first `TokenType` is the expected token, and the second is the actual token found.
    UnexpectedTokenType(TokenType, TokenType, usize),

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

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedTokenType(expected, actual, line) => {
                write!(
                    formatter,
                    "expected token '{:?}', but found '{:?}' on line {}",
                    expected, actual, line
                )
            }
            ParseError::UnexpectedEof => {
                write!(formatter, "unexpected end of file")
            }
            ParseError::LexError(err) => {
                write!(formatter, "{}", err)
            }
            ParseError::ExpressionError(err) => {
                write!(formatter, "{}", err)
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_unexpected_token_type() {
        let err = ParseError::UnexpectedTokenType(TokenType::Var, TokenType::Identifier, 4);
        assert_eq!(
            err.to_string(),
            "expected token 'Var', but found 'Identifier' on line 4"
        );
    }

    #[test]
    fn display_unexpected_eof() {
        let err = ParseError::UnexpectedEof;
        assert_eq!(err.to_string(), "unexpected end of file");
    }

    #[test]
    fn display_lex_error() {
        let lex_err = LexError::UnrecognizedChar('$', 2);
        let err = ParseError::from(lex_err);
        assert_eq!(err.to_string(), "unrecognized character '$' on line 2");
    }

    #[test]
    fn display_expression_error() {
        let expression_err = ExpressionError::ParseIntError("1234567890123456".to_string(), 7);
        let err = ParseError::from(expression_err);
        assert_eq!(
            err.to_string(),
            "failed to parse integer '1234567890123456' on line 7"
        );
    }
}
