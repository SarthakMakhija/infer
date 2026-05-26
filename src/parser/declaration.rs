use crate::ast::expr::Expression;
use crate::ast::statement::{Statement, VariableDeclaration};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::stream::ParserStream;

/// A sub-parser responsible for parsing variable declaration and assignment statements.
///
/// `VariableDeclarationParser` parses constructs following the grammar:
/// `declaration = "var" identifier [ ":" type ] [ "=" literal ] ";" ;`
pub(crate) struct VariableDeclarationParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>>
    VariableDeclarationParser<'src, 'stream, I>
{
    /// Creates a new instance of `VariableDeclarationParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses a variable declaration and assignment statements from the stream.
    ///
    /// Validates syntax constructs, handles optional type annotations and optional expressions,
    /// and ensures the statement is terminated with a semicolon.
    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::Var)?;
        let name = self.stream.expect_identifier()?;
        let data_type = self.maybe_datatype()?;
        let expression = self.maybe_expression()?;
        self.stream.expect(TokenType::Semicolon)?;

        Ok(Statement::variable_declaration(VariableDeclaration::new(
            name.owned_value(),
            data_type,
            expression,
        )))
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
    fn parse_variable_declaration_with_type_and_expr() {
        let lexer = Lexer::new("var id: i32 = 100;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let statement = parser.parse().unwrap();

        assert_eq!(
            statement,
            Statement::variable_declaration(VariableDeclaration::new(
                "id".to_string(),
                Some("i32".to_string()),
                Some(Expression::I32(100))
            ))
        );
    }

    #[test]
    fn parse_variable_declaration_without_type_with_expr() {
        let lexer = Lexer::new("var greeting = \"hello\";", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::variable_declaration(VariableDeclaration::new(
                "greeting".to_string(),
                None,
                Some(Expression::String("hello".to_string()))
            ))
        );
    }

    #[test]
    fn parse_variable_declaration_with_type_without_expr() {
        let lexer = Lexer::new("var age: i32;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::variable_declaration(VariableDeclaration::new(
                "age".to_string(),
                Some("i32".to_string()),
                None
            ))
        );
    }

    #[test]
    fn parse_variable_declaration_without_type_without_expr() {
        let lexer = Lexer::new("var flag;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::variable_declaration(VariableDeclaration::new(
                "flag".to_string(),
                None,
                None
            ))
        );
    }

    #[test]
    fn parse_variable_declaration_missing_semicolon() {
        let lexer = Lexer::new("var id = 100", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let res = parser.parse();
        assert_eq!(res.err().unwrap(), ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_variable_declaration_missing_variable() {
        let lexer = Lexer::new("var = 100;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let res = parser.parse();
        assert_eq!(
            res.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::Identifier, TokenType::Equals, 1)
        );
    }

    #[test]
    fn parse_variable_declaration_with_expression_on_rhs() {
        let lexer = Lexer::new("var total = amount + interest;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let statement = parser.parse().unwrap();

        assert_eq!(
            statement,
            Statement::variable_declaration(VariableDeclaration::new(
                "total".to_string(),
                None,
                Some(Expression::Binary(
                    Box::new(Expression::Identifier("amount".to_string())),
                    crate::ast::expr::BinaryOperator::Plus,
                    Box::new(Expression::Identifier("interest".to_string()))
                ))
            ))
        );
    }
}
