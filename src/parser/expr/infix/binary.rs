use crate::ast::expr::{BinaryOperator, Expression};
use crate::lexer::token::Token;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::precedence::Precedence;
use crate::parser::expr::{ExpressionParser, InfixParser};
use std::convert::TryInto;

pub(crate) struct BinaryExpressionParser<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>>
{
    expression_parser: &'expr mut ExpressionParser<'src, 'stream, I>,
    precedence: Precedence,
}

impl<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>>
    BinaryExpressionParser<'expr, 'src, 'stream, I>
{
    pub(crate) fn new(
        expression_parser: &'expr mut ExpressionParser<'src, 'stream, I>,
        precedence: Precedence,
    ) -> Self {
        Self {
            expression_parser,
            precedence,
        }
    }
}

impl<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>> InfixParser<'src>
    for BinaryExpressionParser<'expr, 'src, 'stream, I>
{
    fn parse(&mut self, left: Expression, token: &Token<'src>) -> Result<Expression, ParseError> {
        let operator: BinaryOperator = token.try_into()?;
        if operator.is_comparison() {
            if let Expression::Binary(_, ref left_operator, _) = left {
                if left_operator.is_comparison() {
                    return Err(ParseError::ChainedComparison(token.line));
                }
            }
        }
        let right = self
            .expression_parser
            .parse_with_precedence(self.precedence)?;

        Ok(Expression::Binary(
            Box::new(left),
            operator,
            Box::new(right),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::BinaryOperator;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::token::TokenType;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_binary_operator_plus() {
        let lexer = Lexer::new("2", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut binary_operator = BinaryExpressionParser::new(&mut parser, Precedence::Plus);

        let left = Expression::I32(1);
        let token = Token::new(TokenType::Plus, 0..1, 1, "+");
        let expression = binary_operator.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::I32(1)),
                BinaryOperator::Plus,
                Box::new(Expression::I32(2))
            )
        );
    }

    #[test]
    fn parse_binary_operator_minus() {
        let lexer = Lexer::new("5", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut binary_operator = BinaryExpressionParser::new(&mut parser, Precedence::Plus);

        let left = Expression::I32(10);
        let token = Token::new(TokenType::Minus, 0..1, 1, "-");
        let expression = binary_operator.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::I32(10)),
                BinaryOperator::Minus,
                Box::new(Expression::I32(5))
            )
        );
    }

    #[test]
    fn parse_binary_operator_multiply() {
        let lexer = Lexer::new("4", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut binary_operator = BinaryExpressionParser::new(&mut parser, Precedence::Star);

        let left = Expression::I32(3);
        let token = Token::new(TokenType::Star, 0..1, 1, "*");
        let expression = binary_operator.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::I32(3)),
                BinaryOperator::Multiply,
                Box::new(Expression::I32(4))
            )
        );
    }

    #[test]
    fn parse_binary_operator_divide() {
        let lexer = Lexer::new("4", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut binary_operator = BinaryExpressionParser::new(&mut parser, Precedence::Star);

        let left = Expression::I32(20);
        let token = Token::new(TokenType::Slash, 0..1, 1, "/");
        let expression = binary_operator.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::I32(20)),
                BinaryOperator::Divide,
                Box::new(Expression::I32(4))
            )
        );
    }

    #[test]
    fn attempt_to_parse_binary_operator_invalid_operator() {
        let lexer = Lexer::new("2", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut binary_operator = BinaryExpressionParser::new(&mut parser, Precedence::Plus);

        let left = Expression::I32(1);
        let token = Token::new(TokenType::Identifier, 0..4, 1, "name");
        let result = binary_operator.parse(left, &token);

        assert_eq!(
            result.err().unwrap(),
            ParseError::ExpressionError(crate::ast::expr::ExpressionError::UnsupportedOperator(
                TokenType::Identifier,
                1
            ))
        );
    }

    #[test]
    fn parse_chained_comparison_error() {
        let lexer = Lexer::new("score1 < score2 < score3", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let result = parser.parse();
        assert!(matches!(
            result.err().unwrap(),
            ParseError::ChainedComparison(1)
        ));
    }

    #[test]
    fn parse_binary_operator_comparison() {
        let lexer = Lexer::new("2", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut binary_operator = BinaryExpressionParser::new(&mut parser, Precedence::Comparison);

        let left = Expression::I32(1);
        let token = Token::new(TokenType::LessThan, 0..1, 1, "<");
        let expression = binary_operator.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::I32(1)),
                BinaryOperator::LessThan,
                Box::new(Expression::I32(2))
            )
        );
    }

    #[test]
    fn parse_binary_operator_right_side_error() {
        let lexer = Lexer::new("", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut binary_operator = BinaryExpressionParser::new(&mut parser, Precedence::Plus);

        let left = Expression::I32(1);
        let token = Token::new(TokenType::Plus, 0..1, 1, "+");
        let result = binary_operator.parse(left, &token);

        assert_eq!(result.err().unwrap(), ParseError::UnexpectedEof);
    }
}
