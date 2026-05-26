pub(crate) mod precedence;
pub(crate) mod prefix;

use crate::ast::expr::Expression;
use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::precedence::Precedence;
use crate::parser::expr::prefix::boolean::Boolean;
use crate::parser::expr::prefix::identifier::Identifier;
use crate::parser::expr::prefix::string::String;
use crate::parser::expr::prefix::whole_number::WholeNumber;
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
        self.parse_with_precedence(Precedence::None)
    }

    pub(crate) fn parse_with_precedence(
        &mut self,
        _precedence: Precedence,
    ) -> Result<Expression, ParseError> {
        let token = self.stream.expect_token()?;
        let left = self.parse_prefix(&token)?;
        Ok(left)
    }

    fn parse_prefix(&self, token: &Token<'src>) -> Result<Expression, ParseError> {
        //TODO: missing '-' (minus) and '(' (open parentheses)
        match token.token_type {
            TokenType::Identifier => Identifier.parse(token),
            TokenType::WholeNumber => WholeNumber.parse(token),
            TokenType::StringLiteral => String.parse(token),
            TokenType::BooleanLiteral(_) => Boolean.parse(token),
            _ => Err(ParseError::UnsupportedPrefixExpression(
                token.token_type,
                token.line,
            )),
        }
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
