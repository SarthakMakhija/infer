use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub(crate) enum SemanticError {
    DuplicateVariable(String),
    UndefinedVariable(String),
    DuplicateFunctionName(String),
    ReturnOutsideFunction,
    MissingReturnExpression,
    UnexpectedReturnExpression,
    BreakOutsideLoop,
    UnreachableCode,
    NotAFunction(String),
    ArgumentCountMismatch(String, usize, usize),
}

impl Display for SemanticError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticError::DuplicateVariable(variable) => {
                write!(formatter, "duplicate variable declaration: {}", variable)
            }
            SemanticError::DuplicateFunctionName(name) => {
                write!(formatter, "duplicate function name: {}", name)
            }
            SemanticError::UndefinedVariable(variable) => {
                write!(formatter, "undefined variable: {}", variable)
            }
            SemanticError::ReturnOutsideFunction => {
                write!(formatter, "return statement outside of any function")
            }
            SemanticError::MissingReturnExpression => {
                write!(
                    formatter,
                    "empty return statement in a function with a return type"
                )
            }
            SemanticError::UnexpectedReturnExpression => {
                write!(
                    formatter,
                    "return statement with a value in a function with no return type"
                )
            }
            SemanticError::BreakOutsideLoop => {
                write!(formatter, "break statement outside of any loop")
            }
            SemanticError::UnreachableCode => {
                write!(formatter, "unreachable code")
            }
            SemanticError::NotAFunction(name) => {
                write!(formatter, "not a function: {}", name)
            }
            SemanticError::ArgumentCountMismatch(name, expected, actual) => {
                write!(
                    formatter,
                    "arity mismatch for function {}: expected {}, got {}",
                    name, expected, actual
                )
            }
        }
    }
}

impl std::error::Error for SemanticError {}
