/// Constructs a `Statement::VariableDeclaration` statement.
#[macro_export]
macro_rules! variable_declaration {
    // 1. Name only: variable_declaration!("user_score")
    ($name:expr) => {
        $crate::ast::statement::Statement::variable_declaration(
            $crate::ast::statement::VariableDeclaration::new($name.to_string(), None, None),
        )
    };

    // 2. Name and Type annotation: variable_declaration!("user_score", type: "i32")
    ($name:expr, type: $data_type:expr) => {
        $crate::ast::statement::Statement::variable_declaration(
            $crate::ast::statement::VariableDeclaration::new(
                $name.to_string(),
                Some($data_type.to_string()),
                None,
            ),
        )
    };

    // 3. Name and Value: variable_declaration!("user_score", value: expr)
    ($name:expr, value: $expression:expr) => {
        $crate::ast::statement::Statement::variable_declaration(
            $crate::ast::statement::VariableDeclaration::new(
                $name.to_string(),
                None,
                Some($expression),
            ),
        )
    };

    // 4. Name, Type, and Value: variable_declaration!("user_score", type: "i32", value: expr)
    ($name:expr, type: $data_type:expr, value: $expression:expr) => {
        $crate::ast::statement::Statement::variable_declaration(
            $crate::ast::statement::VariableDeclaration::new(
                $name.to_string(),
                Some($data_type.to_string()),
                Some($expression),
            ),
        )
    };
}

/// Constructs a `Statement::Assignment` statement.
#[macro_export]
macro_rules! assignment {
    ($name:expr, $expression:expr) => {
        $crate::ast::statement::Statement::assignment($crate::ast::statement::Assignment::new(
            $name.to_string(),
            $expression,
        ))
    };
}

/// Constructs a `Statement::If` (conditional) statement.
#[macro_export]
macro_rules! conditional {
    // 1. No else: conditional!(condition, body)
    ($condition:expr, $body:expr) => {
        $crate::ast::statement::Statement::conditional($crate::ast::statement::If::new(
            $condition, $body, None,
        ))
    };

    // 2. With else: conditional!(condition, body, else: block)
    ($condition:expr, $body:expr, else: $else_body:expr) => {
        $crate::ast::statement::Statement::conditional($crate::ast::statement::If::new(
            $condition,
            $body,
            Some($else_body),
        ))
    };
}

/// Constructs a `Statement::Break` statement.
#[macro_export]
macro_rules! break_statement {
    () => {
        $crate::ast::statement::Statement::control_flow($crate::ast::statement::Break::new())
    };
}

/// Constructs a `Statement::Return` statement.
#[macro_export]
macro_rules! return_statement {
    () => {
        $crate::ast::statement::Statement::return_($crate::ast::statement::Return::new(None))
    };
    ($expression:expr) => {
        $crate::ast::statement::Statement::return_($crate::ast::statement::Return::new(Some(
            $expression,
        )))
    };
}

/// Constructs a `Statement::Print` statement.
#[macro_export]
macro_rules! print_statement {
    ($($arg:expr),*) => {
        $crate::ast::statement::Statement::print($crate::ast::statement::Print::new(vec![$($arg),*]))
    };
}

#[cfg(test)]
mod tests {
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::Statement;

    #[test]
    fn variable_declaration_name_only() {
        let statement = variable_declaration!("user_score");
        let Statement::VariableDeclaration(declaration, _node_id) = statement else {
            panic!("Expected VariableDeclaration variant");
        };

        assert_eq!(declaration.variable(), "user_score");
        assert_eq!(declaration.data_type(), None);
        assert!(declaration.expression().is_none());
    }

    #[test]
    fn variable_declaration_with_type() {
        let statement = variable_declaration!("user_score", type: "i32");
        let Statement::VariableDeclaration(declaration, _node_id) = statement else {
            panic!("Expected VariableDeclaration variant");
        };

        assert_eq!(declaration.variable(), "user_score");
        assert_eq!(declaration.data_type(), Some("i32"));
        assert!(declaration.expression().is_none());
    }

    #[test]
    fn variable_declaration_with_value() {
        let expression = Expression::new(ExpressionKind::I32(42), 1);
        let statement = variable_declaration!("user_score", value: expression);
        let Statement::VariableDeclaration(declaration, _node_id) = statement else {
            panic!("Expected VariableDeclaration variant");
        };

        assert_eq!(declaration.variable(), "user_score");
        assert_eq!(declaration.data_type(), None);
        assert_eq!(
            declaration.expression().unwrap().kind,
            ExpressionKind::I32(42)
        );
    }

    #[test]
    fn variable_declaration_with_type_and_value() {
        let expression = Expression::new(ExpressionKind::I32(42), 1);
        let statement = variable_declaration!("user_score", type: "i32", value: expression);
        let Statement::VariableDeclaration(declaration, _node_id) = statement else {
            panic!("Expected VariableDeclaration variant");
        };

        assert_eq!(declaration.variable(), "user_score");
        assert_eq!(declaration.data_type(), Some("i32"));
        assert_eq!(
            declaration.expression().unwrap().kind,
            ExpressionKind::I32(42)
        );
    }

    #[test]
    fn break_statement() {
        let statement = break_statement!();
        assert!(matches!(statement, Statement::Break(_, _)));
    }

    #[test]
    fn return_statement() {
        let empty_return = return_statement!();
        let Statement::Return(ret, _node_id) = empty_return else {
            panic!("Expected Return variant");
        };
        assert!(ret.expression().is_none());

        let value_expression = expression_i32!(42, 1);
        let value_return = return_statement!(value_expression);
        let Statement::Return(ret_val, _node_id) = value_return else {
            panic!("Expected Return variant");
        };
        assert_eq!(
            ret_val.expression().as_ref().unwrap().kind,
            ExpressionKind::I32(42)
        );
    }

    #[test]
    fn print_statement() {
        let print_stmt = print_statement!(expression_i32!(42, 1));
        let Statement::Print(print, _node_id) = print_stmt else {
            panic!("Expected Print variant");
        };
        assert_eq!(print.arguments().len(), 1);
        assert_eq!(print.arguments()[0].kind, ExpressionKind::I32(42));
    }

    #[test]
    fn assignment_statement() {
        let expression = expression_i32!(42, 1);
        let statement = assignment!("user_score", expression);
        let Statement::Assignment(assignment, _node_id) = statement else {
            panic!("Expected Assignment variant");
        };

        assert_eq!(assignment.variable(), "user_score");
        assert_eq!(assignment.expression().kind, ExpressionKind::I32(42));
    }

    #[test]
    fn conditional_statement_without_else() {
        use crate::ast::statement::{Block, Statement};

        let statement = conditional!(expression_boolean!(true, 0), Block::new(vec![]));
        let Statement::If(if_statement, _node_id) = statement else {
            panic!("Expected If variant");
        };

        assert_eq!(if_statement.condition().kind, ExpressionKind::Boolean(true));
        assert!(if_statement.else_body().is_none());
    }

    #[test]
    fn conditional_statement_with_else() {
        use crate::ast::statement::{Block, Statement};

        let then_statement = variable_declaration!("then_var");
        let else_statement = variable_declaration!("else_var");

        let statement = conditional!(
            expression_boolean!(true, 0),
            Block::new(vec![then_statement]),
            else: Block::new(vec![else_statement])
        );
        let Statement::If(if_statement, _node_id) = statement else {
            panic!("Expected If variant");
        };

        assert_eq!(if_statement.condition().kind, ExpressionKind::Boolean(true));
        assert_eq!(if_statement.body().len(), 1);
        assert!(if_statement.else_body().is_some());
        assert_eq!(if_statement.else_body().unwrap().len(), 1);
    }
}
