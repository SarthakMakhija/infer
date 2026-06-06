use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub(crate) enum SemanticError {
    DuplicateVariable(String),
    ReturnOutsideFunction,
    MissingReturnExpression,
    UnexpectedReturnExpression,
    BreakOutsideLoop,
}

impl Display for SemanticError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticError::DuplicateVariable(variable) => {
                write!(formatter, "duplicate variable declaration: {}", variable)
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
        }
    }
}

impl std::error::Error for SemanticError {}
