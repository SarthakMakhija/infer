use crate::ast::program::{Program, ProgramBuilder};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::declaration::VariableDeclarationParser;
use crate::parser::error::ParseError;
use crate::parser::function::FnParser;
use crate::parser::stream::ParserStream;

pub(crate) mod assignment;
pub(crate) mod block;
pub(crate) mod conditional;
pub(crate) mod control_flow;
pub(crate) mod declaration;
pub(crate) mod error;
pub(crate) mod expr;
pub(crate) mod function;
pub(crate) mod iteration;
pub(crate) mod statement;
pub(crate) mod stream;

/// The top-level parser for the `infer` language.
///
/// `Parser` reads from a `ParserStream` and constructs a fully-parsed [`Program`].
/// It only allows `var` declarations and `fn` definitions at the top level;
/// any other statement token results in a `ParseError::UnsupportedTopLevelStatement`.
pub(crate) struct Parser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> Parser<'src, 'stream, I> {
    /// Creates a new `Parser` instance backed by the given `ParserStream`.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses the entire token stream into a [`Program`].
    ///
    /// Iterates over all top-level statements, delegating each to the
    /// appropriate sub-parser based on the leading token type.
    ///
    /// # Errors
    ///
    /// Returns `Err(ParseError)` if any token violates the top-level grammar,
    /// including unsupported statement types and lexer errors propagated from the stream.
    pub(crate) fn parse(&mut self) -> Result<Program, ParseError> {
        let mut builder = ProgramBuilder::new();

        while let Some(token_ref) = self.stream.peek()? {
            let statement = match token_ref.token_type {
                TokenType::Var => VariableDeclarationParser::new(self.stream).parse()?,
                TokenType::Fn => FnParser::new(self.stream).parse()?,
                _ => {
                    return Err(ParseError::UnsupportedTopLevelStatement(
                        token_ref.token_type,
                        token_ref.line,
                    ));
                }
            };
            builder = builder.add(statement);
        }
        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expression;
    use crate::ast::statement::{
        Block, FunctionDefinition, FunctionParameter, Statement, VariableDeclaration,
    };
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

    #[test]
    fn parse_unsupported_top_level_assignment_statement() {
        let lexer = Lexer::new("x = 10;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);
        let res = parser.parse();
        assert_eq!(
            res.err().unwrap(),
            ParseError::UnsupportedTopLevelStatement(TokenType::Identifier, 1)
        );
    }

    #[test]
    fn parse_unsupported_top_level_conditional_statement() {
        let lexer = Lexer::new("if x > 0 { var y = 1; }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);
        let res = parser.parse();
        assert_eq!(
            res.err().unwrap(),
            ParseError::UnsupportedTopLevelStatement(TokenType::If, 1)
        );
    }

    #[test]
    fn parse_unsupported_top_level_iteration_statement() {
        let lexer = Lexer::new("loop {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);
        let res = parser.parse();
        assert_eq!(
            res.err().unwrap(),
            ParseError::UnsupportedTopLevelStatement(TokenType::Loop, 1)
        );
    }

    #[test]
    fn parse_top_level_function_definition() {
        let lexer = Lexer::new(
            "fn adjust_risk(score: i32): i32 { var risk_level = score; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::function_definition(FunctionDefinition::new(
                "adjust_risk".to_string(),
                vec![FunctionParameter::new(
                    "score".to_string(),
                    Some("i32".to_string()),
                )],
                Some("i32".to_string()),
                Block::new(vec![Statement::variable_declaration(
                    VariableDeclaration::new(
                        "risk_level".to_string(),
                        None,
                        Some(Expression::Identifier("score".to_string())),
                    ),
                )]),
            )))
            .build();
        assert_eq!(program, expected);
    }
}
