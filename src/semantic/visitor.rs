use crate::ast::expr::{Expression, ExpressionKind};
use crate::ast::statement::{
    Assignment, Block, FunctionDefinition, If, Loop, NodeId, Print, Return, VariableDeclaration,
};
use crate::semantic::error::SemanticError;

/// A visitor trait for walking and processing AST statement nodes.
pub(crate) trait StatementVisitor {
    /// Visits a variable declaration statement.
    fn visit_var_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> Result<(), SemanticError>;

    /// Visits a variable assignment statement.
    fn visit_assignment(
        &mut self,
        assignment: &Assignment,
        node_id: NodeId,
    ) -> Result<(), SemanticError>;

    /// Visits an if-else conditional block statement.
    fn visit_if(&mut self, if_statement: &If) -> Result<(), SemanticError>;

    /// Visits a loop iteration statement.
    fn visit_loop(&mut self, block: &Loop) -> Result<(), SemanticError>;

    /// Visits a block statement containing a sequence of statements.
    fn visit_block(&mut self, block: &Block) -> Result<(), SemanticError>;

    /// Visits a function definition statement.
    fn visit_function_definition(
        &mut self,
        definition: &FunctionDefinition,
    ) -> Result<(), SemanticError>;

    /// Visits a standalone expression statement representing a function call.
    fn visit_function_call(&mut self, call: &Expression) -> Result<(), SemanticError>;

    /// Visits a loop control flow break statement.
    fn visit_break(&mut self) -> Result<(), SemanticError>;

    /// Visits a function return statement.
    fn visit_return(&mut self, return_statement: &Return) -> Result<(), SemanticError>;

    /// Visits a print statement.
    fn visit_print(&mut self, print_statement: &Print) -> Result<(), SemanticError>;
}

/// A visitor trait for walking and processing AST expression kind nodes.
pub(crate) trait ExpressionVisitor {
    /// Visits an identifier expression.
    fn visit_identifier(&mut self, name: &str, node_id: NodeId) -> Result<(), SemanticError>;

    /// Visits a function call expression.
    fn visit_function_call(
        &mut self,
        callee: &ExpressionKind,
        arguments: &[ExpressionKind],
    ) -> Result<(), SemanticError>;

    /// Visits a unary expression.
    fn visit_unary(&mut self, expr: &ExpressionKind) -> Result<(), SemanticError>;

    /// Visits a binary operator expression.
    fn visit_binary(
        &mut self,
        left: &ExpressionKind,
        right: &ExpressionKind,
    ) -> Result<(), SemanticError>;

    /// Visits a parenthesized/grouped expression.
    fn visit_grouped(&mut self, expr: &ExpressionKind) -> Result<(), SemanticError>;
}
