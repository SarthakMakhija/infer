use crate::ast::statement::{next_id, NodeId};
use crate::lexer::token::{Token, TokenType};
use crate::semantic::error::SemanticError;
use crate::semantic::visitor::ExpressionVisitor;
use std::fmt;

/// Represents errors encountered while parsing tokens into AST expression kinds.
#[derive(Debug, PartialEq)]
pub(crate) enum ExpressionError {
    /// Failed to parse a numeric literal string slice into a concrete integer value.
    ///
    /// Stores the invalid string that failed parsing and the line number where it occurred.
    ParseIntError(String, usize),

    /// The parser encountered a token type that cannot be converted into or used as an operator.
    ///
    /// Stores the unsupported `TokenType` and the line number where it occurred.
    UnsupportedOperator(TokenType, usize),
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionError::ParseIntError(val, line) => {
                write!(
                    formatter,
                    "failed to parse integer '{}' on line {}",
                    val, line
                )
            }
            ExpressionError::UnsupportedOperator(token_type, line) => {
                write!(
                    formatter,
                    "unsupported operator '{:?}' on line {}",
                    token_type, line
                )
            }
        }
    }
}

impl std::error::Error for ExpressionError {}

/// Represents a parsed expression with source location metadata in the AST.
#[derive(Debug)]
pub struct Expression {
    /// The specific variant of this expression.
    pub kind: ExpressionKind,
    /// The line number in the source code where this expression was parsed.
    pub line: usize,
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.line == other.line
    }
}

impl PartialEq<ExpressionKind> for Expression {
    fn eq(&self, other: &ExpressionKind) -> bool {
        &self.kind == other
    }
}

impl PartialEq<Expression> for ExpressionKind {
    fn eq(&self, other: &Expression) -> bool {
        self == &other.kind
    }
}

impl Expression {
    /// Creates a new `Expression` wrapping an `ExpressionKind` and its source line number.
    pub fn new(kind: ExpressionKind, line: usize) -> Self {
        Expression { kind, line }
    }

    /// Accepts an expression visitor, delegating the visitor dispatch to the inner `ExpressionKind`.
    pub(crate) fn accept(&self, visitor: &mut dyn ExpressionVisitor) -> Result<(), SemanticError> {
        self.kind.accept(visitor)
    }
}

/// Represents a parsed expression kind in the AST.
///
/// Expression kinds evaluate to values and can appear on the right-hand side
/// of declarations, in conditions, as function arguments, and more.
#[derive(Debug)]
pub enum ExpressionKind {
    /// A 32-bit signed integer literal (e.g., `42`).
    I32(i32),
    /// A string literal (e.g., `"hello"`).
    String(String),
    /// A boolean literal: `true` or `false`.
    Boolean(bool),
    /// A reference to a named variable or function (e.g., `score`), along with unique identifier.
    Identifier(String, NodeId),
    /// A unary expression applying an operator to a single operand (e.g., `-x`, `!flag`).
    Unary(Box<ExpressionKind>, UnaryOperator),
    /// A binary expression applying an operator to a left and right operand (e.g., `a + b`).
    Binary(Box<ExpressionKind>, BinaryOperator, Box<ExpressionKind>),
    /// A parenthesised expression that controls evaluation order (e.g., `(a + b)`).
    Grouped(Box<ExpressionKind>),
    /// A function call expression with a callee expression and a list of argument expressions, along with unique identifier.
    FunctionCall(Box<ExpressionKind>, Vec<ExpressionKind>, NodeId),
}

impl PartialEq for ExpressionKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExpressionKind::I32(this), ExpressionKind::I32(other)) => this == other,
            (ExpressionKind::String(this), ExpressionKind::String(other)) => this == other,
            (ExpressionKind::Boolean(this), ExpressionKind::Boolean(other)) => this == other,
            (ExpressionKind::Identifier(this, _), ExpressionKind::Identifier(other, _)) => {
                this == other
            }
            (ExpressionKind::Unary(this, this_op), ExpressionKind::Unary(other, other_op)) => {
                this == other && this_op == other_op
            }
            (
                ExpressionKind::Binary(this_left, this_op, this_right),
                ExpressionKind::Binary(other_left, other_op, other_right),
            ) => this_left == other_left && this_op == other_op && this_right == other_right,
            (ExpressionKind::Grouped(this), ExpressionKind::Grouped(other)) => this == other,
            (
                ExpressionKind::FunctionCall(this_callee, this_args, _),
                ExpressionKind::FunctionCall(other_callee, other_args, _),
            ) => this_callee == other_callee && this_args == other_args,
            _ => false,
        }
    }
}

impl ExpressionKind {
    /// Recursively unwraps nested `Grouped` expression kinds, returning the innermost non-grouped expression kind.
    pub(crate) fn unwrap_grouped(&self) -> &ExpressionKind {
        match self {
            ExpressionKind::Grouped(inner) => inner.unwrap_grouped(),
            _ => self,
        }
    }

    /// Factory method to construct an `ExpressionKind::Identifier`.
    pub fn identifier(name: String) -> Self {
        ExpressionKind::Identifier(name, next_id())
    }

    /// Factory method to construct an `ExpressionKind::FunctionCall`.
    pub fn function_call(callee: ExpressionKind, arguments: Vec<ExpressionKind>) -> Self {
        ExpressionKind::FunctionCall(Box::new(callee), arguments, next_id())
    }

    /// Accepts an expression visitor, dispatching the expression kind to the corresponding `visit_*` method.
    pub(crate) fn accept(&self, visitor: &mut dyn ExpressionVisitor) -> Result<(), SemanticError> {
        match self {
            ExpressionKind::Identifier(ref name, id) => visitor.visit_identifier(name, *id),
            ExpressionKind::FunctionCall(ref callee, ref arguments, _) => {
                visitor.visit_function_call(callee, arguments)
            }
            ExpressionKind::Unary(ref expr, _) => visitor.visit_unary(expr),
            ExpressionKind::Grouped(ref expr) => visitor.visit_grouped(expr),
            ExpressionKind::Binary(ref left, _, ref right) => visitor.visit_binary(left, right),
            _ => Ok(()),
        }
    }

    pub(crate) fn node_id(&self) -> Option<NodeId> {
        match self {
            ExpressionKind::Identifier(_, node_id) => Some(*node_id),
            ExpressionKind::FunctionCall(_, _, node_id) => Some(*node_id),
            _ => None,
        }
    }
}

/// Represents a binary operator used between two expression kinds.
///
/// Arithmetic operators follow standard mathematical precedence:
/// multiplication and division bind tighter than addition and subtraction.
/// Comparison and logical operators bind at a lower level.
#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    /// The `+` addition operator.
    Plus,
    /// The `-` subtraction operator.
    Minus,
    /// The `*` multiplication operator.
    Multiply,
    /// The `/` division operator.
    Divide,
    /// The `>` comparison operator.
    GreaterThan,
    /// The `<` comparison operator.
    LessThan,
    /// The `>=` comparison operator.
    GreaterThanEquals,
    /// The `<=` comparison operator.
    LessThanEquals,
    /// The `==` equality operator.
    EqualsEquals,
    /// The `!=` inequality operator.
    NotEquals,
    /// The `and` logical conjunction operator.
    And,
    /// The `or` logical disjunction operator.
    Or,
}

impl TryFrom<&Token<'_>> for BinaryOperator {
    type Error = ExpressionError;

    /// Attempts to convert a `Token` reference into a `BinaryOperator`.
    ///
    /// Returns `Err(ExpressionError::UnsupportedOperator)` if the token's type
    /// cannot be mapped to any known binary operator.
    fn try_from(token: &Token<'_>) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Plus => Ok(BinaryOperator::Plus),
            TokenType::Minus => Ok(BinaryOperator::Minus),
            TokenType::Star => Ok(BinaryOperator::Multiply),
            TokenType::Slash => Ok(BinaryOperator::Divide),
            TokenType::GreaterThan => Ok(BinaryOperator::GreaterThan),
            TokenType::LessThan => Ok(BinaryOperator::LessThan),
            TokenType::GreaterThanEquals => Ok(BinaryOperator::GreaterThanEquals),
            TokenType::LessThanEquals => Ok(BinaryOperator::LessThanEquals),
            TokenType::EqualsEquals => Ok(BinaryOperator::EqualsEquals),
            TokenType::BangEquals => Ok(BinaryOperator::NotEquals),
            TokenType::And => Ok(BinaryOperator::And),
            TokenType::Or => Ok(BinaryOperator::Or),
            _ => Err(ExpressionError::UnsupportedOperator(
                token.token_type,
                token.line,
            )),
        }
    }
}

impl BinaryOperator {
    /// Returns `true` if this operator is a relational or equality comparison
    /// (`>`, `>=`, `<`, `<=`, `==`, `!=`).
    ///
    /// Used to detect chained comparisons (e.g., `a < b < c`) which are not supported.
    pub(crate) fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOperator::GreaterThan
                | BinaryOperator::GreaterThanEquals
                | BinaryOperator::LessThan
                | BinaryOperator::LessThanEquals
                | BinaryOperator::EqualsEquals
                | BinaryOperator::NotEquals
        )
    }
}

/// Represents a unary operator applied to a single operand.
#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    /// The unary `-` negation operator (e.g., `-10`).
    Minus,
    /// The logical NOT `!` operator (e.g., `!active`).
    Negation,
}

impl TryFrom<&Token<'_>> for UnaryOperator {
    type Error = ExpressionError;

    /// Attempts to convert a `Token` reference into a `UnaryOperator`.
    ///
    /// Returns `Err(ExpressionError::UnsupportedOperator)` if the token's type
    /// cannot be mapped to a known unary operator.
    fn try_from(token: &Token<'_>) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Minus => Ok(UnaryOperator::Minus),
            TokenType::Bang => Ok(UnaryOperator::Negation),
            _ => Err(ExpressionError::UnsupportedOperator(
                token.token_type,
                token.line,
            )),
        }
    }
}

#[cfg(test)]
mod binary_operator_tests {
    use super::*;

    #[test]
    fn try_from_plus_token_to_plus_operator() {
        let token = Token::new(TokenType::Plus, 0..1, 1, "+");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Plus);
    }

    #[test]
    fn try_from_minus_token_to_minus_operator() {
        let token = Token::new(TokenType::Minus, 0..1, 1, "-");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Minus);
    }

    #[test]
    fn try_from_star_token_to_multiply_operator() {
        let token = Token::new(TokenType::Star, 0..1, 1, "*");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Multiply);
    }

    #[test]
    fn try_from_slash_token_to_divide_operator() {
        let token = Token::new(TokenType::Slash, 0..1, 1, "/");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Divide);
    }

    #[test]
    fn try_from_invalid_token_type_to_unsupported_operator_error() {
        let token = Token::new(TokenType::Identifier, 0..4, 2, "name");
        let result = BinaryOperator::try_from(&token);
        assert_eq!(
            result.err().unwrap(),
            ExpressionError::UnsupportedOperator(TokenType::Identifier, 2)
        );
    }

    #[test]
    fn try_from_greater_than_token_to_greater_than_operator() {
        let token = Token::new(TokenType::GreaterThan, 0..1, 1, ">");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::GreaterThan);
    }

    #[test]
    fn try_from_less_than_token_to_less_than_operator() {
        let token = Token::new(TokenType::LessThan, 0..1, 1, "<");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::LessThan);
    }

    #[test]
    fn try_from_greater_than_equals_token_to_greater_than_equals_operator() {
        let token = Token::new(TokenType::GreaterThanEquals, 0..2, 1, ">=");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::GreaterThanEquals);
    }

    #[test]
    fn try_from_less_than_equals_token_to_less_than_equals_operator() {
        let token = Token::new(TokenType::LessThanEquals, 0..2, 1, "<=");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::LessThanEquals);
    }

    #[test]
    fn try_from_equals_equals_token_to_equals_equals_operator() {
        let token = Token::new(TokenType::EqualsEquals, 0..2, 1, "==");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::EqualsEquals);
    }

    #[test]
    fn try_from_bang_equals_token_to_not_equals_operator() {
        let token = Token::new(TokenType::BangEquals, 0..2, 1, "!=");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::NotEquals);
    }

    #[test]
    fn try_from_and_token_to_and_operator() {
        let token = Token::new(TokenType::And, 0..3, 1, "and");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::And);
    }

    #[test]
    fn try_from_or_token_to_or_operator() {
        let token = Token::new(TokenType::Or, 0..2, 1, "or");
        let operator = BinaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, BinaryOperator::Or);
    }

    #[test]
    fn greater_than_is_comparison() {
        assert!(BinaryOperator::GreaterThan.is_comparison());
    }

    #[test]
    fn greater_than_equals_is_comparison() {
        assert!(BinaryOperator::GreaterThanEquals.is_comparison());
    }

    #[test]
    fn less_than_is_comparison() {
        assert!(BinaryOperator::LessThan.is_comparison());
    }

    #[test]
    fn less_than_equals_is_comparison() {
        assert!(BinaryOperator::LessThanEquals.is_comparison());
    }

    #[test]
    fn equals_equals_is_comparison() {
        assert!(BinaryOperator::EqualsEquals.is_comparison());
    }

    #[test]
    fn not_equals_is_comparison() {
        assert!(BinaryOperator::NotEquals.is_comparison());
    }

    #[test]
    fn plus_is_not_comparison() {
        assert!(!BinaryOperator::Plus.is_comparison());
    }
}

#[cfg(test)]
mod expression_tests {
    use super::*;

    #[test]
    fn unwrap_grouped_returns_inner_expression() {
        let inner_expression_kind = ExpressionKind::I32(42);
        let grouped_expression_kind = ExpressionKind::Grouped(Box::new(inner_expression_kind));
        assert_eq!(
            grouped_expression_kind.unwrap_grouped(),
            &ExpressionKind::I32(42)
        );
    }

    #[test]
    fn unwrap_grouped_returns_self_for_non_grouped() {
        let expression_kind = ExpressionKind::I32(42);
        assert_eq!(expression_kind.unwrap_grouped(), &ExpressionKind::I32(42));
    }

    #[test]
    fn unwrap_grouped_unwraps_multiple_levels() {
        let inner_expression_kind = ExpressionKind::I32(42);
        let nested_expression_kind = ExpressionKind::Grouped(Box::new(ExpressionKind::Grouped(
            Box::new(inner_expression_kind),
        )));
        assert_eq!(
            nested_expression_kind.unwrap_grouped(),
            &ExpressionKind::I32(42)
        );
    }

    struct TestExpressionVisitor {
        visited_identifier: Option<(String, NodeId)>,
        visited_function_call: bool,
        visited_unary: bool,
        visited_grouped: bool,
        visited_binary: bool,
    }

    impl ExpressionVisitor for TestExpressionVisitor {
        fn visit_identifier(&mut self, name: &str, node_id: NodeId) -> Result<(), SemanticError> {
            self.visited_identifier = Some((name.to_string(), node_id));
            Ok(())
        }

        fn visit_function_call(
            &mut self,
            _callee: &ExpressionKind,
            _arguments: &[ExpressionKind],
        ) -> Result<(), SemanticError> {
            self.visited_function_call = true;
            Ok(())
        }

        fn visit_unary(&mut self, _expr: &ExpressionKind) -> Result<(), SemanticError> {
            self.visited_unary = true;
            Ok(())
        }

        fn visit_binary(
            &mut self,
            _left: &ExpressionKind,
            _right: &ExpressionKind,
        ) -> Result<(), SemanticError> {
            self.visited_binary = true;
            Ok(())
        }

        fn visit_grouped(&mut self, _expr: &ExpressionKind) -> Result<(), SemanticError> {
            self.visited_grouped = true;
            Ok(())
        }
    }

    #[test]
    fn accept_dispatches_identifier_to_visitor() {
        let identifier_expression_kind = ExpressionKind::identifier("score".to_string());
        let expected_node_id = identifier_expression_kind.node_id().unwrap();

        let mut visitor = TestExpressionVisitor {
            visited_identifier: None,
            visited_function_call: false,
            visited_unary: false,
            visited_grouped: false,
            visited_binary: false,
        };

        let result = identifier_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.visited_identifier,
            Some(("score".to_string(), expected_node_id))
        );
    }

    #[test]
    fn accept_dispatches_function_call_to_visitor() {
        let callee_expression_kind = ExpressionKind::identifier("calculate_total".to_string());
        let call_expression_kind = ExpressionKind::function_call(callee_expression_kind, vec![]);

        let mut visitor = TestExpressionVisitor {
            visited_identifier: None,
            visited_function_call: false,
            visited_unary: false,
            visited_grouped: false,
            visited_binary: false,
        };

        let result = call_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(visitor.visited_function_call);
    }

    #[test]
    fn accept_dispatches_unary_to_visitor() {
        let operand_expression_kind = ExpressionKind::I32(10);
        let unary_expression_kind =
            ExpressionKind::Unary(Box::new(operand_expression_kind), UnaryOperator::Minus);

        let mut visitor = TestExpressionVisitor {
            visited_identifier: None,
            visited_function_call: false,
            visited_unary: false,
            visited_grouped: false,
            visited_binary: false,
        };

        let result = unary_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(visitor.visited_unary);
    }

    #[test]
    fn accept_dispatches_grouped_to_visitor() {
        let operand_expression_kind = ExpressionKind::I32(10);
        let grouped_expression_kind = ExpressionKind::Grouped(Box::new(operand_expression_kind));

        let mut visitor = TestExpressionVisitor {
            visited_identifier: None,
            visited_function_call: false,
            visited_unary: false,
            visited_grouped: false,
            visited_binary: false,
        };

        let result = grouped_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(visitor.visited_grouped);
    }

    #[test]
    fn accept_dispatches_binary_to_visitor() {
        let left_expression_kind = ExpressionKind::I32(10);
        let right_expression_kind = ExpressionKind::I32(20);
        let binary_expression_kind = ExpressionKind::Binary(
            Box::new(left_expression_kind),
            BinaryOperator::Plus,
            Box::new(right_expression_kind),
        );

        let mut visitor = TestExpressionVisitor {
            visited_identifier: None,
            visited_function_call: false,
            visited_unary: false,
            visited_grouped: false,
            visited_binary: false,
        };

        let result = binary_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(visitor.visited_binary);
    }
}

#[cfg(test)]
mod expression_node_id_tests {
    use super::*;

    #[test]
    fn gets_node_id_from_identifier() {
        let expression_kind = ExpressionKind::identifier("first_name".to_string());
        let ExpressionKind::Identifier(_, expected_id) = &expression_kind else {
            panic!("Expected ExpressionKind::Identifier");
        };

        assert_eq!(expression_kind.node_id(), Some(*expected_id));
        assert!(*expected_id > NodeId(0));
    }

    #[test]
    fn gets_node_id_from_function_call() {
        let callee_expression_kind = ExpressionKind::identifier("greeting".to_string());
        let expression_kind = ExpressionKind::function_call(callee_expression_kind, vec![]);
        let ExpressionKind::FunctionCall(_, _, expected_id) = &expression_kind else {
            panic!("Expected ExpressionKind::FunctionCall");
        };

        assert_eq!(expression_kind.node_id(), Some(*expected_id));
        assert!(*expected_id > NodeId(0));
    }

    #[test]
    fn returns_none_for_expressions_without_node_id() {
        assert_eq!(ExpressionKind::I32(42).node_id(), None);
        assert_eq!(ExpressionKind::String("hello".to_string()).node_id(), None);
        assert_eq!(ExpressionKind::Boolean(true).node_id(), None);
    }

    #[test]
    fn generates_unique_node_ids_for_multiple_identifiers() {
        let first_name_expression_kind = ExpressionKind::identifier("first_name".to_string());
        let second_name_expression_kind = ExpressionKind::identifier("first_name".to_string());
        assert_ne!(
            first_name_expression_kind.node_id(),
            second_name_expression_kind.node_id()
        );
    }

    #[test]
    fn generates_unique_node_ids_for_multiple_function_calls() {
        let callee_one_expression_kind = ExpressionKind::identifier("greeting".to_string());
        let call_one_expression_kind =
            ExpressionKind::function_call(callee_one_expression_kind, vec![]);

        let callee_two_expression_kind = ExpressionKind::identifier("greeting".to_string());
        let call_two_expression_kind =
            ExpressionKind::function_call(callee_two_expression_kind, vec![]);

        assert_ne!(
            call_one_expression_kind.node_id(),
            call_two_expression_kind.node_id()
        );
    }
}

#[cfg(test)]
mod unary_operator_tests {
    use super::*;

    #[test]
    fn try_from_minus_token_to_minus_operator() {
        let token = Token::new(TokenType::Minus, 0..1, 1, "-");
        let operator = UnaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, UnaryOperator::Minus);
    }

    #[test]
    fn try_from_bang_token_to_negation_operator() {
        let token = Token::new(TokenType::Bang, 0..1, 1, "!");
        let operator = UnaryOperator::try_from(&token).unwrap();
        assert_eq!(operator, UnaryOperator::Negation);
    }

    #[test]
    fn try_from_invalid_token_type_to_unsupported_operator_error() {
        let token = Token::new(TokenType::Identifier, 0..4, 2, "name");
        let result = UnaryOperator::try_from(&token);
        assert_eq!(
            result.err().unwrap(),
            ExpressionError::UnsupportedOperator(TokenType::Identifier, 2)
        );
    }
}
