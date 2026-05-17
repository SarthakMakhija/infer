use crate::error::LexError;
use crate::keywords::Keywords;
use crate::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::CharIndices;

pub(crate) struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    keywords: Keywords,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str, keywords: Keywords) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            keywords,
            line: 1,
        }
    }

    pub(crate) fn lex(&mut self) -> Result<Token<'a>, LexError> {
        self.skip_whitespace();

        if let Some(&(index, char)) = self.peek() {
            return match char {
                '=' => {
                    self.next();
                    Ok(Token::equals(self.source, index, self.line))
                }
                ';' => {
                    self.next();
                    Ok(Token::semicolon(self.source, index, self.line))
                }
                ':' => {
                    self.next();
                    Ok(Token::colon(self.source, index, self.line))
                }
                '"' => {
                    self.next();
                    self.string(index)
                }
                char if Self::looks_like_identifier(char) => self.maybe_identifier(index),
                char if Self::looks_like_whole_number(char) => self.whole_number(index),
                _ => Err(LexError::UnrecognizedChar(char)),
            };
        }
        Ok(Token::eof(self.source, self.line))
    }

    fn next(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }

    fn increment_line(&mut self) {
        self.line += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(&(_, char)) = self.peek() {
            match char {
                '\n' => {
                    self.increment_line();
                    self.next();
                }
                ' ' | '\t' | '\r' => {
                    self.next();
                }
                _ => break,
            }
        }
    }

    fn maybe_identifier(&mut self, index: usize) -> Result<Token<'a>, LexError> {
        while let Some(&(next_index, char)) = self.peek() {
            if Self::looks_like_identifier(char) {
                self.next();
            } else {
                return self.identifier_or_keyword(index, next_index);
            }
        }
        self.identifier_or_keyword(index, self.source.len())
    }

    fn identifier_or_keyword(
        &self,
        index: usize,
        last_index: usize,
    ) -> Result<Token<'a>, LexError> {
        let token = &self.source[index..last_index];
        if self.keywords.has(token) {
            return Ok(Token::new(
                TokenType::keyword_type(token)?,
                index..last_index,
                self.line,
                self.source,
            ));
        }
        Ok(Token::new(
            TokenType::Identifier,
            index..last_index,
            self.line,
            self.source,
        ))
    }

    fn whole_number(&mut self, index: usize) -> Result<Token<'a>, LexError> {
        while let Some(&(next_index, char)) = self.peek() {
            if Self::looks_like_whole_number(char) {
                self.next();
            } else {
                return Ok(Token::new(
                    TokenType::WholeNumber,
                    index..next_index,
                    self.line,
                    self.source,
                ));
            }
        }
        Ok(Token::new(
            TokenType::WholeNumber,
            index..self.source.len(),
            self.line,
            self.source,
        ))
    }

    fn string(&mut self, index: usize) -> Result<Token<'a>, LexError> {
        while let Some(&(next_index, char)) = self.peek() {
            if char == '"' {
                self.next();
                return Ok(Token::new(
                    TokenType::StringLiteral,
                    index..next_index + 1,
                    self.line,
                    self.source,
                ));
            }
            if char == '\n' {
                self.increment_line();
            }
            self.next();
        }
        Err(LexError::UnterminatedStringLiteral(
            self.source[index..].to_string(),
        ))
    }

    fn looks_like_identifier(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn looks_like_whole_number(ch: char) -> bool {
        ch.is_ascii_digit()
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
        let mut lexer = Lexer::new("=", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Equals, 0..1);
        assert_token!(lexer.lex(), TokenType::Eof, 1..1);
    }

    #[test]
    fn lex_semicolon() {
        let mut lexer = Lexer::new(";", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Semicolon, 0..1);
        assert_token!(lexer.lex(), TokenType::Eof, 1..1);
    }

    #[test]
    fn lex_colon() {
        let mut lexer = Lexer::new(":", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Colon, 0..1);
        assert_token!(lexer.lex(), TokenType::Eof, 1..1);
    }

    #[test]
    fn lex_identifier() {
        let mut lexer = Lexer::new("name", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Identifier, 0..4);
        assert_token!(lexer.lex(), TokenType::Eof, 4..4);
    }

    #[test]
    fn lex_identifier_with_underscore() {
        let mut lexer = Lexer::new("first_name", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Identifier, 0..10);
        assert_token!(lexer.lex(), TokenType::Eof, 10..10);
    }

    #[test]
    fn lex_var_keyword() {
        let mut lexer = Lexer::new("var", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Var, 0..3);
        assert_token!(lexer.lex(), TokenType::Eof, 3..3);
    }

    #[test]
    fn lex_var_case_sensitive() {
        let mut lexer = Lexer::new("VAR", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Identifier, 0..3);
        assert_token!(lexer.lex(), TokenType::Eof, 3..3);
    }

    #[test]
    fn lex_whole_number() {
        let mut lexer = Lexer::new("123212", Keywords::new());
        assert_token!(lexer.lex(), TokenType::WholeNumber, 0..6);
        assert_token!(lexer.lex(), TokenType::Eof, 6..6);
    }

    #[test]
    fn lex_string() {
        let mut lexer = Lexer::new("\"john\"", Keywords::new());
        assert_token!(lexer.lex(), TokenType::StringLiteral, 0..6);
        assert_token!(lexer.lex(), TokenType::Eof, 6..6);
    }

    #[test]
    fn attempt_to_lex_unterminated_string() {
        let mut lexer = Lexer::new("\"john", Keywords::new());
        let result = lexer.lex();

        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), LexError::UnterminatedStringLiteral(str) if str == "\"john")
        );
    }

    #[test]
    fn attempt_to_lex_unterminated_multiline_string() {
        let mut lexer = Lexer::new("\"john\ndoe", Keywords::new());
        let result = lexer.lex();

        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), LexError::UnterminatedStringLiteral(str) if str == "\"john\ndoe")
        );
    }

    #[test]
    fn attempt_to_lex_unrecognized_character() {
        let mut lexer = Lexer::new("?", Keywords::new());
        let result = lexer.lex();
        assert!(matches!(result, Err(LexError::UnrecognizedChar(ch)) if ch == '?'));
    }

    #[test]
    fn lex_var_declaration() {
        let mut lexer = Lexer::new("var name = \"john\";", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Var, 0..3);
        assert_token!(lexer.lex(), TokenType::Identifier, 4..8);
        assert_token!(lexer.lex(), TokenType::Equals, 9..10);
        assert_token!(lexer.lex(), TokenType::StringLiteral, 11..17);
        assert_token!(lexer.lex(), TokenType::Semicolon, 17..18);
        assert_token!(lexer.lex(), TokenType::Eof, 18..18);
    }

    #[test]
    fn lex_var_declaration_with_type() {
        let mut lexer = Lexer::new("var name: string = \"john\";", Keywords::new());
        assert_token!(lexer.lex(), TokenType::Var, 0..3);
        assert_token!(lexer.lex(), TokenType::Identifier, 4..8);
        assert_token!(lexer.lex(), TokenType::Colon, 8..9);
        assert_token!(lexer.lex(), TokenType::Identifier, 10..16);
        assert_token!(lexer.lex(), TokenType::Equals, 17..18);
        assert_token!(lexer.lex(), TokenType::StringLiteral, 19..25);
        assert_token!(lexer.lex(), TokenType::Semicolon, 25..26);
        assert_token!(lexer.lex(), TokenType::Eof, 26..26);
    }
}
