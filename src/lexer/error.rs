use std::fmt;

/// Represents errors that can occur during the lexical analysis (tokenization) phase.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum LexError {
    /// Encountered a character that does not match any valid token.
    UnrecognizedChar(char, usize),
    /// A string literal was opened but not closed before the end of the file or line.
    UnterminatedStringLiteral(String, usize),
    /// Encountered an identifier matching a keyword format but which is currently unsupported.
    UnsupportedKeyword(String, usize),
}

impl fmt::Display for LexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnrecognizedChar(c, line) => {
                write!(formatter, "unrecognized character '{}' on line {}", c, line)
            }
            LexError::UnterminatedStringLiteral(s, line) => {
                write!(
                    formatter,
                    "unterminated string literal starting with '{}' on line {}",
                    s, line
                )
            }
            LexError::UnsupportedKeyword(k, line) => {
                write!(formatter, "unsupported keyword '{}' on line {}", k, line)
            }
        }
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_unrecognized_char() {
        let err = LexError::UnrecognizedChar('?', 12);
        assert_eq!(err.to_string(), "unrecognized character '?' on line 12");
    }

    #[test]
    fn display_unterminated_string_literal() {
        let err = LexError::UnterminatedStringLiteral("\"hello".to_string(), 45);
        assert_eq!(
            err.to_string(),
            "unterminated string literal starting with '\"hello' on line 45"
        );
    }

    #[test]
    fn display_unsupported_keyword() {
        let err = LexError::UnsupportedKeyword("fn".to_string(), 3);
        assert_eq!(err.to_string(), "unsupported keyword 'fn' on line 3");
    }
}
