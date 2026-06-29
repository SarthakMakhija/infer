use crate::ast::expr::Expression;
use crate::ast::statement::{
    Assignment, Block, FunctionDefinition, If, Loop, NodeId, Return, VariableDeclaration,
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
}
