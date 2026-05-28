use crate::ast::statement::{Iteration, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::statement::StatementParser;
use crate::parser::stream::ParserStream;

pub(crate) struct LoopParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> LoopParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::Loop)?;
        self.stream.expect(TokenType::LeftBrace)?;
        let body = self.parse_body()?;
        self.stream.expect(TokenType::RightBrace)?;

        Ok(Statement::iteration(Iteration::new(body)))
    }

    fn parse_body(&mut self) -> Result<Vec<Statement>, ParseError> {
        StatementParser::new(self.stream).parse_statements_till(TokenType::RightBrace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{BinaryOperator, Expression};
    use crate::ast::statement::{Assignment, Iteration, Statement};
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_empty_loop() {
        let lexer = Lexer::new("loop {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = LoopParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(statement, Statement::iteration(Iteration::new(vec![])));
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
            Statement::iteration(Iteration::new(vec![
                Statement::assignment(Assignment::new(
                    "counter".to_string(),
                    Expression::Binary(
                        Box::new(Expression::Identifier("counter".to_string())),
                        BinaryOperator::Plus,
                        Box::new(Expression::I32(1))
                    )
                )),
                Statement::assignment(Assignment::new(
                    "total".to_string(),
                    Expression::Binary(
                        Box::new(Expression::Identifier("total".to_string())),
                        BinaryOperator::Plus,
                        Box::new(Expression::Identifier("counter".to_string()))
                    )
                ))
            ]))
        );
    }
}
