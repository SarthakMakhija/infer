use crate::ast::expr::{Expression, ExpressionKind};
use crate::ast::statement::{Statement, VariableDeclaration};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::stream::ParserStream;

/// A sub-parser responsible for parsing variable declaration statements.
///
/// See [grammar.ebnf](../../docs/grammar.ebnf) for the full language grammar.
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
        let var_token = self.stream.expect(TokenType::Var)?;
        let variable_name = self.stream.expect_identifier()?;
        let data_type = self.maybe_datatype()?;
        let expression_kind = self.maybe_expression_kind()?;
        self.stream.expect(TokenType::Semicolon)?;

        let expression = expression_kind.map(|kind| Expression::new(kind, var_token.line));
        Ok(Statement::variable_declaration(VariableDeclaration::new(
            variable_name.owned_value(),
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

    fn maybe_expression_kind(&mut self) -> Result<Option<ExpressionKind>, ParseError> {
        if self.stream.maybe_matches(TokenType::Equals) {
            let mut expression_parser = ExpressionParser::new(self.stream);
            let expression_kind = expression_parser.parse()?;
            return Ok(Some(expression_kind));
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
        let line = 1;

        assert_eq!(
            statement,
            variable_declaration!("id", type: "i32", value: expression_i32!(100, line))
        );
    }

    #[test]
    fn parse_variable_declaration_without_type_with_expr() {
        let lexer = Lexer::new("var greeting = \"hello\";", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        assert_eq!(
            statement,
            variable_declaration!("greeting", value: expression_string!("hello", line))
        );
    }

    #[test]
    fn parse_variable_declaration_with_type_without_expr() {
        let lexer = Lexer::new("var age: i32;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(statement, variable_declaration!("age", type: "i32"));
    }

    #[test]
    fn parse_variable_declaration_without_type_without_expr() {
        let lexer = Lexer::new("var flag;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(statement, variable_declaration!("flag"));
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
        let line = 1;

        assert_eq!(
            statement,
            variable_declaration!(
                "total",
                value: expression_binary!(
                    expression_identifier!("amount"),
                    Plus,
                    expression_identifier!("interest"),
                    line
                )
            )
        );
    }

    #[test]
    fn parse_variable_declaration_missing_type_identifier() {
        let lexer = Lexer::new("var attempts: = 10;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::Identifier, TokenType::Equals, 1)
        );
    }

    #[test]
    fn parse_variable_declaration_immediate_eof() {
        let lexer = Lexer::new("var", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = VariableDeclarationParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }
}
