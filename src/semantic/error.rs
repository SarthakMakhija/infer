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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duplicate_variable_error() {
        assert_eq!(
            SemanticError::DuplicateVariable("score".to_string()).to_string(),
            "duplicate variable declaration: score"
        );
    }

    #[test]
    fn format_duplicate_function_name_error() {
        assert_eq!(
            SemanticError::DuplicateFunctionName("calculate".to_string()).to_string(),
            "duplicate function name: calculate"
        );
    }

    #[test]
    fn format_undefined_variable_error() {
        assert_eq!(
            SemanticError::UndefinedVariable("score".to_string()).to_string(),
            "undefined variable: score"
        );
    }

    #[test]
    fn format_return_outside_function_error() {
        assert_eq!(
            SemanticError::ReturnOutsideFunction.to_string(),
            "return statement outside of any function"
        );
    }

    #[test]
    fn format_missing_return_expression_error() {
        assert_eq!(
            SemanticError::MissingReturnExpression.to_string(),
            "empty return statement in a function with a return type"
        );
    }

    #[test]
    fn format_unexpected_return_expression_error() {
        assert_eq!(
            SemanticError::UnexpectedReturnExpression.to_string(),
            "return statement with a value in a function with no return type"
        );
    }

    #[test]
    fn format_break_outside_loop_error() {
        assert_eq!(
            SemanticError::BreakOutsideLoop.to_string(),
            "break statement outside of any loop"
        );
    }

    #[test]
    fn format_unreachable_code_error() {
        assert_eq!(
            SemanticError::UnreachableCode.to_string(),
            "unreachable code"
        );
    }

    #[test]
    fn format_not_a_function_error() {
        assert_eq!(
            SemanticError::NotAFunction("calculate".to_string()).to_string(),
            "not a function: calculate"
        );
    }

    #[test]
    fn format_argument_count_mismatch_error() {
        assert_eq!(
            SemanticError::ArgumentCountMismatch("calculate".to_string(), 2, 1).to_string(),
            "arity mismatch for function calculate: expected 2, got 1"
        );
    }
}
