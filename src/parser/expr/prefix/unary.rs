use crate::ast::expr::ExpressionKind;
use crate::lexer::token::Token;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::precedence::Precedence;
use crate::parser::expr::{ExpressionParser, PrefixParser};

/// A prefix parser that handles unary expressions such as `-x` and `!flag`.
///
/// Parses the operand expression at `Precedence::Unary` so that the operand
/// binds tightly (e.g., `-a * b` is `(-a) * b`, not `-(a * b)`).
pub(crate) struct UnaryExpressionParser<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    expression_parser: &'expr mut ExpressionParser<'src, 'stream, I>,
}

impl<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>>
    UnaryExpressionParser<'expr, 'src, 'stream, I>
{
    /// Creates a new `UnaryExpressionParser` delegating to the given `ExpressionParser`.
    pub(crate) fn new(expression_parser: &'expr mut ExpressionParser<'src, 'stream, I>) -> Self {
        Self { expression_parser }
    }
}

impl<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>> PrefixParser<'src>
    for UnaryExpressionParser<'expr, 'src, 'stream, I>
{
    fn parse(&mut self, token: &Token<'src>) -> Result<ExpressionKind, ParseError> {
        let expression = self
            .expression_parser
            .parse_with_precedence(Precedence::Unary)?;
        let operator = token.try_into()?;
        Ok(ExpressionKind::Unary(Box::new(expression), operator))
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
    fn parse_unary_minus() {
        let lexer = Lexer::new("10", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut unary_parser = UnaryExpressionParser::new(&mut parser);

        let token = Token::new(TokenType::Minus, 0..1, 1, "-");
        let expression = unary_parser.parse(&token).unwrap();

        assert_eq!(
            expression,
            ExpressionKind::Unary(
                Box::new(ExpressionKind::I32(10)),
                crate::ast::expr::UnaryOperator::Minus
            )
        );
    }

    #[test]
    fn parse_unary_negation() {
        let lexer = Lexer::new("true", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut unary_parser = UnaryExpressionParser::new(&mut parser);

        let token = Token::new(TokenType::Bang, 0..1, 1, "!");
        let expression = unary_parser.parse(&token).unwrap();

        assert_eq!(
            expression,
            ExpressionKind::Unary(
                Box::new(ExpressionKind::Boolean(true)),
                crate::ast::expr::UnaryOperator::Negation
            )
        );
    }

    #[test]
    fn parse_unary_invalid_operator() {
        let lexer = Lexer::new("10", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut unary_parser = UnaryExpressionParser::new(&mut parser);

        let token = Token::new(TokenType::Identifier, 0..4, 1, "name");
        let result = unary_parser.parse(&token);

        assert_eq!(
            result.err().unwrap(),
            ParseError::ExpressionError(crate::ast::expr::ExpressionError::UnsupportedOperator(
                TokenType::Identifier,
                1
            ))
        );
    }
}
