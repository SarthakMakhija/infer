pub(crate) mod infix;
pub(crate) mod precedence;
pub(crate) mod prefix;

use crate::ast::expr::Expression;
use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::infix::binary::BinaryExpressionParser;
use crate::parser::expr::precedence::Precedence;
use crate::parser::expr::prefix::boolean::BooleanParser;
use crate::parser::expr::prefix::group::GroupParser;
use crate::parser::expr::prefix::identifier::IdentifierParser;
use crate::parser::expr::prefix::string::StringParser;
use crate::parser::expr::prefix::unary::UnaryExpressionParser;
use crate::parser::expr::prefix::whole_number::WholeNumberParser;
use crate::parser::stream::ParserStream;

pub(crate) struct ExpressionParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

pub(crate) trait PrefixParser<'src> {
    fn parse(&mut self, token: &Token<'src>) -> Result<Expression, ParseError>;
}

pub(crate) trait InfixParser<'src> {
    fn parse(&mut self, left: Expression, token: &Token<'src>) -> Result<Expression, ParseError>;
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> ExpressionParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Expression, ParseError> {
        self.parse_with_precedence(Precedence::None)
    }

    pub(crate) fn parse_with_precedence(
        &mut self,
        precedence: Precedence,
    ) -> Result<Expression, ParseError> {
        let token = self.stream.expect_token()?;
        let mut left = self.parse_prefix(&token)?;

        while precedence < self.precedence()? {
            let token = self.stream.expect_token()?;
            left = self.parse_infix(left, &token)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self, token: &Token<'src>) -> Result<Expression, ParseError> {
        match token.token_type {
            TokenType::Identifier => IdentifierParser.parse(token),
            TokenType::WholeNumber => WholeNumberParser.parse(token),
            TokenType::StringLiteral => StringParser.parse(token),
            TokenType::BooleanLiteral(_) => BooleanParser.parse(token),
            TokenType::Minus | TokenType::Bang => UnaryExpressionParser::new(self).parse(token),
            TokenType::LeftParentheses => GroupParser::new(self).parse(token),
            _ => Err(ParseError::UnsupportedPrefixExpression(
                token.token_type,
                token.line,
            )),
        }
    }

    fn parse_infix(
        &mut self,
        left: Expression,
        token: &Token<'src>,
    ) -> Result<Expression, ParseError> {
        //TODO: missing '(' for function call.
        match token.token_type {
            TokenType::Plus | TokenType::Minus => {
                BinaryExpressionParser::new(self, Precedence::Plus).parse(left, token)
            }
            TokenType::Star | TokenType::Slash => {
                BinaryExpressionParser::new(self, Precedence::Star).parse(left, token)
            }
            _ => Err(ParseError::UnsupportedInfixExpression(
                token.token_type,
                token.line,
            )),
        }
    }

    fn precedence(&mut self) -> Result<Precedence, ParseError> {
        if let Some(result) = self.stream.peek().transpose() {
            let token = result?;
            return Ok(Precedence::of(token.token_type));
        }
        Ok(Precedence::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::token::TokenType;
    use crate::lexer::Lexer;

    #[test]
    fn parse_whole_number() {
        let lexer = Lexer::new("123", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(expr, Expression::I32(123));
    }

    #[test]
    fn parse_string_literal() {
        let lexer = Lexer::new("\"infer\"", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(expr, Expression::String("infer".to_string()));
    }

    #[test]
    fn parse_identifier() {
        let lexer = Lexer::new("my_var", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(expr, Expression::Identifier("my_var".to_string()));
    }

    #[test]
    fn parse_unsupported_token_as_expression() {
        let lexer = Lexer::new("var", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let res = parser.parse();
        assert_eq!(
            res.err().unwrap(),
            ParseError::UnsupportedPrefixExpression(TokenType::Var, 1)
        );
    }

    #[test]
    fn parse_eof() {
        let lexer = Lexer::new("", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let res = parser.parse();
        assert_eq!(res.err().unwrap(), ParseError::UnexpectedEof);
    }
}

#[cfg(test)]
mod complex_expression_tests {
    use super::*;
    use crate::ast::expr::BinaryOperator;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_complex_addition_and_subtraction() {
        let lexer = Lexer::new("1 + 3 - 2", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Binary(
                Box::new(Expression::Binary(
                    Box::new(Expression::I32(1)),
                    BinaryOperator::Plus,
                    Box::new(Expression::I32(3))
                )),
                BinaryOperator::Minus,
                Box::new(Expression::I32(2))
            )
        );
    }

    #[test]
    fn parse_complex_addition_and_multiplication() {
        let lexer = Lexer::new("1 + 2 * 4", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Binary(
                Box::new(Expression::I32(1)),
                BinaryOperator::Plus,
                Box::new(Expression::Binary(
                    Box::new(Expression::I32(2)),
                    BinaryOperator::Multiply,
                    Box::new(Expression::I32(4))
                ))
            )
        );
    }

    #[test]
    fn parse_expression_with_identifiers() {
        let lexer = Lexer::new("amount + factor * rate", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Binary(
                Box::new(Expression::Identifier("amount".to_string())),
                BinaryOperator::Plus,
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("factor".to_string())),
                    BinaryOperator::Multiply,
                    Box::new(Expression::Identifier("rate".to_string()))
                ))
            )
        );
    }

    #[test]
    fn parse_complex_division_and_subtraction() {
        let lexer = Lexer::new("10 / 2 - 3", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Binary(
                Box::new(Expression::Binary(
                    Box::new(Expression::I32(10)),
                    BinaryOperator::Divide,
                    Box::new(Expression::I32(2))
                )),
                BinaryOperator::Minus,
                Box::new(Expression::I32(3))
            )
        );
    }

    #[test]
    fn parse_unary_minus_expression() {
        let lexer = Lexer::new("-10", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Unary(
                Box::new(Expression::I32(10)),
                crate::ast::expr::UnaryOperator::Minus
            )
        );
    }

    #[test]
    fn parse_unary_bang_expression() {
        let lexer = Lexer::new("!true", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Unary(
                Box::new(Expression::Boolean(true)),
                crate::ast::expr::UnaryOperator::Negation
            )
        );
    }

    #[test]
    fn parse_unary_with_binary_precedence() {
        let lexer = Lexer::new("-amount + factor * rate", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Binary(
                Box::new(Expression::Unary(
                    Box::new(Expression::Identifier("amount".to_string())),
                    crate::ast::expr::UnaryOperator::Minus
                )),
                BinaryOperator::Plus,
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("factor".to_string())),
                    BinaryOperator::Multiply,
                    Box::new(Expression::Identifier("rate".to_string()))
                ))
            )
        );
    }

    #[test]
    fn parse_grouped_expression() {
        let lexer = Lexer::new("(1 + 2) * 3", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Binary(
                Box::new(Expression::Grouped(Box::new(Expression::Binary(
                    Box::new(Expression::I32(1)),
                    BinaryOperator::Plus,
                    Box::new(Expression::I32(2))
                )))),
                BinaryOperator::Multiply,
                Box::new(Expression::I32(3))
            )
        );
    }

    #[test]
    fn parse_nested_grouped_expression() {
        let lexer = Lexer::new("((1 + 2) * 3)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expression::Grouped(Box::new(Expression::Binary(
                Box::new(Expression::Grouped(Box::new(Expression::Binary(
                    Box::new(Expression::I32(1)),
                    BinaryOperator::Plus,
                    Box::new(Expression::I32(2))
                )))),
                BinaryOperator::Multiply,
                Box::new(Expression::I32(3))
            )))
        );
    }
}
