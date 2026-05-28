use crate::ast::program::Program;
use crate::lexer::keywords::Keywords;
use crate::lexer::Lexer;
use crate::parser::error::ParseError;
use crate::parser::stream::ParserStream;
use crate::parser::Parser;
use std::fmt;
use std::fs;
use std::path::Path;

/// Represents errors encountered during the type inference or parsing phases.
#[derive(Debug, PartialEq)]
pub enum InferenceError {
    /// A syntax or grammatical error occurred during the lexical analysis or parsing phase.
    ParseError(String),
    /// An error occurred during file I/O or file validation (e.g. invalid extension, missing file).
    FileError(String),
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
            InferenceError::FileError(msg) => write!(formatter, "{}", msg),
        }
    }
}

impl std::error::Error for InferenceError {}

/// The main entry point to the type inference compiler pipeline.
///
/// `Infer` orchestrates the conversion of raw source code strings into a parsed
/// Abstract Syntax Tree (AST) representation, preparing it for type inference.
pub struct Infer;

impl Default for Infer {
    /// Creates a default instance of the type inference orchestrator.
    fn default() -> Self {
        Self::new()
    }
}

impl Infer {
    /// Creates a new instance of the `Infer` orchestrator.
    pub fn new() -> Self {
        Infer
    }

    /// Compiles the raw source code into an untyped Abstract Syntax Tree (`Program`).
    ///
    /// # Errors
    ///
    /// Returns an `InferenceError::ParseError` if lexical analysis or recursive descent
    /// parsing encounters a syntax or structural violation.
    pub fn infer(&self, source: &str) -> Result<Program, InferenceError> {
        let lexer = Lexer::new(source, Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        parser.parse().map_err(|err| err.into())
    }

    /// Compiles a source file, verifying that it has the `.toy` extension.
    ///
    /// # Errors
    ///
    /// Returns an `InferenceError` if:
    /// - The file extension is not `.toy` (`InferenceError::FileError`).
    /// - The file cannot be read from the filesystem (`InferenceError::FileError`).
    /// - Lexical analysis or parsing encounters a syntax or structural violation (`InferenceError::ParseError`).
    pub fn infer_file<P: AsRef<Path>>(&self, path: P) -> Result<Program, InferenceError> {
        let path = path.as_ref();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toy") {
            return Err(InferenceError::FileError(format!(
                "invalid file extension, expected '.toy' for file: {}",
                path.display()
            )));
        }

        let source = fs::read_to_string(path)
            .map_err(|err| InferenceError::FileError(format!("failed to read file: {}", err)))?;

        self.infer(&source)
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

    #[test]
    fn infer_file() {
        let infer = Infer::new();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_program.toy");
        fs::write(&file_path, "var x = 10;").unwrap();

        let result = infer.infer_file(&file_path);
        let _ = fs::remove_file(&file_path);

        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.statements().len(), 1);
    }

    #[test]
    fn infer_file_invalid_extension() {
        let infer = Infer::new();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_program.txt");
        fs::write(&file_path, "var x = 10;").unwrap();

        let result = infer.infer_file(&file_path);
        let _ = fs::remove_file(&file_path);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            InferenceError::FileError(format!(
                "invalid file extension, expected '.toy' for file: {}",
                file_path.display()
            ))
        );
    }

    #[test]
    fn infer_file_not_found() {
        let infer = Infer::new();
        let file_path = Path::new("does_not_exist.toy");
        let result = infer.infer_file(file_path);

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, InferenceError::FileError(_)));
        assert!(err.to_string().contains("failed to read file"));
    }
}
