use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub(crate) enum SemanticError {
    DuplicateVariable(String),
}

impl Display for SemanticError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticError::DuplicateVariable(variable) => {
                write!(formatter, "duplicate variable declaration: {}", variable)
            }
        }
    }
}

impl std::error::Error for SemanticError {}
