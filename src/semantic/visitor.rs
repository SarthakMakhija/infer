use crate::ast::expr::Expression;
use crate::ast::statement::{
    Assignment, Block, FunctionDefinition, If, Loop, NodeId, Print, Return, VariableDeclaration,
};
use crate::semantic::error::SemanticError;

pub(crate) trait StatementVisitor {
    fn visit_var_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> Result<(), SemanticError>;

    fn visit_assignment(
        &mut self,
        assignment: &Assignment,
        node_id: NodeId,
    ) -> Result<(), SemanticError>;

    fn visit_if(&mut self, if_statement: &If) -> Result<(), SemanticError>;

    fn visit_loop(&mut self, block: &Loop) -> Result<(), SemanticError>;

    fn visit_block(&mut self, block: &Block) -> Result<(), SemanticError>;

    fn visit_function_definition(
        &mut self,
        definition: &FunctionDefinition,
    ) -> Result<(), SemanticError>;

    fn visit_function_call(&mut self, call: &Expression) -> Result<(), SemanticError>;

    fn visit_break(&mut self) -> Result<(), SemanticError>;

    fn visit_return(&mut self, return_statement: &Return) -> Result<(), SemanticError>;

    fn visit_print(&mut self, print_statement: &Print) -> Result<(), SemanticError>;
}

pub(crate) trait ExpressionVisitor {
    fn visit_identifier(&mut self, name: &str, node_id: NodeId) -> Result<(), SemanticError>;

    fn visit_function_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Result<(), SemanticError>;

    fn visit_unary(&mut self, expr: &Expression) -> Result<(), SemanticError>;

    fn visit_binary(&mut self, left: &Expression, right: &Expression) -> Result<(), SemanticError>;

    fn visit_grouped(&mut self, expr: &Expression) -> Result<(), SemanticError>;
}
