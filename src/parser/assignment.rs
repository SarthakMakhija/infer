use crate::ast::expr::Expression;
use crate::ast::statement::{Assignment, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::stream::ParserStream;

pub(crate) struct AssignmentParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> AssignmentParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        let name = self.stream.expect_identifier()?;
        self.stream.expect(TokenType::Equals)?;

        let expression = self.expression()?;
        self.stream.expect(TokenType::Semicolon)?;

        Ok(Statement::assignment(Assignment::new(
            name.owned_value(),
            expression,
        )))
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression_parser = ExpressionParser::new(self.stream);
        let expression = expression_parser.parse()?;
        Ok(expression)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn parse_valid_assignment() {
        let lexer = Lexer::new("id = 20;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::assignment(Assignment::new("id".to_string(), Expression::I32(20)))
        );
    }

    #[test]
    fn parse_assignment_with_complex_expression() {
        let lexer = Lexer::new("total = amount + rate * percentage;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::assignment(Assignment::new(
                "total".to_string(),
                Expression::Binary(
                    Box::new(Expression::Identifier("amount".to_string())),
                    crate::ast::expr::BinaryOperator::Plus,
                    Box::new(Expression::Binary(
                        Box::new(Expression::Identifier("rate".to_string())),
                        crate::ast::expr::BinaryOperator::Multiply,
                        Box::new(Expression::Identifier("percentage".to_string()))
                    ))
                )
            ))
        );
    }

    #[test]
    fn parse_assignment_missing_semicolon() {
        let lexer = Lexer::new("x = 20", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let result = parser.parse();
        assert_eq!(result.err().unwrap(), ParseError::UnexpectedEof);
    }
}
