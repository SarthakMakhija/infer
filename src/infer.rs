use crate::ast::program::Program;
use crate::lexer::keywords::Keywords;
use crate::lexer::Lexer;
use crate::parser::error::ParseError;
use crate::parser::stream::ParserStream;
use crate::parser::Parser;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum InferenceError {
    ParseError(String),
}

impl From<ParseError> for InferenceError {
    fn from(error: ParseError) -> Self {
        InferenceError::ParseError(error.to_string())
    }
}

impl fmt::Display for InferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InferenceError::ParseError(msg) => write!(formatter, "{}", msg),
        }
    }
}

impl std::error::Error for InferenceError {}

pub struct Infer;

impl Default for Infer {
    fn default() -> Self {
        Self::new()
    }
}

impl Infer {
    pub fn new() -> Self {
        Infer
    }

    pub fn infer(&self, source: &str) -> Result<Program, InferenceError> {
        let lexer = Lexer::new(source, Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        parser.parse().map_err(|err| err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expression;

    use crate::{
        assert_function_body_len, assert_function_definition, assert_function_name,
        assert_function_parameters, assert_function_return_type, assert_variable_declaration,
    };

    #[test]
    fn infer_empty_source() {
        let infer = Infer::new();
        let program = infer.infer("").unwrap();
        assert_eq!(program.statements().len(), 0);
    }

    #[test]
    fn infer_valid_variable_declaration() {
        let infer = Infer::new();
        let program = infer.infer("var greeting = \"hello\";").unwrap();
        let statements = program.statements();

        assert_eq!(statements.len(), 1);
        assert_variable_declaration!(
            &statements[0],
            "greeting",
            None,
            Some(&Expression::String("hello".to_string()))
        );
    }

    #[test]
    fn infer_valid_function_definition() {
        let infer = Infer::new();
        let program = infer.infer("fn calculate() {}").unwrap();
        let statements = program.statements();

        assert_eq!(statements.len(), 1);
        let func = assert_function_definition!(&statements[0]);
        assert_function_name!(func, "calculate");
        assert_function_parameters!(func, []);
        assert_function_return_type!(func, None::<&str>);
        assert_function_body_len!(func, 0);
    }

    #[test]
    fn infer_invalid_top_level_statement() {
        let infer = Infer::new();
        let result = infer.infer("x = 10;");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            InferenceError::ParseError(
                "unsupported token 'Identifier' at top-level on line 1".to_string()
            )
        );
    }

    #[test]
    fn infer_lex_error() {
        let infer = Infer::new();
        let result = infer.infer("var x = 100; ?");
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            InferenceError::ParseError("unrecognized character '?' on line 1".to_string())
        );
    }
}
