use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::ast::expr::Expression;
use crate::parser::ast::AssignmentNode;
use crate::parser::error::ParseError;
use crate::parser::expression::ExpressionParser;
use crate::parser::ParserStream;

pub(crate) struct AssignmentParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> AssignmentParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<AssignmentNode, ParseError> {
        self.stream.expect(TokenType::Var)?;
        let name = self.stream.expect_identifier()?;
        let data_type = self.maybe_datatype()?;
        let expression = self.maybe_expression()?;
        self.stream.expect(TokenType::Semicolon)?;

        Ok(AssignmentNode::new(
            name.owned_value(),
            data_type,
            expression,
        ))
    }

    fn maybe_datatype(&mut self) -> Result<Option<String>, ParseError> {
        if self.stream.maybe_matches(TokenType::Colon) {
            let data_type = self.stream.expect_identifier()?;
            return Ok(Some(data_type.owned_value()));
        }
        Ok(None)
    }

    fn maybe_expression(&mut self) -> Result<Option<Expression>, ParseError> {
        if self.stream.maybe_matches(TokenType::Equals) {
            let mut expression_parser = ExpressionParser::new(self.stream);
            let expression = expression_parser.parse()?;
            return Ok(Some(expression));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn parse_assignment_with_type_and_expr() {
        let lexer = Lexer::new("var id: i32 = 100;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let node = parser.parse().unwrap();
        assert_eq!(node.variable, "id");
        assert_eq!(node.data_type, Some("i32".to_string()));
        assert_eq!(node.expression, Some(Expression::I32(100)));
    }

    #[test]
    fn parse_assignment_without_type_with_expr() {
        let lexer = Lexer::new("var greeting = \"hello\";", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let node = parser.parse().unwrap();
        assert_eq!(node.variable, "greeting");
        assert_eq!(node.data_type, None);
        assert_eq!(
            node.expression,
            Some(Expression::String("hello".to_string()))
        );
    }

    #[test]
    fn parse_assignment_with_type_without_expr() {
        let lexer = Lexer::new("var age: i32;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let node = parser.parse().unwrap();
        assert_eq!(node.variable, "age");
        assert_eq!(node.data_type, Some("i32".to_string()));
        assert_eq!(node.expression, None);
    }

    #[test]
    fn parse_assignment_without_type_without_expr() {
        let lexer = Lexer::new("var flag;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let node = parser.parse().unwrap();
        assert_eq!(node.variable, "flag");
        assert_eq!(node.data_type, None);
        assert_eq!(node.expression, None);
    }

    #[test]
    fn parse_assignment_missing_semicolon() {
        let lexer = Lexer::new("var id = 100", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let res = parser.parse();
        assert_eq!(res.err().unwrap(), ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_assignment_missing_variable() {
        let lexer = Lexer::new("var = 100;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = AssignmentParser::new(&mut stream);

        let res = parser.parse();
        assert_eq!(
            res.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::Identifier, TokenType::Equals)
        );
    }
}
