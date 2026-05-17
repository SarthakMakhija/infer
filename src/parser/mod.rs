pub(crate) mod error;

use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use std::iter::Peekable;

pub(crate) struct ParserStream<'a, T: Iterator<Item = LexResult<'a>>> {
    stream: Peekable<T>,
}

impl<'a, T: Iterator<Item = LexResult<'a>>> ParserStream<'a, T> {
    pub(crate) fn new(stream: T) -> ParserStream<'a, T> {
        Self {
            stream: stream.peekable(),
        }
    }

    pub(crate) fn expect(&mut self, expected: TokenType) -> Result<Token<'a>, ParseError> {
        let optional_token = self.stream.next().transpose()?;
        match optional_token {
            Some(token) if token.token_type == expected => Ok(token),
            Some(token) => Err(ParseError::UnexpectedTokenType(expected, token.token_type)),
            None => Err(ParseError::UnexpectedEof),
        }
    }

    pub(crate) fn expect_identifier(&mut self) -> Result<Token<'a>, ParseError> {
        self.expect(TokenType::Identifier)
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
}
