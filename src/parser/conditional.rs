use crate::ast::expr::Expression;
use crate::ast::statement::{Conditional, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::statement::StatementParser;
use crate::parser::stream::ParserStream;

pub(crate) struct ConditionalParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> ConditionalParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        let (condition, body) = self.parse_if()?;
        let else_body = self.maybe_parse_else()?;

        Ok(Statement::conditional(Conditional::new(
            condition, body, else_body,
        )))
    }

    fn parse_if(&mut self) -> Result<(Expression, Vec<Statement>), ParseError> {
        self.stream.expect(TokenType::If)?;
        let condition = ExpressionParser::new(self.stream).parse()?;
        self.stream.expect(TokenType::LeftBrace)?;
        let body = self.parse_body()?;
        self.stream.expect(TokenType::RightBrace)?;
        Ok((condition, body))
    }

    fn maybe_parse_else(&mut self) -> Result<Option<Vec<Statement>>, ParseError> {
        let mut else_body = None;
        if self.stream.maybe_matches(TokenType::Else) {
            else_body = Some(self.parse_else()?);
        }
        Ok(else_body)
    }

    fn parse_else(&mut self) -> Result<Vec<Statement>, ParseError> {
        self.stream.expect(TokenType::LeftBrace)?;
        let body = self.parse_body()?;
        self.stream.expect(TokenType::RightBrace)?;
        Ok(body)
    }

    fn parse_body(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut body = Vec::new();
        while let Some(next_token) = self.stream.peek()? {
            if next_token.token_type == TokenType::RightBrace {
                break;
            }
            let statement = StatementParser::new(self.stream).parse()?;
            body.push(statement);
        }
        Ok(body)
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
        let mut parser = ConditionalParser::new(&mut stream);

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
                ))],
                None
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
        let mut parser = ConditionalParser::new(&mut stream);

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
                ],
                None
            ))
        );
    }

    #[test]
    fn parse_conditional_with_empty_body() {
        let lexer = Lexer::new("if debug_mode_enabled == true {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::conditional(Conditional::new(
                Expression::Binary(
                    Box::new(Expression::Identifier("debug_mode_enabled".to_string())),
                    BinaryOperator::EqualsEquals,
                    Box::new(Expression::Boolean(true)),
                ),
                vec![],
                None
            ))
        );
    }

    #[test]
    fn parse_conditional_with_else() {
        let lexer = Lexer::new(
            "if total_price > budget { status = \"over_budget\"; } else { status = \"within_budget\"; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::conditional(Conditional::new(
                Expression::Binary(
                    Box::new(Expression::Identifier("total_price".to_string())),
                    BinaryOperator::GreaterThan,
                    Box::new(Expression::Identifier("budget".to_string())),
                ),
                vec![Statement::assignment(Assignment::new(
                    "status".to_string(),
                    Expression::String("over_budget".to_string()),
                ))],
                Some(vec![Statement::assignment(Assignment::new(
                    "status".to_string(),
                    Expression::String("within_budget".to_string()),
                ))])
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
        let mut parser = ConditionalParser::new(&mut stream);

        let result = parser.parse();
        assert!(matches!(
            result.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::LeftBrace, TokenType::Identifier, 1)
        ));
    }
}
