use infer::ast::expr::{BinaryOperator, Expression};
use infer::ast::statement::Statement;
use infer::{
    assert_assignment, assert_conditional, assert_function_body_len, assert_function_definition,
    assert_function_name, assert_function_parameters, assert_function_return_type, assert_loop,
    assert_variable_declaration, Infer,
};
use std::path::Path;

#[test]
fn test_parse_factorial_example() {
    let compiler = Infer::new();
    let result = compiler.infer_file(Path::new("examples/factorial.toy"));

    assert!(
        result.is_ok(),
        "Failed to parse examples/factorial.toy: {:?}",
        result.err()
    );

    let program = result.unwrap();
    let statements = program.statements();
    assert_eq!(statements.len(), 1);

    let function = assert_function_definition!(&statements[0]);
    assert_function_name!(function, "factorial");
    assert_function_parameters!(function, [("n", Some("i32"))]);
    assert_function_return_type!(function, Some("i32"));
    assert_function_body_len!(function, 1);

    let body = function.body();
    let expected_condition = Expression::Binary(
        Box::new(Expression::Identifier("n".to_string())),
        BinaryOperator::LessThanEquals,
        Box::new(Expression::I32(1)),
    );
    let conditional = assert_conditional!(&body[0], &expected_condition, 1, Some(1));

    assert_variable_declaration!(
        &conditional.body()[0],
        "result",
        None,
        Some(&Expression::I32(1))
    );

    let else_body = conditional.else_body().unwrap();
    let expected_else_expr = Expression::Binary(
        Box::new(Expression::Identifier("n".to_string())),
        BinaryOperator::Multiply,
        Box::new(Expression::FunctionCall(
            Box::new(Expression::Identifier("factorial".to_string())),
            vec![Expression::Binary(
                Box::new(Expression::Identifier("n".to_string())),
                BinaryOperator::Minus,
                Box::new(Expression::I32(1)),
            )],
        )),
    );
    assert_variable_declaration!(&else_body[0], "result", None, Some(&expected_else_expr));
}

#[test]
fn test_parse_loops_example() {
    let compiler = Infer::new();
    let result = compiler.infer_file(Path::new("examples/loops.toy"));

    assert!(
        result.is_ok(),
        "Failed to parse examples/loops.toy: {:?}",
        result.err()
    );
    let program = result.unwrap();
    let statements = program.statements();
    assert_eq!(statements.len(), 1);

    let function = assert_function_definition!(&statements[0]);
    assert_function_name!(function, "run_loop");
    assert_function_parameters!(function, []);
    assert_function_return_type!(function, None::<&str>);
    assert_function_body_len!(function, 2);

    let body = function.body();
    assert_variable_declaration!(&body[0], "count", None, Some(&Expression::I32(0)));

    let loop_statement = assert_loop!(&body[1], 2);
    let loop_body = loop_statement.body();

    let expected_loop_cond = Expression::Binary(
        Box::new(Expression::Identifier("count".to_string())),
        BinaryOperator::GreaterThanEquals,
        Box::new(Expression::I32(10)),
    );
    let loop_conditional = assert_conditional!(&loop_body[0], &expected_loop_cond, 1, None);
    let Statement::Break(_) = &loop_conditional.body()[0] else {
        panic!("Expected Break statement");
    };

    let expected_assignment_expr = Expression::Binary(
        Box::new(Expression::Identifier("count".to_string())),
        BinaryOperator::Plus,
        Box::new(Expression::I32(1)),
    );
    assert_assignment!(&loop_body[1], "count", &expected_assignment_expr);
}

#[test]
fn test_parse_variables_example() {
    let compiler = Infer::new();
    let result = compiler.infer_file(Path::new("examples/variables.toy"));

    assert!(
        result.is_ok(),
        "Failed to parse examples/variables.toy: {:?}",
        result.err()
    );

    let program = result.unwrap();
    let statements = program.statements();

    assert_eq!(statements.len(), 4);
    assert_variable_declaration!(&statements[0], "x", None, Some(&Expression::I32(42)));

    let expected_y_expr = Expression::Binary(
        Box::new(Expression::Identifier("x".to_string())),
        BinaryOperator::Multiply,
        Box::new(Expression::I32(2)),
    );

    assert_variable_declaration!(&statements[1], "y", Some("i32"), Some(&expected_y_expr));
    assert_variable_declaration!(
        &statements[2],
        "active",
        Some("bool"),
        Some(&Expression::Boolean(true))
    );
    assert_variable_declaration!(
        &statements[3],
        "message",
        Some("string"),
        Some(&Expression::String("hello world".to_string()))
    );
}

#[test]
fn parse_functions_example() {
    let compiler = Infer::new();
    let result = compiler.infer_file(Path::new("examples/functions_no_type_annotations.toy"));

    assert!(
        result.is_ok(),
        "Failed to parse examples/functions_no_type_annotations.toy: {:?}",
        result.err()
    );

    let program = result.unwrap();
    let statements = program.statements();
    assert_eq!(statements.len(), 2);

    // 1. Validate "calculate" function: no parameter types, no return type
    let calculate_function = assert_function_definition!(&statements[0]);
    assert_function_name!(calculate_function, "calculate");
    assert_function_parameters!(calculate_function, [("a", None), ("b", None)]);
    assert_function_return_type!(calculate_function, None::<&str>);
    assert_function_body_len!(calculate_function, 1);

    let calculate_body = calculate_function.body();
    let expected_total_expr = Expression::Binary(
        Box::new(Expression::Identifier("a".to_string())),
        BinaryOperator::Plus,
        Box::new(Expression::Identifier("b".to_string())),
    );
    assert_variable_declaration!(
        &calculate_body[0],
        "total",
        None,
        Some(&expected_total_expr)
    );

    // 2. Validate "execute" function: calls "calculate" as initializer
    let execute_function = assert_function_definition!(&statements[1]);
    assert_function_name!(execute_function, "execute");
    assert_function_parameters!(execute_function, []);
    assert_function_return_type!(execute_function, None::<&str>);
    assert_function_body_len!(execute_function, 1);

    let execute_body = execute_function.body();
    let expected_result_expr = Expression::FunctionCall(
        Box::new(Expression::Identifier("calculate".to_string())),
        vec![Expression::I32(10), Expression::I32(20)],
    );
    assert_variable_declaration!(
        &execute_body[0],
        "result",
        None,
        Some(&expected_result_expr)
    );
}
