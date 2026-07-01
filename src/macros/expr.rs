/// Constructs an `ExpressionKind::I32` literal expression.
#[macro_export]
macro_rules! expression_i32 {
    ($val:expr) => {
        $crate::ast::expr::ExpressionKind::I32($val)
    };
    ($val:expr, $line:expr) => {
        $crate::ast::expr::Expression::new($crate::ast::expr::ExpressionKind::I32($val), $line)
    };
}

/// Constructs an `ExpressionKind::String` literal expression.
#[macro_export]
macro_rules! expression_string {
    ($val:expr) => {
        $crate::ast::expr::ExpressionKind::String($val.to_string())
    };
    ($val:expr, $line:expr) => {
        $crate::ast::expr::Expression::new(
            $crate::ast::expr::ExpressionKind::String($val.to_string()),
            $line,
        )
    };
}

/// Constructs an `ExpressionKind::Boolean` literal expression.
#[macro_export]
macro_rules! expression_boolean {
    ($val:expr) => {
        $crate::ast::expr::ExpressionKind::Boolean($val)
    };
    ($val:expr, $line:expr) => {
        $crate::ast::expr::Expression::new($crate::ast::expr::ExpressionKind::Boolean($val), $line)
    };
}

/// Constructs an `ExpressionKind::Identifier` expression.
#[macro_export]
macro_rules! expression_identifier {
    ($name:expr) => {
        $crate::ast::expr::ExpressionKind::identifier($name.to_string())
    };
    ($name:expr, $line:expr) => {
        $crate::ast::expr::Expression::new(
            $crate::ast::expr::ExpressionKind::identifier($name.to_string()),
            $line,
        )
    };
}

/// Constructs an `ExpressionKind::Unary` expression.
#[macro_export]
macro_rules! expression_unary {
    ($op:ident, $operand:expr) => {
        $crate::ast::expr::ExpressionKind::Unary(
            Box::new($operand),
            $crate::ast::expr::UnaryOperator::$op,
        )
    };
    ($op:ident, $operand:expr, $line:expr) => {
        $crate::ast::expr::Expression::new(
            $crate::ast::expr::ExpressionKind::Unary(
                Box::new($operand),
                $crate::ast::expr::UnaryOperator::$op,
            ),
            $line,
        )
    };
}

/// Constructs an `ExpressionKind::Binary` expression.
#[macro_export]
macro_rules! expression_binary {
    ($left:expr, $op:ident, $right:expr) => {
        $crate::ast::expr::ExpressionKind::Binary(
            Box::new($left),
            $crate::ast::expr::BinaryOperator::$op,
            Box::new($right),
        )
    };
    ($left:expr, $op:ident, $right:expr, $line:expr) => {
        $crate::ast::expr::Expression::new(
            $crate::ast::expr::ExpressionKind::Binary(
                Box::new($left),
                $crate::ast::expr::BinaryOperator::$op,
                Box::new($right),
            ),
            $line,
        )
    };
}

/// Constructs an `ExpressionKind::Grouped` expression.
#[macro_export]
macro_rules! expression_grouped {
    ($inner:expr) => {
        $crate::ast::expr::ExpressionKind::Grouped(Box::new($inner))
    };
    ($inner:expr, $line:expr) => {
        $crate::ast::expr::Expression::new(
            $crate::ast::expr::ExpressionKind::Grouped(Box::new($inner)),
            $line,
        )
    };
}

/// Constructs an `ExpressionKind::FunctionCall` expression.
#[macro_export]
macro_rules! expression_function_call {
    ($callee:expr, $args:expr) => {
        $crate::ast::expr::ExpressionKind::function_call($callee, $args)
    };
    ($callee:expr, $args:expr, $line:expr) => {
        $crate::ast::expr::Expression::new(
            $crate::ast::expr::ExpressionKind::function_call($callee, $args),
            $line,
        )
    };
}

#[cfg(test)]
mod tests {
    use crate::ast::expr::{BinaryOperator, ExpressionKind, UnaryOperator};

    #[test]
    fn expression_i32() {
        let expr = expression_i32!(42);
        assert_eq!(expr, ExpressionKind::I32(42));
    }

    #[test]
    fn expression_string() {
        let expr = expression_string!("hello");
        assert_eq!(expr, ExpressionKind::String("hello".to_string()));
    }

    #[test]
    fn expression_boolean() {
        let expr_true = expression_boolean!(true);
        assert_eq!(expr_true, ExpressionKind::Boolean(true));

        let expr_false = expression_boolean!(false);
        assert_eq!(expr_false, ExpressionKind::Boolean(false));
    }

    #[test]
    fn expression_identifier() {
        let expr = expression_identifier!("bonus");
        let ExpressionKind::Identifier(name, _node_id) = expr else {
            panic!("Expected Identifier variant");
        };
        assert_eq!(name, "bonus");
    }

    #[test]
    fn expression_unary() {
        let inner = expression_i32!(5);
        let expr = expression_unary!(Minus, inner);
        assert_eq!(
            expr,
            ExpressionKind::Unary(Box::new(ExpressionKind::I32(5)), UnaryOperator::Minus)
        );
    }

    #[test]
    fn expression_binary() {
        let left = expression_identifier!("bonus");
        let right = expression_i32!(10);
        let expr = expression_binary!(left, Plus, right);

        let ExpressionKind::Binary(left_box, op, right_box) = expr else {
            panic!("Expected Binary variant");
        };
        assert_eq!(op, BinaryOperator::Plus);
        assert_eq!(*right_box, ExpressionKind::I32(10));

        let ExpressionKind::Identifier(name, _) = *left_box else {
            panic!("Expected left operand to be Identifier");
        };
        assert_eq!(name, "bonus");
    }

    #[test]
    fn expression_grouped() {
        let inner = expression_i32!(42);
        let expr = expression_grouped!(inner);
        assert_eq!(
            expr,
            ExpressionKind::Grouped(Box::new(ExpressionKind::I32(42)))
        );
    }

    #[test]
    fn expression_function_call() {
        let callee = expression_identifier!("greet");
        let expr = expression_function_call!(callee, vec![expression_i32!(10)]);

        let ExpressionKind::FunctionCall(callee_box, args, _node_id) = expr else {
            panic!("Expected FunctionCall variant");
        };
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], ExpressionKind::I32(10));
        let ExpressionKind::Identifier(name, _) = *callee_box else {
            panic!("Expected callee to be Identifier");
        };
        assert_eq!(name, "greet");
    }

    #[test]
    fn expression_with_line() {
        let expr = expression_i32!(10, 42);
        assert_eq!(expr.kind, ExpressionKind::I32(10));
        assert_eq!(expr.line, 42);
    }
}
