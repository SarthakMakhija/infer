use std::ops::Range;

#[derive(Debug, PartialEq)]
pub(crate) enum TokenType {
    Equals,
    EOF,
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
            range: index..index+1,
            source,
        }
    }

    pub(crate) fn eof(source: &'a str) -> Self {
        Self {
            token_type: TokenType::EOF,
            range: source.len()..source.len(),
            source,
        }
    }
}
