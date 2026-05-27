use crate::ast::expr::ExpressionError;
use crate::lexer::error::LexError;
use crate::lexer::token::TokenType;
use std::fmt;

/// Represents all syntactic and structural errors encountered during the parsing phase.
#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
    /// Expected a certain token type, but found a different one.
    ///
    /// Stores the expected `TokenType`, the actual `TokenType` found, and the line number where it occurred.
    UnexpectedTokenType(TokenType, TokenType, usize),

    /// The parser reached the end of the input stream unexpectedly while parsing a construct.
    UnexpectedEof,

    /// A scanning error propagated from the lexical analysis phase.
    LexError(LexError),

    /// An error encountered while parsing or validating an individual expression.
    ExpressionError(ExpressionError),

    /// An error encountered while parsing prefix expressions, with unsupported prefix parser.
    UnsupportedPrefixExpression(TokenType, usize),

    /// An error encountered while parsing infix expressions, with unsupported infix parser.
    UnsupportedInfixExpression(TokenType, usize),

    /// An error encountered while parsing chained comparison operations like a < b < c.
    ChainedComparison(usize),
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
            ParseError::UnsupportedPrefixExpression(actual, line) => {
                write!(
                    formatter,
                    "no supported prefix parser for '{:?}' on line {}",
                    actual, line
                )
            }
            ParseError::UnsupportedInfixExpression(actual, line) => {
                write!(
                    formatter,
                    "no supported infix parser for '{:?}' on line {}",
                    actual, line
                )
            }
            ParseError::ChainedComparison(line) => {
                write!(
                    formatter,
                    "chained comparison operations are not supported on line {}",
                    line
                )
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

    #[test]
    fn display_unsupported_infix_expression() {
        let err = ParseError::UnsupportedInfixExpression(TokenType::Plus, 10);
        assert_eq!(
            err.to_string(),
            "no supported infix parser for 'Plus' on line 10"
        );
    }

    #[test]
    fn display_chained_comparison_error() {
        let err = ParseError::ChainedComparison(12);
        assert_eq!(
            err.to_string(),
            "chained comparison operations are not supported on line 12"
        );
    }
}
