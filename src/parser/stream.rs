use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use std::collections::VecDeque;

/// A stateful wrapper around a lexer token stream that provides helper methods for parsing.
///
/// `ParserStream` maintains a peekable iterator of lex results, allowing sub-parsers to
/// look ahead, assert token types, and handle end-of-file conditions cleanly.
pub(crate) struct ParserStream<'src, T: Iterator<Item = LexResult<'src>>> {
    stream: T,
    buffer: VecDeque<LexResult<'src>>,
}

impl<'src, T: Iterator<Item = LexResult<'src>>> ParserStream<'src, T> {
    /// Creates a new `ParserStream` from any iterator yielding `LexResult`s.
    pub(crate) fn new(stream: T) -> ParserStream<'src, T> {
        Self {
            stream,
            buffer: VecDeque::new(),
        }
    }

    /// Consumes the next token from the stream and asserts that it matches the `expected` token type.
    ///
    /// Returns the matched `Token` on success, or a `ParseError` (such as `UnexpectedTokenType` or `UnexpectedEof`) on mismatch.
    pub(crate) fn expect(&mut self, expected: TokenType) -> Result<Token<'src>, ParseError> {
        self.fill_buffer(1);
        let optional_token = self.buffer.pop_front().transpose()?;
        match optional_token {
            Some(token) if token.token_type == expected => Ok(token),
            Some(token) => Err(ParseError::UnexpectedTokenType(
                expected,
                token.token_type,
                token.line,
            )),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Asserts that the next token is an identifier and consumes it.
    pub(crate) fn expect_identifier(&mut self) -> Result<Token<'src>, ParseError> {
        self.expect(TokenType::Identifier)
    }

    /// Consumes and returns the next token from the stream regardless of its type.
    ///
    /// Returns `UnexpectedEof` if the stream is empty.
    pub(crate) fn expect_token(&mut self) -> Result<Token<'src>, ParseError> {
        self.fill_buffer(1);
        let optional_token = self.buffer.pop_front().transpose()?;
        match optional_token {
            Some(token) => Ok(token),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    /// Peeks at the next token, and consumes it if it matches the `expected` token type.
    ///
    /// Returns `true` and advances the stream if it matches; otherwise returns `false` without advancing.
    pub(crate) fn maybe_matches(&mut self, expected: TokenType) -> bool {
        self.fill_buffer(1);
        if let Some(Ok(token)) = self.buffer.front() {
            if token.token_type == expected {
                let _ = self.buffer.pop_front();
                return true;
            }
        }
        false
    }

    /// Peeks at the next item in the token stream without consuming it.
    ///
    /// Returns `None` if the stream has reached the end-of-file.
    pub(crate) fn peek(&mut self) -> Result<Option<&Token<'src>>, ParseError> {
        self.fill_buffer(1);
        match self.buffer.front() {
            None => Ok(None),
            Some(Err(err)) => Err(ParseError::LexError(err.clone())),
            Some(Ok(token)) => Ok(Some(token)),
        }
    }

    fn fill_buffer(&mut self, count: usize) {
        while self.buffer.len() < count {
            if let Some(result) = self.stream.next() {
                self.buffer.push_back(result);
            } else {
                break;
            }
        }
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
            ParseError::UnexpectedTokenType(TokenType::Var, TokenType::Identifier, 1)
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
            ParseError::UnexpectedTokenType(TokenType::Identifier, TokenType::Var, 1)
        );
    }

    #[test]
    fn propagate_lex_error() {
        let lexer = Lexer::new("?", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let result = stream.expect(TokenType::Var);
        assert!(matches!(
            result.err().unwrap(),
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?', 1))
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

    #[test]
    fn peek() {
        let lexer = Lexer::new("var id = 100;", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let peeked = stream.peek().unwrap();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().token_type, TokenType::Var);
    }

    #[test]
    fn peek_does_not_consume() {
        let lexer = Lexer::new("var id = 100;", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let peeked = stream.peek().unwrap();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().token_type, TokenType::Var);

        let consumed = stream.expect(TokenType::Var).unwrap();
        assert_eq!(consumed.token_type, TokenType::Var);
    }

    #[test]
    fn peek_returns_lex_error_given_unrecognized_token() {
        let lexer = Lexer::new("?", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let peeked_err = stream.peek();
        assert!(peeked_err.is_err());
        assert!(matches!(
            peeked_err.err().unwrap(),
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?', 1))
        ));
    }

    #[test]
    fn peek_eof() {
        let lexer = Lexer::new("", Keywords::new());
        let mut stream = ParserStream::new(lexer);

        let peeked = stream.peek().unwrap();
        assert!(peeked.is_none());
    }
}
