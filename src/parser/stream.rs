use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use std::iter::Peekable;

/// A stateful wrapper around a lexer token stream that provides helper methods for parsing.
///
/// `ParserStream` maintains a peekable iterator of lex results, allowing sub-parsers to
/// look ahead, assert token types, and handle end-of-file conditions cleanly.
pub(crate) struct ParserStream<'a, T: Iterator<Item = LexResult<'a>>> {
    stream: Peekable<T>,
}

impl<'a, T: Iterator<Item = LexResult<'a>>> ParserStream<'a, T> {
    /// Creates a new `ParserStream` from any iterator yielding `LexResult`s.
    pub(crate) fn new(stream: T) -> ParserStream<'a, T> {
        Self {
            stream: stream.peekable(),
        }
    }

    /// Consumes the next token from the stream and asserts that it matches the `expected` token type.
    ///
    /// Returns the matched `Token` on success, or a `ParseError` (such as `UnexpectedTokenType` or `UnexpectedEof`) on mismatch.
    pub(crate) fn expect(&mut self, expected: TokenType) -> Result<Token<'a>, ParseError> {
        let optional_token = self.stream.next().transpose()?;
        match optional_token {
            Some(token) if token.token_type == expected => Ok(token),
            Some(token) => Err(ParseError::UnexpectedTokenType(expected, token.token_type)),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Asserts that the next token is an identifier and consumes it.
    pub(crate) fn expect_identifier(&mut self) -> Result<Token<'a>, ParseError> {
        self.expect(TokenType::Identifier)
    }

    /// Consumes and returns the next token from the stream regardless of its type.
    ///
    /// Returns `UnexpectedEof` if the stream is empty.
    pub(crate) fn expect_token(&mut self) -> Result<Token<'a>, ParseError> {
        let optional_token = self.stream.next().transpose()?;
        match optional_token {
            Some(token) => Ok(token),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Peeks at the next token, and consumes it if it matches the `expected` token type.
    ///
    /// Returns `true` and advances the stream if it matches; otherwise returns `false` without advancing.
    pub(crate) fn maybe_matches(&mut self, expected: TokenType) -> bool {
        if let Some(Ok(token)) = self.stream.peek() {
            if token.token_type == expected {
                let _ = self.stream.next();
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn expect_token_var() {
        let lexer = Lexer::new("var", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let token = stream.expect(TokenType::Var).unwrap();
        assert_eq!(token.token_type, TokenType::Var);
        assert_eq!(token.source, "var");
    }

    #[test]
    fn expect_token_mismatch() {
        let lexer = Lexer::new("name", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let result = stream.expect(TokenType::Var);
        assert_eq!(
            result.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::Var, TokenType::Identifier)
        );
    }

    #[test]
    fn expect_unexpected_eof() {
        let lexer = Lexer::new("", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let result = stream.expect(TokenType::Var);
        assert_eq!(result.err().unwrap(), ParseError::UnexpectedEof);
    }

    #[test]
    fn expect_identifier() {
        let lexer = Lexer::new("name", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let token = stream.expect_identifier().unwrap();
        assert_eq!(token.token_type, TokenType::Identifier);
        assert_eq!(token.source, "name");
    }

    #[test]
    fn expect_identifier_mismatch() {
        let lexer = Lexer::new("var", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let result = stream.expect_identifier();
        assert_eq!(
            result.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::Identifier, TokenType::Var)
        );
    }

    #[test]
    fn propagate_lex_error() {
        let lexer = Lexer::new("?", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let result = stream.expect(TokenType::Var);
        assert!(matches!(
            result.err().unwrap(),
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?'))
        ));
    }

    #[test]
    fn expect_token() {
        let lexer = Lexer::new("var name", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let var = stream.expect_token().unwrap();
        assert_eq!(var.token_type, TokenType::Var);

        let name = stream.expect_token().unwrap();
        assert_eq!(name.token_type, TokenType::Identifier);
    }

    #[test]
    fn attempt_to_get_token_where_there_is_none() {
        let lexer = Lexer::new("var name", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        stream.expect_token().unwrap();
        stream.expect_token().unwrap();

        let token = stream.expect_token();
        assert_eq!(token.err().unwrap(), ParseError::UnexpectedEof);
    }

    #[test]
    fn maybe_matches() {
        let lexer = Lexer::new("var name", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        assert!(stream.maybe_matches(TokenType::Var));
    }

    #[test]
    fn maybe_matches_does_not_match() {
        let lexer = Lexer::new("var name", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        assert!(!stream.maybe_matches(TokenType::Identifier));
    }

    #[test]
    fn maybe_matches_does_not_match_and_does_not_consume_token() {
        let lexer = Lexer::new("var name", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        assert!(!stream.maybe_matches(TokenType::Identifier));

        let token = stream.expect(TokenType::Var).unwrap();
        assert_eq!(token.token_type, TokenType::Var);
        assert_eq!(token.value(), "var");
    }

    #[test]
    fn maybe_matches_eof() {
        let lexer = Lexer::new("", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        assert!(!stream.maybe_matches(TokenType::Var));
    }
}
