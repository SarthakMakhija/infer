/// Constructs a `VariableDeclaration` node.
#[macro_export]
macro_rules! variable_declaration {
    // 1. Name only: variable_declaration!("user_score")
    ($name:expr) => {
        $crate::ast::statement::VariableDeclaration::new($name.to_string(), None, None)
    };

    // 2. Name and Type annotation: variable_declaration!("user_score", type: "i32")
    ($name:expr, type: $data_type:expr) => {
        $crate::ast::statement::VariableDeclaration::new(
            $name.to_string(),
            Some($data_type.to_string()),
            None,
        )
    };

    // 3. Name and Value: variable_declaration!("user_score", value: expr)
    ($name:expr, value: $expression:expr) => {
        $crate::ast::statement::VariableDeclaration::new($name.to_string(), None, Some($expression))
    };

    // 4. Name, Type, and Value: variable_declaration!("user_score", type: "i32", value: expr)
    ($name:expr, type: $data_type:expr, value: $expression:expr) => {
        $crate::ast::statement::VariableDeclaration::new(
            $name.to_string(),
            Some($data_type.to_string()),
            Some($expression),
        )
    };
}

/// Constructs a `Statement::VariableDeclaration` statement.
#[macro_export]
macro_rules! statement_variable_declaration {
    ($($args:tt)*) => {
        $crate::ast::statement::Statement::variable_declaration(
            $crate::variable_declaration!($($args)*)
        )
    };
}

#[cfg(test)]
mod tests {
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::Statement;

    #[test]
    fn variable_declaration_name_only() {
        let declaration = variable_declaration!("user_score");
        assert_eq!(declaration.variable(), "user_score");
        assert_eq!(declaration.data_type(), None);
        assert!(declaration.expression().is_none());
    }

    #[test]
    fn variable_declaration_with_type() {
        let declaration = variable_declaration!("user_score", type: "i32");

        assert_eq!(declaration.variable(), "user_score");
        assert_eq!(declaration.data_type(), Some("i32"));
        assert!(declaration.expression().is_none());
    }

    #[test]
    fn variable_declaration_with_value() {
        let expression = Expression::new(ExpressionKind::I32(42), 1);
        let declaration = variable_declaration!("user_score", value: expression);

        assert_eq!(declaration.variable(), "user_score");
        assert_eq!(declaration.data_type(), None);
        assert_eq!(
            declaration.expression().unwrap().kind,
            ExpressionKind::I32(42)
        );
    }

    #[test]
    fn variable_declaration_with_type_and_value() {
        let expr = Expression::new(ExpressionKind::I32(42), 1);
        let declaration = variable_declaration!("user_score", type: "i32", value: expr);

        assert_eq!(declaration.variable(), "user_score");
        assert_eq!(declaration.data_type(), Some("i32"));
        assert_eq!(
            declaration.expression().unwrap().kind,
            ExpressionKind::I32(42)
        );
    }

    #[test]
    fn statement_variable_declaration() {
        let statement = statement_variable_declaration!("user_score");
        let Statement::VariableDeclaration(declaration, _node_id) = statement else {
            panic!("Expected VariableDeclaration variant");
        };
        assert_eq!(declaration.variable(), "user_score");
    }
}
