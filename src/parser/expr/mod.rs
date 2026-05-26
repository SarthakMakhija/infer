pub(crate) mod identifier;

use crate::ast::expr::Expression;
use crate::lexer::token::Token;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::stream::ParserStream;

pub(crate) struct ExpressionParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

pub(crate) trait PrefixRule<'src> {
    fn parse(&mut self, token: &'src Token) -> Result<Expression, ParseError>;
}

pub(crate) trait InfixRule<'src> {
    fn parse(&mut self, left: Expression, token: &'src Token) -> Result<Expression, ParseError>;
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> ExpressionParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Expression, ParseError> {
        let token = self.stream.expect_token()?;
        Ok(Expression::try_from(token)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::ExpressionError;
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
    fn parse_unsupported_token() {
        let lexer = Lexer::new("var", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ExpressionParser::new(&mut stream);

        let res = parser.parse();
        assert_eq!(
            res.err().unwrap(),
            ParseError::ExpressionError(ExpressionError::UnsupportedTokenType(TokenType::Var, 1))
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
