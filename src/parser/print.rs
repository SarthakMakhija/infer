use crate::ast::expr::Expression;
#[cfg(test)]
use crate::ast::expr::ExpressionKind;
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
        let print_token = self.stream.expect(TokenType::Print)?;

        let arguments = self.arguments(print_token.line)?;
        self.stream.expect(TokenType::Semicolon)?;

        Ok(Statement::print(Print::new(arguments)))
    }

    fn arguments(&mut self, line: usize) -> Result<Vec<Expression>, ParseError> {
        let mut expression_parser = ExpressionParser::new(self.stream);
        let expression_kind = expression_parser.parse()?;

        let mut arguments = Vec::new();
        arguments.push(Expression::new(expression_kind, line));

        while expression_parser.stream.maybe_matches(TokenType::Comma) {
            if let Some(next_token) = expression_parser.stream.peek()? {
                if next_token.token_type == TokenType::Semicolon {
                    return Err(ParseError::TrailingComma(next_token.line));
                }
            }
            let expression_kind = expression_parser.parse()?;
            arguments.push(Expression::new(expression_kind, line));
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
        let line = 1;
        assert_eq!(
            statement,
            Statement::print(Print::new(vec![
                Expression::new(ExpressionKind::identifier("name".to_string()), line),
                Expression::new(ExpressionKind::I32(42), line),
                Expression::new(ExpressionKind::Boolean(true), line),
            ]))
        );
    }

    #[test]
    fn parse_print_statement_with_expressions() {
        let lexer = Lexer::new("print age + 10;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = PrintParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;
        assert_eq!(
            statement,
            Statement::print(Print::new(vec![Expression::new(
                ExpressionKind::Binary(
                    Box::new(ExpressionKind::identifier("age".to_string())),
                    crate::ast::expr::BinaryOperator::Plus,
                    Box::new(ExpressionKind::I32(10))
                ),
                line
            )]))
        );
    }

    #[test]
    fn parse_print_statement_missing_semicolon() {
        let lexer = Lexer::new("print name", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = PrintParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_print_statement_trailing_comma() {
        let lexer = Lexer::new("print name, 42,;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = PrintParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::TrailingComma(1));
    }

    #[test]
    fn parse_print_statement_empty_arguments() {
        let lexer = Lexer::new("print ;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = PrintParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        // Since it expects an expression and finds a Semicolon, it will fail to parse the expression.
        assert_eq!(
            error,
            ParseError::UnsupportedPrefixExpression(TokenType::Semicolon, 1)
        );
    }

    #[test]
    fn parse_print_statement_unexpected_token_instead_of_semicolon() {
        let lexer = Lexer::new("print name loop", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = PrintParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::Semicolon, TokenType::Loop, 1)
        );
    }
}
