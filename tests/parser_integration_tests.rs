use infer::ast::expr::{BinaryOperator, Expression};
use infer::ast::statement::Statement;
use infer::{
    assert_assignment, assert_conditional, assert_function_body_len, assert_function_definition,
    assert_function_name, assert_function_parameters, assert_function_return_type, assert_loop,
    assert_variable_declaration, Infer, InferenceError,
};

#[test]
fn test_parse_function_with_parameters_and_return_type() {
    let compiler = Infer::new();
    let program = compiler
        .infer("fn calculate_sum(first: int, second: int): int {}")
        .unwrap();
    let statements = program.statements();
    assert_eq!(statements.len(), 1);

    let function = assert_function_definition!(&statements[0]);
    assert_function_name!(function, "calculate_sum");
    assert_function_parameters!(function, [("first", Some("int")), ("second", Some("int"))]);
    assert_function_return_type!(function, Some("int"));
    assert_function_body_len!(function, 0);
}

#[test]
fn test_parse_function_with_assignments() {
    let compiler = Infer::new();
    let program = compiler
        .infer("fn track_attempts() { var attempts = 1; attempts = 2; }")
        .unwrap();
    let statements = program.statements();
    assert_eq!(statements.len(), 1);

    let function = assert_function_definition!(&statements[0]);
    assert_function_name!(function, "track_attempts");
    assert_function_return_type!(function, None::<&str>);
    assert_function_body_len!(function, 2);

    let body = function.body();
    assert_variable_declaration!(&body[0], "attempts", None, Some(&Expression::I32(1)));

    assert_assignment!(&body[1], "attempts", &Expression::I32(2));
}

#[test]
fn test_parse_function_with_conditional() {
    let compiler = Infer::new();
    let program = compiler.infer("fn check_status(code: int) { if code == 200 { var success = 1; } else { var success = 0; } }").unwrap();
    let statements = program.statements();
    assert_eq!(statements.len(), 1);

    let function = assert_function_definition!(&statements[0]);
    assert_function_name!(function, "check_status");
    assert_function_parameters!(function, [("code", Some("int"))]);
    assert_function_return_type!(function, None::<&str>);
    assert_function_body_len!(function, 1);

    let body = function.body();
    let expected_condition = Expression::Binary(
        Box::new(Expression::Identifier("code".to_string())),
        BinaryOperator::EqualsEquals,
        Box::new(Expression::I32(200)),
    );
    let conditional_statement = assert_conditional!(&body[0], &expected_condition, 1, Some(1));

    assert_variable_declaration!(
        &conditional_statement.body()[0],
        "success",
        None,
        Some(&Expression::I32(1))
    );
    assert_variable_declaration!(
        &conditional_statement.else_body().unwrap()[0],
        "success",
        None,
        Some(&Expression::I32(0))
    );
}

#[test]
fn test_parse_function_with_loop() {
    let compiler = Infer::new();
    let program = compiler
        .infer("fn retry_connection() { loop { break; } }")
        .unwrap();
    let statements = program.statements();
    assert_eq!(statements.len(), 1);

    let function = assert_function_definition!(&statements[0]);
    assert_function_name!(function, "retry_connection");
    assert_function_return_type!(function, None::<&str>);
    assert_function_body_len!(function, 1);

    let body = function.body();
    let loop_statement = assert_loop!(&body[0], 1);

    let Statement::Break(_) = &loop_statement.body()[0] else {
        panic!("Expected Break");
    };
}

#[test]
fn test_parse_invalid_top_level_statement() {
    let compiler = Infer::new();
    let result = compiler.infer("attempts = 10;");

    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        InferenceError::ParseError(
            "unsupported token 'Identifier' at top-level on line 1".to_string()
        )
    );
}
