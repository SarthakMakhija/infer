use crate::lexer::error::LexError;
use std::ops::Range;

/// The catalog of different syntactic categories (token types) supported by the language.
#[derive(Debug, PartialEq)]
pub(crate) enum TokenType {
    /// The `=` assignment operator.
    Equals,
    /// The `;` statement terminator.
    Semicolon,
    /// The `:` type annotation separator.
    Colon,
    /// A user-defined identifier (e.g. variable names or type names).
    Identifier,
    /// The `var` structural keyword.
    Var,
    /// A numeric integer literal.
    WholeNumber,
    /// A double-quoted string literal.
    StringLiteral,
}

impl TokenType {
    /// Maps a string slice lexeme to its corresponding reserved keyword `TokenType`.
    /// Returns `LexError::UnsupportedKeyword` if the slice is not a recognized keyword.
    pub(crate) fn keyword_type(token: &str) -> Result<Self, LexError> {
        match token {
            "var" => Ok(TokenType::Var),
            _ => Err(LexError::UnsupportedKeyword),
        }
    }
}

/// Represents a single lexical token scanned from the source code.
///
/// A `Token` keeps track of its type, its byte range in the original source,
/// the line number it was found on, and holds a reference to the source itself.
#[derive(Debug, PartialEq)]
pub(crate) struct Token<'a> {
    /// The syntactic category of this token.
    pub(crate) token_type: TokenType,
    /// The byte range span of this token in the source code.
    pub(crate) range: Range<usize>,
    /// The 1-based line number where this token is located.
    pub(crate) line: usize,
    /// A reference to the original source code slice.
    pub(crate) source: &'a str,
}

impl<'a> Token<'a> {
    /// Creates a generic new `Token`.
    pub(crate) fn new(
        token_type: TokenType,
        range: Range<usize>,
        line: usize,
        source: &'a str,
    ) -> Self {
        Self {
            token_type,
            range,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `=` operator.
    pub(crate) fn equals(source: &'a str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Equals,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `;` terminator.
    pub(crate) fn semicolon(source: &'a str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Semicolon,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `:` separator.
    pub(crate) fn colon(source: &'a str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Colon,
            range: index..index + 1,
            line,
            source,
        }
    }
}

#[cfg(test)]
mod token_type_tests {
    use super::*;

    #[test]
    fn keyword_type_var() {
        assert_eq!(TokenType::keyword_type("var").unwrap(), TokenType::Var);
    }

    #[test]
    fn keyword_type_invalid() {
        assert!(matches!(
            TokenType::keyword_type("unknown"),
            Err(LexError::UnsupportedKeyword)
        ));
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test]
    fn token_equals() {
        let token = Token::equals("=", 0, 1);
        assert_eq!(token.token_type, TokenType::Equals);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "=");
    }

    #[test]
    fn token_semicolon() {
        let token = Token::semicolon(";", 0, 2);
        assert_eq!(token.token_type, TokenType::Semicolon);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 2);
        assert_eq!(token.source, ";");
    }

    #[test]
    fn token_colon() {
        let token = Token::colon(":", 0, 1);
        assert_eq!(token.token_type, TokenType::Colon);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, ":");
    }

    #[test]
    fn new_token() {
        let token = Token::new(TokenType::Identifier, 4..8, 1, "var name = 10;");
        assert_eq!(token.token_type, TokenType::Identifier);
        assert_eq!(token.range, 4..8);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "var name = 10;");
    }
}
