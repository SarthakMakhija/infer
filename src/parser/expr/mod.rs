pub(crate) mod infix;
pub(crate) mod precedence;
pub(crate) mod prefix;

use crate::ast::expr::Expression;
use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::infix::binary::BinaryExpressionParser;
use crate::parser::expr::infix::call::FunctionCallParser;
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
        match token.token_type {
            TokenType::Plus | TokenType::Minus => {
                BinaryExpressionParser::new(self, Precedence::Plus).parse(left, token)
            }
            TokenType::Star | TokenType::Slash => {
                BinaryExpressionParser::new(self, Precedence::Star).parse(left, token)
            }
            TokenType::EqualsEquals | TokenType::BangEquals => {
                BinaryExpressionParser::new(self, Precedence::Equality).parse(left, token)
            }
            TokenType::GreaterThan
            | TokenType::GreaterThanEquals
            | TokenType::LessThan
            | TokenType::LessThanEquals => {
                BinaryExpressionParser::new(self, Precedence::Comparison).parse(left, token)
            }
            TokenType::And => BinaryExpressionParser::new(self, Precedence::And).parse(left, token),
            TokenType::Or => BinaryExpressionParser::new(self, Precedence::Or).parse(left, token),
            TokenType::LeftParentheses => FunctionCallParser::new(self).parse(left, token),
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

        let expression = parser.parse().unwrap();
        assert_eq!(expression, Expression::I32(123));
    }

    #[test]
    fn parse_string_literal() {
        let lexer = Lexer::new("\"infer\"", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(expression, Expression::String("infer".to_string()));
    }

    #[test]
    fn parse_identifier() {
        let lexer = Lexer::new("my_var", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(expression, Expression::Identifier("my_var".to_string()));
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
mod arithmetic_expression_tests {
    use super::*;
    use crate::ast::expr::BinaryOperator;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_addition_and_subtraction() {
        let lexer = Lexer::new("1 + 3 - 2", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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
    fn parse_addition_and_multiplication() {
        let lexer = Lexer::new("1 + 2 * 4", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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
    fn parse_division_and_subtraction() {
        let lexer = Lexer::new("10 / 2 - 3", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
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

#[cfg(test)]
mod comparison_expression_tests {
    use super::*;
    use crate::ast::expr::BinaryOperator;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_equality_equals_equals() {
        let lexer = Lexer::new("status == active", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("status".to_string())),
                BinaryOperator::EqualsEquals,
                Box::new(Expression::Identifier("active".to_string()))
            )
        );
    }

    #[test]
    fn parse_equality_bang_equals() {
        let lexer = Lexer::new("status != disabled", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("status".to_string())),
                BinaryOperator::NotEquals,
                Box::new(Expression::Identifier("disabled".to_string()))
            )
        );
    }

    #[test]
    fn parse_comparison_greater_than() {
        let lexer = Lexer::new("score > passing_score", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("score".to_string())),
                BinaryOperator::GreaterThan,
                Box::new(Expression::Identifier("passing_score".to_string()))
            )
        );
    }

    #[test]
    fn parse_comparison_greater_than_equals() {
        let lexer = Lexer::new("score >= target_score", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("score".to_string())),
                BinaryOperator::GreaterThanEquals,
                Box::new(Expression::Identifier("target_score".to_string()))
            )
        );
    }

    #[test]
    fn parse_comparison_less_than() {
        let lexer = Lexer::new("weight < limit", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("weight".to_string())),
                BinaryOperator::LessThan,
                Box::new(Expression::Identifier("limit".to_string()))
            )
        );
    }

    #[test]
    fn parse_comparison_less_than_equals() {
        let lexer = Lexer::new("weight <= max_weight", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("weight".to_string())),
                BinaryOperator::LessThanEquals,
                Box::new(Expression::Identifier("max_weight".to_string()))
            )
        );
    }

    #[test]
    fn parse_comparison_and_arithmetic_precedence() {
        let lexer = Lexer::new("base_price + tax_rate > budget", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("base_price".to_string())),
                    BinaryOperator::Plus,
                    Box::new(Expression::Identifier("tax_rate".to_string()))
                )),
                BinaryOperator::GreaterThan,
                Box::new(Expression::Identifier("budget".to_string()))
            )
        );
    }

    #[test]
    fn parse_comparison_and_equality_precedence() {
        let lexer = Lexer::new(
            "adjusted_score == threshold_score > base_score",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("adjusted_score".to_string())),
                BinaryOperator::EqualsEquals,
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("threshold_score".to_string())),
                    BinaryOperator::GreaterThan,
                    Box::new(Expression::Identifier("base_score".to_string()))
                ))
            )
        );
    }
}

#[cfg(test)]
mod logical_expression_tests {
    use super::*;
    use crate::ast::expr::BinaryOperator;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_logical_and() {
        let lexer = Lexer::new("active and validated", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("active".to_string())),
                BinaryOperator::And,
                Box::new(Expression::Identifier("validated".to_string()))
            )
        );
    }

    #[test]
    fn parse_logical_or() {
        let lexer = Lexer::new("cached or retrieved", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("cached".to_string())),
                BinaryOperator::Or,
                Box::new(Expression::Identifier("retrieved".to_string()))
            )
        );
    }

    #[test]
    fn parse_logical_and_and_or_precedence() {
        // 'and' has higher precedence
        let lexer = Lexer::new("a and b or c", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("a".to_string())),
                    BinaryOperator::And,
                    Box::new(Expression::Identifier("b".to_string()))
                )),
                BinaryOperator::Or,
                Box::new(Expression::Identifier("c".to_string()))
            )
        );
    }

    #[test]
    fn parse_logical_and_and_comparison_precedence() {
        // Comparisons (40) have higher precedence
        let lexer = Lexer::new("a < b and c > d", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("a".to_string())),
                    BinaryOperator::LessThan,
                    Box::new(Expression::Identifier("b".to_string()))
                )),
                BinaryOperator::And,
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("c".to_string())),
                    BinaryOperator::GreaterThan,
                    Box::new(Expression::Identifier("d".to_string()))
                ))
            )
        );
    }

    #[test]
    fn parse_logical_or_and_equality_precedence() {
        // Equality (30) has higher precedence
        let lexer = Lexer::new("a == b or c != d", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("a".to_string())),
                    BinaryOperator::EqualsEquals,
                    Box::new(Expression::Identifier("b".to_string()))
                )),
                BinaryOperator::Or,
                Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("c".to_string())),
                    BinaryOperator::NotEquals,
                    Box::new(Expression::Identifier("d".to_string()))
                ))
            )
        );
    }

    #[test]
    fn parse_logical_precedence_with_grouping() {
        // Grouping overrides default precedence: a and (b or c)
        let lexer = Lexer::new("a and (b or c)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("a".to_string())),
                BinaryOperator::And,
                Box::new(Expression::Grouped(Box::new(Expression::Binary(
                    Box::new(Expression::Identifier("b".to_string())),
                    BinaryOperator::Or,
                    Box::new(Expression::Identifier("c".to_string()))
                ))))
            )
        );
    }
}

#[cfg(test)]
mod function_call_tests {
    use super::*;
    use crate::ast::expr::BinaryOperator;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_simple_function_call() {
        let lexer = Lexer::new("calculate_tax()", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("calculate_tax".to_string())),
                vec![]
            )
        );
    }

    #[test]
    fn parse_function_call_with_single_argument() {
        let lexer = Lexer::new("compute_grade(score)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("compute_grade".to_string())),
                vec![Expression::Identifier("score".to_string())]
            )
        );
    }

    #[test]
    fn parse_function_call_with_multiple_arguments() {
        let lexer = Lexer::new("adjust_salary(salary, rating)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("adjust_salary".to_string())),
                vec![
                    Expression::Identifier("salary".to_string()),
                    Expression::Identifier("rating".to_string())
                ]
            )
        );
    }

    #[test]
    fn parse_function_call_with_expression_arguments() {
        let lexer = Lexer::new("greater_of(45, base_price + tax_rate)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("greater_of".to_string())),
                vec![
                    Expression::I32(45),
                    Expression::Binary(
                        Box::new(Expression::Identifier("base_price".to_string())),
                        BinaryOperator::Plus,
                        Box::new(Expression::Identifier("tax_rate".to_string()))
                    )
                ]
            )
        );
    }

    #[test]
    fn parse_nested_function_calls() {
        let lexer = Lexer::new("get_discount(get_age(user_id))", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("get_discount".to_string())),
                vec![Expression::FunctionCall(
                    Box::new(Expression::Identifier("get_age".to_string())),
                    vec![Expression::Identifier("user_id".to_string())]
                )]
            )
        );
    }

    #[test]
    fn parse_function_call_in_arithmetic_expression() {
        let lexer = Lexer::new("rating + increment()", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("rating".to_string())),
                BinaryOperator::Plus,
                Box::new(Expression::FunctionCall(
                    Box::new(Expression::Identifier("increment".to_string())),
                    vec![]
                ))
            )
        );
    }

    #[test]
    fn parse_function_call_in_comparison_expression() {
        let lexer = Lexer::new("rating < expected()", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let expression = parser.parse().unwrap();
        assert_eq!(
            expression,
            Expression::Binary(
                Box::new(Expression::Identifier("rating".to_string())),
                BinaryOperator::LessThan,
                Box::new(Expression::FunctionCall(
                    Box::new(Expression::Identifier("expected".to_string())),
                    vec![]
                ))
            )
        );
    }
}
