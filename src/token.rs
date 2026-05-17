use crate::error::LexError;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub(crate) enum TokenType {
    Equals,
    Semicolon,
    Colon,
    Identifier,
    Var,
    WholeNumber,
    StringLiteral,
    Eof,
}

impl TokenType {
    pub(crate) fn keyword_type(token: &str) -> Result<Self, LexError> {
        match token {
            "var" => Ok(TokenType::Var),
            _ => Err(LexError::UnsupportedKeyword),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Token<'a> {
    pub(crate) token_type: TokenType,
    pub(crate) range: Range<usize>,
    pub(crate) line: usize,
    pub(crate) source: &'a str,
}

impl<'a> Token<'a> {
    pub(crate) fn equals(source: &'a str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Equals,
            range: index..index + 1,
            line,
            source,
        }
    }

    pub(crate) fn semicolon(source: &'a str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Semicolon,
            range: index..index + 1,
            line,
            source,
        }
    }

    pub(crate) fn colon(source: &'a str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Colon,
            range: index..index + 1,
            line,
            source,
        }
    }

    pub(crate) fn eof(source: &'a str, line: usize) -> Self {
        Self {
            token_type: TokenType::Eof,
            range: source.len()..source.len(),
            line,
            source,
        }
    }

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
    fn token_eof() {
        let source = "var x = 10;";
        let token = Token::eof(source, 3);
        assert_eq!(token.token_type, TokenType::Eof);
        assert_eq!(token.range, source.len()..source.len());
        assert_eq!(token.line, 3);
        assert_eq!(token.source, source);
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
