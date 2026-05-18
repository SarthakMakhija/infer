use crate::ast::program::{Program, ProgramBuilder};
use crate::ast::statement::Statement;
use crate::lexer::token::Token;
use crate::lexer::LexResult;
use crate::parser::declaration::VariableDeclarationParser;
use crate::parser::error::ParseError;
use crate::parser::stream::ParserStream;

pub(crate) mod declaration;
pub(crate) mod error;
pub(crate) mod expression;
pub(crate) mod stream;

pub(crate) struct Parser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> Parser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Program, ParseError> {
        let mut builder = ProgramBuilder::new();
        while let Some(token_ref) = self.stream.peek()? {
            let token = token_ref.clone();
            let statement = self.statement_beginning_at(&token)?;
            builder = builder.add(statement);
        }
        Ok(builder.build())
    }

    fn statement_beginning_at(&mut self, token: &Token) -> Result<Statement, ParseError> {
        let statement = match token {
            token if token.is_var() => VariableDeclarationParser::new(self.stream).parse()?,
            _ => unimplemented!(),
        };
        Ok(statement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expression;
    use crate::ast::statement::VariableDeclaration;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn parse_empty_program() {
        let lexer = Lexer::new("", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        assert_eq!(program, ProgramBuilder::new().build());
    }

    #[test]
    fn parse_single_variable_declaration() {
        let lexer = Lexer::new("var greeting = \"hello\";", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "greeting".to_string(),
                None,
                Some(Expression::String("hello".to_string())),
            )))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_multiple_variable_declarations() {
        let lexer = Lexer::new("var x = 100; var flag = true;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "x".to_string(),
                None,
                Some(Expression::I32(100)),
            )))
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "flag".to_string(),
                None,
                Some(Expression::Boolean(true)),
            )))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_lex_error() {
        let lexer = Lexer::new("var x = 100; ?", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let res = parser.parse();
        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?', 1))
        ));
    }
}
