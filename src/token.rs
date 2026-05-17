use std::ops::Range;

#[derive(Debug, PartialEq)]
pub(crate) enum TokenType {
    Equals,
    Semicolon,
    Colon,
    Identifier,
    WholeNumber,
    StringLiteral,
    Eof,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Token<'a> {
    pub(crate) token_type: TokenType,
    pub(crate) range: Range<usize>,
    pub(crate) source: &'a str,
}

impl<'a> Token<'a> {
    pub(crate) fn equals(source: &'a str, index: usize) -> Self {
        Self {
            token_type: TokenType::Equals,
            range: index..index + 1,
            source,
        }
    }

    pub(crate) fn semicolon(source: &'a str, index: usize) -> Self {
        Self {
            token_type: TokenType::Semicolon,
            range: index..index + 1,
            source,
        }
    }

    pub(crate) fn colon(source: &'a str, index: usize) -> Self {
        Self {
            token_type: TokenType::Colon,
            range: index..index + 1,
            source,
        }
    }

    pub(crate) fn eof(source: &'a str) -> Self {
        Self {
            token_type: TokenType::Eof,
            range: source.len()..source.len(),
            source,
        }
    }

    pub(crate) fn new(token_type: TokenType, range: Range<usize>, source: &'a str) -> Self {
        Self {
            token_type,
            range,
            source,
        }
    }
}
