use crate::error::LexError;
use crate::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::CharIndices;

pub(crate) struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
        }
    }

    pub(crate) fn lex(&mut self) -> Result<Token<'a>, LexError> {
        if let Some(&(index, char)) = self.peek() {
            return match char {
                '=' => {
                    self.next();
                    Ok(Token::equals(self.source, index))
                }
                ';' => {
                    self.next();
                    Ok(Token::semicolon(self.source, index))
                }
                ':' => {
                    self.next();
                    Ok(Token::colon(self.source, index))
                }
                char if Self::looks_like_identifier(char) => Ok(self.identifier(index)),
                _ => Err(LexError::UnrecognizedChar(char)),
            };
        }
        Ok(Token::eof(self.source))
    }

    fn next(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }

    fn identifier(&mut self, index: usize) -> Token<'a> {
        while let Some(&(next_index, char)) = self.peek() {
            if Self::looks_like_identifier(char) {
                self.next();
            } else {
                return Token::new(TokenType::Identifier, index..next_index, self.source);
            }
        }
        Token::new(TokenType::Identifier, index..self.source.len(), self.source)
    }

    fn looks_like_identifier(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    macro_rules! assert_token {
        ($token:expr, $expected_type:pat, $expected_range:expr) => {
            let t = $token.expect("Expected Ok(Token), got Err");
            assert!(
                matches!(t.token_type, $expected_type),
                "Expected token type {:?}, got {:?}",
                stringify!($expected_type),
                t.token_type
            );
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
    fn lex_semicolon() {
        let mut lexer = Lexer::new(";");
        assert_token!(lexer.lex(), TokenType::Semicolon, 0..1);
        assert_token!(lexer.lex(), TokenType::EOF, 1..1);
    }

    #[test]
    fn lex_colon() {
        let mut lexer = Lexer::new(":");
        assert_token!(lexer.lex(), TokenType::Colon, 0..1);
        assert_token!(lexer.lex(), TokenType::EOF, 1..1);
    }

    #[test]
    fn lex_identifier() {
        let mut lexer = Lexer::new("name");
        assert_token!(lexer.lex(), TokenType::Identifier, 0..4);
        assert_token!(lexer.lex(), TokenType::EOF, 4..4);
    }

    #[test]
    fn lex_identifier_with_underscore() {
        let mut lexer = Lexer::new("first_name");
        assert_token!(lexer.lex(), TokenType::Identifier, 0..10);
        assert_token!(lexer.lex(), TokenType::EOF, 10..10);
    }

    #[test]
    fn attempt_to_lex_unrecognized_character() {
        let mut lexer = Lexer::new("?");
        let result = lexer.lex();
        assert!(matches!(result, Err(LexError::UnrecognizedChar(ch)) if ch == '?'));
    }
}
