//! The `lexer` module contains the scanner/tokenizer implementation.
//!
//! It converts a raw string of source code into a stream of structured `Token`s.

pub(crate) mod error;
pub(crate) mod keywords;
pub(crate) mod token;

use crate::lexer::error::LexError;
use crate::lexer::keywords::Keywords;
use crate::lexer::token::{Token, TokenType};
use std::iter::Peekable;
use std::str::CharIndices;

/// A convenience type alias for standard Result returned by the lexer.
pub(crate) type LexResult<'src> = Result<Token<'src>, LexError>;

/// A stateful lexical analyzer that scans source code and emits tokens.
///
/// The `Lexer` implements the standard `Iterator` trait, making it an
/// on-demand, lazy stream of `LexResult`s.
pub(crate) struct Lexer<'src> {
    source: &'src str,
    chars: Peekable<CharIndices<'src>>,
    keywords: Keywords,
    line: usize,
}

impl<'src> Lexer<'src> {
    /// Creates a new `Lexer` instance for the given source string.
    pub(crate) fn new(source: &'src str, keywords: Keywords) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            keywords,
            line: 1,
        }
    }

    fn move_ahead(&mut self) -> Option<(usize, char)> {
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
                    self.move_ahead();
                }
                ' ' | '\t' | '\r' => {
                    self.move_ahead();
                }
                _ => break,
            }
        }
    }

    fn maybe_identifier(&mut self, index: usize) -> LexResult<'src> {
        while let Some(&(next_index, char)) = self.peek() {
            if Self::is_identifier_char(char) {
                self.move_ahead();
            } else {
                return self.identifier_or_keyword(index, next_index);
            }
        }
        self.identifier_or_keyword(index, self.source.len())
    }

    fn identifier_or_keyword(&self, index: usize, last_index: usize) -> LexResult<'src> {
        let token = &self.source[index..last_index];
        if self.keywords.has(token) {
            return Ok(Token::new(
                TokenType::keyword_type(token, self.line)?,
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

    fn whole_number(&mut self, index: usize) -> LexResult<'src> {
        while let Some(&(next_index, char)) = self.peek() {
            if Self::looks_like_whole_number(char) {
                self.move_ahead();
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

    fn string(&mut self, index: usize) -> LexResult<'src> {
        while let Some(&(next_index, char)) = self.peek() {
            if char == '"' {
                self.move_ahead();
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
            self.move_ahead();
        }
        Err(LexError::UnterminatedStringLiteral(
            self.source[index..].to_string(),
            self.line,
        ))
    }

    fn is_identifier_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn is_identifier_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }

    fn looks_like_whole_number(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    fn match_next(&mut self, expected: char) -> bool {
        if let Some(&(_, ch)) = self.peek() {
            if ch == expected {
                self.move_ahead();
                return true;
            }
        }
        false
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = LexResult<'src>;

    /// Scans the next token from the source code.
    ///
    /// Silently consumes any leading whitespace or newlines, updating the line counter.
    /// Returns `Some(Ok(Token))` for successfully scanned tokens, `Some(Err(LexError))`
    /// if a scanning error occurs, and `None` when the end of the source is reached.
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        if let Some(&(index, char)) = self.peek() {
            return match char {
                '=' => {
                    self.move_ahead();
                    if self.match_next('=') {
                        Some(Ok(Token::equals_equals(self.source, index, self.line)))
                    } else {
                        Some(Ok(Token::equals(self.source, index, self.line)))
                    }
                }
                ';' => {
                    self.move_ahead();
                    Some(Ok(Token::semicolon(self.source, index, self.line)))
                }
                ':' => {
                    self.move_ahead();
                    Some(Ok(Token::colon(self.source, index, self.line)))
                }
                '+' => {
                    self.move_ahead();
                    Some(Ok(Token::plus(self.source, index, self.line)))
                }
                '-' => {
                    self.move_ahead();
                    Some(Ok(Token::minus(self.source, index, self.line)))
                }
                '*' => {
                    self.move_ahead();
                    Some(Ok(Token::star(self.source, index, self.line)))
                }
                '/' => {
                    self.move_ahead();
                    Some(Ok(Token::slash(self.source, index, self.line)))
                }
                '!' => {
                    self.move_ahead();
                    if self.match_next('=') {
                        Some(Ok(Token::bang_equals(self.source, index, self.line)))
                    } else {
                        Some(Ok(Token::bang(self.source, index, self.line)))
                    }
                }
                '>' => {
                    self.move_ahead();
                    if self.match_next('=') {
                        Some(Ok(Token::greater_than_equals(
                            self.source,
                            index,
                            self.line,
                        )))
                    } else {
                        Some(Ok(Token::greater_than(self.source, index, self.line)))
                    }
                }
                '<' => {
                    self.move_ahead();
                    if self.match_next('=') {
                        Some(Ok(Token::less_than_equals(self.source, index, self.line)))
                    } else {
                        Some(Ok(Token::less_than(self.source, index, self.line)))
                    }
                }
                '(' => {
                    self.move_ahead();
                    Some(Ok(Token::left_parentheses(self.source, index, self.line)))
                }
                ')' => {
                    self.move_ahead();
                    Some(Ok(Token::right_parentheses(self.source, index, self.line)))
                }
                '{' => {
                    self.move_ahead();
                    Some(Ok(Token::left_brace(self.source, index, self.line)))
                }
                '}' => {
                    self.move_ahead();
                    Some(Ok(Token::right_brace(self.source, index, self.line)))
                }
                '"' => {
                    self.move_ahead();
                    Some(self.string(index))
                }
                char if Self::is_identifier_start(char) => Some(self.maybe_identifier(index)),
                char if Self::looks_like_whole_number(char) => Some(self.whole_number(index)),
                _ => Some(Err(LexError::UnrecognizedChar(char, self.line))),
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_token {
        ($token:expr, $expected_type:pat, $expected_range:expr) => {
            let t = $token
                .expect("Expected Some(Token), got None")
                .expect("Expected Ok(Token), got Err");

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
        assert_token!(lexer.next(), TokenType::Equals, 0..1);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_semicolon() {
        let mut lexer = Lexer::new(";", Keywords::new());
        assert_token!(lexer.next(), TokenType::Semicolon, 0..1);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_colon() {
        let mut lexer = Lexer::new(":", Keywords::new());
        assert_token!(lexer.next(), TokenType::Colon, 0..1);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_bang_operator() {
        let mut lexer = Lexer::new("!", Keywords::new());
        assert_token!(lexer.next(), TokenType::Bang, 0..1);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_left_paren() {
        let mut lexer = Lexer::new("(", Keywords::new());
        assert_token!(lexer.next(), TokenType::LeftParentheses, 0..1);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_right_paren() {
        let mut lexer = Lexer::new(")", Keywords::new());
        assert_token!(lexer.next(), TokenType::RightParentheses, 0..1);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_identifier() {
        let mut lexer = Lexer::new("name", Keywords::new());
        assert_token!(lexer.next(), TokenType::Identifier, 0..4);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_identifier_with_underscore() {
        let mut lexer = Lexer::new("first_name", Keywords::new());
        assert_token!(lexer.next(), TokenType::Identifier, 0..10);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_identifier_containing_digits() {
        let mut lexer = Lexer::new("i32", Keywords::new());
        assert_token!(lexer.next(), TokenType::Identifier, 0..3);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_number_is_not_identifier() {
        let mut lexer = Lexer::new("100", Keywords::new());
        assert_token!(lexer.next(), TokenType::WholeNumber, 0..3);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_var_keyword() {
        let mut lexer = Lexer::new("var", Keywords::new());
        assert_token!(lexer.next(), TokenType::Var, 0..3);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_if_keyword() {
        let mut lexer = Lexer::new("if", Keywords::new());
        assert_token!(lexer.next(), TokenType::If, 0..2);
    }

    #[test]
    fn lex_else_keyword() {
        let mut lexer = Lexer::new("else", Keywords::new());
        assert_token!(lexer.next(), TokenType::Else, 0..4);
    }

    #[test]
    fn lex_var_case_sensitive() {
        let mut lexer = Lexer::new("VAR", Keywords::new());
        assert_token!(lexer.next(), TokenType::Identifier, 0..3);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_whole_number() {
        let mut lexer = Lexer::new("123212", Keywords::new());
        assert_token!(lexer.next(), TokenType::WholeNumber, 0..6);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_string() {
        let mut lexer = Lexer::new("\"john\"", Keywords::new());
        assert_token!(lexer.next(), TokenType::StringLiteral, 0..6);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn attempt_to_lex_unterminated_string() {
        let mut lexer = Lexer::new("\"john", Keywords::new());
        let result = lexer.next();

        assert!(result.is_some());
        assert!(
            matches!(result.unwrap().err().unwrap(), LexError::UnterminatedStringLiteral(ref str, 1) if str == "\"john")
        );
    }

    #[test]
    fn attempt_to_lex_unterminated_multiline_string() {
        let mut lexer = Lexer::new("\"john\ndoe", Keywords::new());
        let result = lexer.next();

        assert!(result.is_some());
        assert!(
            matches!(result.unwrap().err().unwrap(), LexError::UnterminatedStringLiteral(ref str, 2) if str == "\"john\ndoe")
        );
    }

    #[test]
    fn attempt_to_lex_unrecognized_character() {
        let mut lexer = Lexer::new("?", Keywords::new());
        let result = lexer.next().unwrap();
        assert!(matches!(result, Err(LexError::UnrecognizedChar(ch, 1)) if ch == '?'));
    }

    #[test]
    fn lex_var_declaration() {
        let mut lexer = Lexer::new("var name = \"john\";", Keywords::new());
        assert_token!(lexer.next(), TokenType::Var, 0..3);
        assert_token!(lexer.next(), TokenType::Identifier, 4..8);
        assert_token!(lexer.next(), TokenType::Equals, 9..10);
        assert_token!(lexer.next(), TokenType::StringLiteral, 11..17);
        assert_token!(lexer.next(), TokenType::Semicolon, 17..18);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_var_declaration_with_type() {
        let mut lexer = Lexer::new("var name: string = \"john\";", Keywords::new());
        assert_token!(lexer.next(), TokenType::Var, 0..3);
        assert_token!(lexer.next(), TokenType::Identifier, 4..8);
        assert_token!(lexer.next(), TokenType::Colon, 8..9);
        assert_token!(lexer.next(), TokenType::Identifier, 10..16);
        assert_token!(lexer.next(), TokenType::Equals, 17..18);
        assert_token!(lexer.next(), TokenType::StringLiteral, 19..25);
        assert_token!(lexer.next(), TokenType::Semicolon, 25..26);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_plus_operator() {
        let mut lexer = Lexer::new("amount + increment", Keywords::new());
        assert_token!(lexer.next(), TokenType::Identifier, 0..6);
        assert_token!(lexer.next(), TokenType::Plus, 7..8);
        assert_token!(lexer.next(), TokenType::Identifier, 9..18);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_minus_operator() {
        let mut lexer = Lexer::new("a - b", Keywords::new());
        assert_token!(lexer.next(), TokenType::Identifier, 0..1);
        assert_token!(lexer.next(), TokenType::Minus, 2..3);
        assert_token!(lexer.next(), TokenType::Identifier, 4..5);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_star_operator() {
        let mut lexer = Lexer::new("amount * factor", Keywords::new());
        assert_token!(lexer.next(), TokenType::Identifier, 0..6);
        assert_token!(lexer.next(), TokenType::Star, 7..8);
        assert_token!(lexer.next(), TokenType::Identifier, 9..15);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_slash_operator() {
        let mut lexer = Lexer::new("a / b", Keywords::new());
        assert_token!(lexer.next(), TokenType::Identifier, 0..1);
        assert_token!(lexer.next(), TokenType::Slash, 2..3);
        assert_token!(lexer.next(), TokenType::Identifier, 4..5);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn lex_equals_equals() {
        let mut lexer = Lexer::new("==", Keywords::new());
        assert_token!(lexer.next(), TokenType::EqualsEquals, 0..2);
    }

    #[test]
    fn lex_bang_equals() {
        let mut lexer = Lexer::new("!=", Keywords::new());
        assert_token!(lexer.next(), TokenType::BangEquals, 0..2);
    }

    #[test]
    fn lex_greater_than() {
        let mut lexer = Lexer::new(">", Keywords::new());
        assert_token!(lexer.next(), TokenType::GreaterThan, 0..1);
    }

    #[test]
    fn lex_greater_than_equals() {
        let mut lexer = Lexer::new(">=", Keywords::new());
        assert_token!(lexer.next(), TokenType::GreaterThanEquals, 0..2);
    }

    #[test]
    fn lex_less_than() {
        let mut lexer = Lexer::new("<", Keywords::new());
        assert_token!(lexer.next(), TokenType::LessThan, 0..1);
    }

    #[test]
    fn lex_less_than_equals() {
        let mut lexer = Lexer::new("<=", Keywords::new());
        assert_token!(lexer.next(), TokenType::LessThanEquals, 0..2);
    }

    #[test]
    fn lex_left_brace() {
        let mut lexer = Lexer::new("{", Keywords::new());
        assert_token!(lexer.next(), TokenType::LeftBrace, 0..1);
    }

    #[test]
    fn lex_right_brace() {
        let mut lexer = Lexer::new("}", Keywords::new());
        assert_token!(lexer.next(), TokenType::RightBrace, 0..1);
    }
}
