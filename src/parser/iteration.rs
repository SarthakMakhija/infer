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
    use crate::ast::expr::{BinaryOperator, Expression};
    use crate::ast::statement::{Assignment, Block, Loop, Statement};
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_empty_loop() {
        let lexer = Lexer::new("loop {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = LoopParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::iteration(Loop::new(Block::new(vec![])))
        );
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
        assert_eq!(
            statement,
            Statement::iteration(Loop::new(Block::new(vec![
                Statement::assignment(Assignment::new(
                    "counter".to_string(),
                    Expression::Binary(
                        Box::new(Expression::identifier("counter".to_string())),
                        BinaryOperator::Plus,
                        Box::new(Expression::I32(1))
                    )
                )),
                Statement::assignment(Assignment::new(
                    "total".to_string(),
                    Expression::Binary(
                        Box::new(Expression::identifier("total".to_string())),
                        BinaryOperator::Plus,
                        Box::new(Expression::identifier("counter".to_string()))
                    )
                ))
            ])))
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
