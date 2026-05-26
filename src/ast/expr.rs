use std::fmt;

/// Represents errors encountered while parsing tokens into AST expressions.
#[derive(Debug, PartialEq)]
pub(crate) enum ExpressionError {
    /// Failed to parse a numeric literal string slice into a concrete integer value.
    ///
    /// Stores the invalid string that failed parsing and the line number where it occurred.
    ParseIntError(String, usize),
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionError::ParseIntError(val, line) => {
                write!(
                    formatter,
                    "failed to parse integer '{}' on line {}",
                    val, line
                )
            }
        }
    }
}

impl std::error::Error for ExpressionError {}

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    I32(i32),
    String(String),
    Identifier(String),
    Boolean(bool),
}
