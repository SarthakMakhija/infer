pub mod ast;
pub mod infer;

pub use infer::{Infer, InferenceError};

// Keep compiler internal phases private to the crate
pub(crate) mod lexer;
pub(crate) mod parser;

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

#[macro_export]
macro_rules! assert_function_name {
    ($function:expr, $expected_name:expr) => {{
        assert_eq!($function.name(), $expected_name);
    }};
}

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

#[macro_export]
macro_rules! assert_function_return_type {
    ($function:expr, $expected_return_type:expr) => {{
        assert_eq!($function.return_type(), $expected_return_type);
    }};
}

#[macro_export]
macro_rules! assert_function_body_len {
    ($function:expr, $expected_body_len:expr) => {{
        assert_eq!($function.body().len(), $expected_body_len);
    }};
}

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
