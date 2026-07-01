use crate::ast::statement::{Block, Loop, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::block::BlockParser;
use crate::parser::error::ParseError;
use crate::parser::stream::ParserStream;

/// A sub-parser responsible for parsing `loop` iteration statements.
///
/// See [grammar.ebnf](../../docs/grammar.ebnf) for the full language grammar.
pub(crate) struct LoopParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> LoopParser<'src, 'stream, I> {
    /// Creates a new `LoopParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses a `loop { ... }` block from the token stream.
    ///
    /// Expects the `loop` keyword, an opening `{`, zero or more inner statements,
    /// and a closing `}`.
    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::Loop)?;
        let body = self.parse_body()?;

        Ok(Statement::iteration(Loop::new(body)))
    }

    fn parse_body(&mut self) -> Result<Block, ParseError> {
        BlockParser::new(self.stream).parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_empty_loop() {
        let lexer = Lexer::new("loop {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = LoopParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(statement, iteration!(block!()));
    }

    #[test]
    fn parse_loop_with_statements() {
        let lexer = Lexer::new(
            "loop { counter = counter + 1; total = total + counter; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = LoopParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        assert_eq!(
            statement,
            iteration!(block!(
                assignment!(
                    "counter",
                    expression_binary!(
                        expression_identifier!("counter"),
                        Plus,
                        expression_i32!(1),
                        line
                    )
                ),
                assignment!(
                    "total",
                    expression_binary!(
                        expression_identifier!("total"),
                        Plus,
                        expression_identifier!("counter"),
                        line
                    )
                )
            ))
        );
    }

    #[test]
    fn parse_loop_missing_left_brace() {
        let lexer = Lexer::new("loop break;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = LoopParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::LeftBrace, TokenType::Break, 1)
        );
    }

    #[test]
    fn parse_loop_missing_right_brace() {
        let lexer = Lexer::new("loop { break;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = LoopParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }
}
