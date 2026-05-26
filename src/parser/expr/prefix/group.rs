use crate::ast::expr::Expression;
use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::{ExpressionParser, PrefixParser};

pub(crate) struct GroupParser<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    expression_parser: &'expr mut ExpressionParser<'src, 'stream, I>,
}

impl<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>>
    GroupParser<'expr, 'src, 'stream, I>
{
    pub(crate) fn new(expression_parser: &'expr mut ExpressionParser<'src, 'stream, I>) -> Self {
        Self { expression_parser }
    }
}

impl<'expr, 'src, 'stream, I: Iterator<Item = LexResult<'src>>> PrefixParser<'src>
    for GroupParser<'expr, 'src, 'stream, I>
{
    fn parse(&mut self, _token: &Token<'src>) -> Result<Expression, ParseError> {
        let expression = self.expression_parser.parse()?;
        self.expression_parser
            .stream
            .expect(TokenType::RightParentheses)?;
        Ok(Expression::Grouped(Box::new(expression)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_valid_group() {
        let lexer = Lexer::new("123)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut group_parser = GroupParser::new(&mut parser);

        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let expression = group_parser.parse(&token).unwrap();

        assert_eq!(
            expression,
            Expression::Grouped(Box::new(Expression::I32(123)))
        );
    }

    #[test]
    fn parse_group_missing_closing_parenthesis_eof() {
        let lexer = Lexer::new("123", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut group_parser = GroupParser::new(&mut parser);

        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let result = group_parser.parse(&token);

        assert_eq!(result.err().unwrap(), ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_group_mismatched_closing_parenthesis() {
        let lexer = Lexer::new("123;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);
        let mut group_parser = GroupParser::new(&mut parser);

        let token = Token::new(TokenType::LeftParentheses, 0..1, 1, "(");
        let result = group_parser.parse(&token);

        assert_eq!(
            result.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::RightParentheses, TokenType::Semicolon, 1)
        );
    }
}
