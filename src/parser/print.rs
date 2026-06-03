use crate::ast::expr::Expression;
use crate::ast::statement::{Print, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::stream::ParserStream;

pub(crate) struct PrintParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> PrintParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::Print)?;

        let arguments = self.arguments()?;
        self.stream.expect(TokenType::Semicolon)?;

        Ok(Statement::print(Print::new(arguments)))
    }

    fn arguments(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut expression_parser = ExpressionParser::new(self.stream);
        let expression = expression_parser.parse()?;

        let mut arguments = Vec::new();
        arguments.push(expression);

        while expression_parser.stream.maybe_matches(TokenType::Comma) {
            let expression = expression_parser.parse()?;
            arguments.push(expression);
        }
        Ok(arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn parse_print_statement_with_multiple_arguments() {
        let lexer = Lexer::new("print name, 42, true;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = PrintParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::print(Print::new(vec![
                Expression::identifier("name".to_string()),
                Expression::I32(42),
                Expression::Boolean(true),
            ]))
        );
    }

    #[test]
    fn parse_print_statement_with_expressions() {
        let lexer = Lexer::new("print age + 10;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = PrintParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::print(Print::new(vec![Expression::Binary(
                Box::new(Expression::identifier("age".to_string())),
                crate::ast::expr::BinaryOperator::Plus,
                Box::new(Expression::I32(10))
            )]))
        );
    }
}
