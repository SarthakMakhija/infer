use crate::token::Token;
use std::iter::Peekable;
use std::str::CharIndices;
use crate::error::LexError;

pub(crate) struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    index: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            index: 0,
        }
    }

    pub(crate) fn lex(&mut self) -> Result<Token<'a>, LexError> {
        if let Some((index, char)) = self.next() {
            return match char {
                '=' => Ok(Token::equals(self.source, index)),
                _ => Err(LexError::UnrecognizedChar(char)),
            };
        }
        Ok(Token::eof(self.source))
    }

    fn next(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    macro_rules! assert_token {
        ($token:expr, $expected_type:pat, $expected_range:expr) => {
            let t = $token.expect("Expected Ok(Token), got Err");
            assert!(matches!(t.token_type, $expected_type), "Expected token type {:?}, got {:?}", stringify!($expected_type), t.token_type);
            assert_eq!(t.range, $expected_range, "Token range mismatch");
        };
    }

    #[test]
    fn lex_equals() {
        let mut lexer = Lexer::new("=");
        assert_token!(lexer.lex(), TokenType::Equals, 0..1);
        assert_token!(lexer.lex(), TokenType::EOF, 1..1);
    }

    #[test]
    fn attempt_to_lex_unrecognized_character() {
        let mut lexer = Lexer::new("?");
        let result = lexer.lex();
        assert!(matches!(result, Err(LexError::UnrecognizedChar(ch)) if ch == '?'));
    }
}