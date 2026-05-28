use crate::ast::expr::Expression;
use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::{ExpressionParser, InfixParser};

/// An infix parser that handles function call expressions like `f(a, b)`.
///
/// The `(` token is received as the infix operator token by the [`InfixParser::parse`] contract.
/// It parses comma-separated argument expressions and expects a closing `)`.
pub(crate) struct FunctionCallParser<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    expression_parser: &'expr mut ExpressionParser<'src, 'stream, I>,
}

impl<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>>
    FunctionCallParser<'expr, 'src, 'stream, I>
{
    /// Creates a new `FunctionCallParser` delegating to the given `ExpressionParser`.
    pub(crate) fn new(expression_parser: &'expr mut ExpressionParser<'src, 'stream, I>) -> Self {
        Self { expression_parser }
    }
}

impl<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>> InfixParser<'src>
    for FunctionCallParser<'expr, 'src, 'stream, I>
{
    fn parse(&mut self, left: Expression, _token: &Token<'src>) -> Result<Expression, ParseError> {
        let mut arguments = Vec::new();
        while let Some(next_token) = self.expression_parser.stream.peek()? {
            if next_token.token_type == TokenType::RightParentheses {
                break;
            }
            let argument = self.expression_parser.parse()?;
            arguments.push(argument);

            if !self
                .expression_parser
                .stream
                .maybe_matches(TokenType::Comma)
            {
                if let Some(after) = self.expression_parser.stream.peek()? {
                    if after.token_type != TokenType::RightParentheses {
                        return Err(ParseError::UnexpectedTokenType(
                            TokenType::RightParentheses,
                            after.token_type,
                            after.line,
                        ));
                    }
                }
            } else if let Some(after) = self.expression_parser.stream.peek()? {
                if after.token_type == TokenType::RightParentheses {
                    return Err(ParseError::TrailingComma(after.line));
                }
            }
        }

        self.expression_parser
            .stream
            .expect(TokenType::RightParentheses)?;
        Ok(Expression::FunctionCall(Box::new(left), arguments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::token::TokenType;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_call_with_no_arguments() {
        let lexer = Lexer::new(")", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("retrieve_data".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let expression = call_parser.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("retrieve_data".to_string())),
                vec![]
            )
        );
    }

    #[test]
    fn parse_call_with_single_argument() {
        let lexer = Lexer::new("income)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("calculate_tax".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let expression = call_parser.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("calculate_tax".to_string())),
                vec![Expression::Identifier("income".to_string())]
            )
        );
    }

    #[test]
    fn parse_call_with_multiple_arguments() {
        let lexer = Lexer::new("score, is_active)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("update_profile".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let expression = call_parser.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("update_profile".to_string())),
                vec![
                    Expression::Identifier("score".to_string()),
                    Expression::Identifier("is_active".to_string())
                ]
            )
        );
    }

    #[test]
    fn parse_call_missing_comma() {
        let lexer = Lexer::new("risk_level status)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("process_application".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let error = call_parser.parse(left, &token).unwrap_err();

        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::RightParentheses, TokenType::Identifier, 1)
        );
    }

    #[test]
    fn parse_call_with_expression_arguments() {
        let lexer = Lexer::new("45, one + other)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("greater_of".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let expression = call_parser.parse(left, &token).unwrap();

        assert_eq!(
            expression,
            Expression::FunctionCall(
                Box::new(Expression::Identifier("greater_of".to_string())),
                vec![
                    Expression::I32(45),
                    Expression::Binary(
                        Box::new(Expression::Identifier("one".to_string())),
                        crate::ast::expr::BinaryOperator::Plus,
                        Box::new(Expression::Identifier("other".to_string()))
                    )
                ]
            )
        );
    }

    #[test]
    fn parse_call_argument_syntax_error() {
        let lexer = Lexer::new("*)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("calculate".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let error = call_parser.parse(left, &token).unwrap_err();

        assert!(matches!(
            error,
            ParseError::UnsupportedPrefixExpression(TokenType::Star, 1)
        ));
    }

    #[test]
    fn parse_call_lex_error_in_argument() {
        let lexer = Lexer::new("?)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("calculate".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let error = call_parser.parse(left, &token).unwrap_err();

        assert!(matches!(
            error,
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?', 1))
        ));
    }

    #[test]
    fn parse_call_lex_error_after_argument() {
        let lexer = Lexer::new("a ?", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("calculate".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let error = call_parser.parse(left, &token).unwrap_err();

        assert!(matches!(
            error,
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?', 1))
        ));
    }

    #[test]
    fn parse_call_trailing_comma_single_argument() {
        let lexer = Lexer::new("income,)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("calculate_tax".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let error = call_parser.parse(left, &token).unwrap_err();

        assert_eq!(error, ParseError::TrailingComma(1));
    }

    #[test]
    fn parse_call_trailing_comma_multiple_arguments() {
        let lexer = Lexer::new("score, is_active,)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut call_parser = FunctionCallParser::new(&mut parser);

        let left = Expression::Identifier("update_profile".to_string());
        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let error = call_parser.parse(left, &token).unwrap_err();

        assert_eq!(error, ParseError::TrailingComma(1));
    }
}
