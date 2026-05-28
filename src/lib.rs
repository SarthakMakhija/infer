pub mod ast;
pub mod infer;

pub use infer::{Infer, InferenceError};

// Keep compiler internal phases private to the crate
pub(crate) mod lexer;
pub(crate) mod parser;

/// Asserts that a [`ast::statement::Statement`] is a `VariableDeclaration` and checks its fields.
///
/// Panics with a descriptive message if the statement variant does not match.
/// Returns the inner `VariableDeclaration` reference so callers can chain further assertions.
///
/// # Arguments
///
/// * `$statement` - the `Statement` to inspect.
/// * `$expected_name` - expected variable name (`&str`).
/// * `$expected_type` - expected type annotation (`Option<&str>`).
/// * `$expected_expression` - expected initialiser expression (`Option<&Expression>`).
#[macro_export]
macro_rules! assert_variable_declaration {
    ($statement:expr, $expected_name:expr, $expected_type:expr, $expected_expression:expr) => {{
        use $crate::ast::statement::Statement;
        let Statement::VariableDeclaration(declaration) = $statement else {
            panic!("Expected VariableDeclaration, found {:?}", $statement);
        };
        assert_eq!(declaration.variable(), $expected_name);
        assert_eq!(declaration.data_type(), $expected_type);
        assert_eq!(declaration.expression(), $expected_expression);
        declaration
    }};
}

/// Asserts that a [`ast::statement::Statement`] is a `FunctionDefinition` and extracts it.
///
/// Panics with a descriptive message if the statement variant does not match.
/// Returns the inner `FunctionDefinition` reference so callers can chain further assertions.
#[macro_export]
macro_rules! assert_function_definition {
    ($statement:expr) => {{
        use $crate::ast::statement::Statement;
        let Statement::FunctionDefinition(function) = $statement else {
            panic!("Expected FunctionDefinition, found {:?}", $statement);
        };
        function
    }};
}

/// Asserts that a function's name matches the expected value.
#[macro_export]
macro_rules! assert_function_name {
    ($function:expr, $expected_name:expr) => {{
        assert_eq!($function.name(), $expected_name);
    }};
}

/// Asserts that a function's parameters match the expected list.
///
/// Accepts either `[]` for an empty parameter list, or a list of `(name, type)` pairs.
#[macro_export]
macro_rules! assert_function_parameters {
    ($function:expr, []) => {{
        let params = $function.parameters();
        assert_eq!(params.len(), 0, "Expected function parameters to be empty");
    }};

    ($function:expr, [ $(($param_name:expr, $param_type:expr)),+ ]) => {{
        let parameters = $function.parameters();
        let expected_parameters = [ $(($param_name, $param_type)),+ ];

        assert_eq!(parameters.len(), expected_parameters.len(), "Function parameter count mismatch");
        for (index, (expected_name, expected_type)) in expected_parameters.iter().enumerate() {
            assert_eq!(parameters[index].name(), *expected_name, "Parameter name mismatch at index {}", index);
            assert_eq!(parameters[index].data_type(), *expected_type, "Parameter type mismatch at index {}", index);
        }
    }};
}

/// Asserts that a function's return type matches the expected value.
#[macro_export]
macro_rules! assert_function_return_type {
    ($function:expr, $expected_return_type:expr) => {{
        assert_eq!($function.return_type(), $expected_return_type);
    }};
}

/// Asserts that the number of statements in a function's body equals the expected count.
#[macro_export]
macro_rules! assert_function_body_len {
    ($function:expr, $expected_body_len:expr) => {{
        assert_eq!($function.body().len(), $expected_body_len);
    }};
}

/// Asserts that a [`ast::statement::Statement`] is an `Assignment` and checks its variable and expression.
///
/// Returns the inner `Assignment` reference so callers can chain further assertions.
#[macro_export]
macro_rules! assert_assignment {
    ($statement:expr, $expected_variable:expr, $expected_expression:expr) => {{
        use $crate::ast::statement::Statement;
        let Statement::Assignment(assignment) = $statement else {
            panic!("Expected Assignment, found {:?}", $statement);
        };
        assert_eq!(assignment.variable(), $expected_variable);
        assert_eq!(assignment.expression(), $expected_expression);
        assignment
    }};
}

/// Asserts that a [`ast::statement::Statement`] is an `If` conditional and checks its condition, body length,
/// and optional else-body length.
///
/// Returns the inner `If` reference so callers can chain further assertions.
#[macro_export]
macro_rules! assert_conditional {
    ($statement:expr, $expected_condition:expr, $expected_body_len:expr, $expected_else_len:expr) => {{
        use $crate::ast::statement::Statement;
        let Statement::If(condition) = $statement else {
            panic!("Expected If conditional, found {:?}", $statement);
        };
        assert_eq!(condition.condition(), $expected_condition);
        assert_eq!(condition.body().len(), $expected_body_len);
        assert_eq!(condition.else_body().map(|b| b.len()), $expected_else_len);
        condition
    }};
}

/// Asserts that a [`ast::statement::Statement`] is a `Loop` and checks its body length.
///
/// Returns the inner `Loop` reference so callers can chain further assertions.
#[macro_export]
macro_rules! assert_loop {
    ($statement:expr, $expected_body_len:expr) => {{
        use $crate::ast::statement::Statement;
        let Statement::Loop(loop_) = $statement else {
            panic!("Expected Loop iteration, found {:?}", $statement);
        };
        assert_eq!(loop_.body().len(), $expected_body_len);
        loop_
    }};
}
