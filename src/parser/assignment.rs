use crate::ast::expr::{Expression, ExpressionKind};
use crate::ast::statement::{Assignment, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::stream::ParserStream;

/// A sub-parser responsible for parsing variable assignment statements.
///
/// See [grammar.ebnf](../../docs/grammar.ebnf) for the full language grammar.
pub(crate) struct AssignmentParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> AssignmentParser<'src, 'stream, I> {
    /// Creates a new `AssignmentParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses a variable assignment from the token stream.
    ///
    /// Expects an identifier, followed by `=`, an expression, and a trailing `;`.
    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        let name = self.stream.expect_identifier()?;
        self.stream.expect(TokenType::Equals)?;

        let expression_kind = self.expression_kind()?;
        self.stream.expect(TokenType::Semicolon)?;

        let expression = Expression::new(expression_kind, name.line);
        Ok(Statement::assignment(Assignment::new(
            name.owned_value(),
            expression,
        )))
    }

    fn expression_kind(&mut self) -> Result<ExpressionKind, ParseError> {
        let mut expression_parser = ExpressionParser::new(self.stream);
        let expression_kind = expression_parser.parse()?;
        Ok(expression_kind)
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
            Statement::assignment(Assignment::new(
                "id".to_string(),
                Expression::new(ExpressionKind::I32(20), 1)
            ))
        );
    }

    #[test]
    fn parse_assignment_with_complex_expression() {
        let lexer = Lexer::new("total = amount + rate * percentage;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);
        let line = 1;

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::assignment(Assignment::new(
                "total".to_string(),
                Expression::new(
                    ExpressionKind::Binary(
                        Box::new(ExpressionKind::identifier("amount".to_string())),
                        crate::ast::expr::BinaryOperator::Plus,
                        Box::new(ExpressionKind::Binary(
                            Box::new(ExpressionKind::identifier("rate".to_string())),
                            crate::ast::expr::BinaryOperator::Multiply,
                            Box::new(ExpressionKind::identifier("percentage".to_string()))
                        ))
                    ),
                    line
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

    #[test]
    fn parse_assignment_missing_equals() {
        let lexer = Lexer::new("attempts 10;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::Equals, TokenType::WholeNumber, 1)
        );
    }

    #[test]
    fn parse_assignment_invalid_start() {
        let lexer = Lexer::new("10 = attempts;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::Identifier, TokenType::WholeNumber, 1)
        );
    }
}
