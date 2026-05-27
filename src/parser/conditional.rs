use crate::ast::statement::{Conditional, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::statement::StatementParser;
use crate::parser::stream::ParserStream;

pub(crate) struct IfParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> IfParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::If)?;
        let condition = ExpressionParser::new(self.stream).parse()?;

        self.stream.expect(TokenType::LeftBrace)?;

        let mut body = Vec::new();
        while let Some(next_token) = self.stream.peek()? {
            if next_token.token_type == TokenType::RightBrace {
                break;
            }
            let statement = StatementParser::new(self.stream).parse()?;
            body.push(statement);
        }
        self.stream.expect(TokenType::RightBrace)?;

        Ok(Statement::conditional(Conditional::new(condition, body)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{BinaryOperator, Expression};
    use crate::ast::statement::{Assignment, Statement};
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_conditional_with_single_statement() {
        let lexer = Lexer::new(
            "if score >= minimum_score { is_eligible = true; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = IfParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::conditional(Conditional::new(
                Expression::Binary(
                    Box::new(Expression::Identifier("score".to_string())),
                    BinaryOperator::GreaterThanEquals,
                    Box::new(Expression::Identifier("minimum_score".to_string())),
                ),
                vec![Statement::assignment(Assignment::new(
                    "is_eligible".to_string(),
                    Expression::Boolean(true),
                ))]
            ))
        );
    }

    #[test]
    fn parse_conditional_with_multiple_statements() {
        let lexer = Lexer::new(
            "if total_price > budget { status = \"over_budget\"; charge = base_price + excess_fee; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = IfParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::conditional(Conditional::new(
                Expression::Binary(
                    Box::new(Expression::Identifier("total_price".to_string())),
                    BinaryOperator::GreaterThan,
                    Box::new(Expression::Identifier("budget".to_string())),
                ),
                vec![
                    Statement::assignment(Assignment::new(
                        "status".to_string(),
                        Expression::String("over_budget".to_string()),
                    )),
                    Statement::assignment(Assignment::new(
                        "charge".to_string(),
                        Expression::Binary(
                            Box::new(Expression::Identifier("base_price".to_string())),
                            BinaryOperator::Plus,
                            Box::new(Expression::Identifier("excess_fee".to_string())),
                        ),
                    )),
                ]
            ))
        );
    }

    #[test]
    fn parse_conditional_with_empty_body() {
        let lexer = Lexer::new("if debug_mode_enabled == true {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = IfParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::conditional(Conditional::new(
                Expression::Binary(
                    Box::new(Expression::Identifier("debug_mode_enabled".to_string())),
                    BinaryOperator::EqualsEquals,
                    Box::new(Expression::Boolean(true)),
                ),
                vec![]
            ))
        );
    }

    #[test]
    fn parse_conditional_missing_left_brace() {
        let lexer = Lexer::new(
            "if score >= minimum_score is_eligible = true; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = IfParser::new(&mut stream);

        let result = parser.parse();
        assert!(matches!(
            result.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::LeftBrace, TokenType::Identifier, 1)
        ));
    }
}
