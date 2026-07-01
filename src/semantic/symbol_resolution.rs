use crate::ast::expr::{Expression, ExpressionKind};
use crate::ast::statement::{
    Assignment, Block, FunctionDefinition, If, Loop, NodeId, Print, Return, Statement,
    VariableDeclaration,
};
use crate::semantic::error::SemanticError;
use crate::semantic::next_symbol_id;
use crate::semantic::resolution_table::ResolutionTable;
use crate::semantic::scope::Scopes;
use crate::semantic::state::{FunctionMetadata, State};
use crate::semantic::visitor::{ExpressionVisitor, StatementVisitor};

pub(crate) struct SymbolResolutionVisitor {
    scopes: Scopes,
    state: State,
    resolution_table: ResolutionTable,
}

impl SymbolResolutionVisitor {
    pub(crate) fn new() -> Self {
        Self {
            scopes: Scopes::new(),
            state: State::new(),
            resolution_table: ResolutionTable::new(),
        }
    }

    pub(crate) fn visit_statements(
        &mut self,
        statements: &[Statement],
    ) -> Result<(), SemanticError> {
        for statement in statements {
            if self.state.is_unreachable() {
                return Err(SemanticError::UnreachableCode);
            }
            statement.accept(self)?
        }
        Ok(())
    }

    pub(crate) fn resolve_pending_calls(&mut self) -> Result<(), SemanticError> {
        for pending_call in &self.state.pending_calls {
            let symbol_id = self
                .scopes
                .get(&pending_call.name)
                .ok_or_else(|| SemanticError::UndefinedVariable(pending_call.name.clone()))?;

            self.validate_function_call(&pending_call.name, pending_call.argument_count)?;
            self.resolution_table
                .resolve(pending_call.callee_node_id, symbol_id);
        }
        Ok(())
    }

    fn validate_function_call(
        &self,
        name: &str,
        argument_count: usize,
    ) -> Result<(), SemanticError> {
        let symbol_id = self
            .scopes
            .get(name)
            .ok_or_else(|| SemanticError::UndefinedVariable(name.to_string()))?;

        let metadata = self
            .state
            .get_global_function(&symbol_id)
            .ok_or_else(|| SemanticError::NotAFunction(name.to_string()))?;

        if metadata.parameter_count != argument_count {
            return Err(SemanticError::ArgumentCountMismatch(
                name.to_string(),
                metadata.parameter_count,
                argument_count,
            ));
        }
        Ok(())
    }
}

impl StatementVisitor for SymbolResolutionVisitor {
    fn visit_var_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> Result<(), SemanticError> {
        let name = variable_declaration.variable();
        if self.scopes.contains_locally(name) {
            return Err(SemanticError::DuplicateVariable(name.to_string()));
        }
        if let Some(expression_kind) = variable_declaration.expression() {
            expression_kind.accept(self)?;
        }
        self.scopes.define(name.to_string(), next_symbol_id());
        Ok(())
    }

    fn visit_assignment(
        &mut self,
        assignment: &Assignment,
        node_id: NodeId,
    ) -> Result<(), SemanticError> {
        let symbol_id = self.scopes.get(assignment.variable());
        if symbol_id.is_none() {
            return Err(SemanticError::UndefinedVariable(
                assignment.variable().to_string(),
            ));
        }
        assignment.expression().accept(self)?;
        //SAFETY: symbol_id has been checked for non-none.
        self.resolution_table.resolve(node_id, symbol_id.unwrap());
        Ok(())
    }

    fn visit_if(&mut self, if_statement: &If) -> Result<(), SemanticError> {
        if_statement.condition.accept(self)?;
        self.visit_block(&if_statement.body)?;
        if let Some(body) = if_statement.else_body.as_ref() {
            self.visit_block(body)?;
        };
        Ok(())
    }

    fn visit_loop(&mut self, loop_statement: &Loop) -> Result<(), SemanticError> {
        self.state.entered_loop();
        self.visit_block(&loop_statement.body)?;
        self.state.exited_loop();
        Ok(())
    }

    fn visit_block(&mut self, block: &Block) -> Result<(), SemanticError> {
        if self.state.is_unreachable() {
            return Err(SemanticError::UnreachableCode);
        }
        self.scopes.begin_scope();
        self.visit_statements(&block.statements)?;
        self.scopes.end_scope();
        self.state.reset_break();
        self.state.reset_return();
        Ok(())
    }

    fn visit_function_definition(
        &mut self,
        definition: &FunctionDefinition,
    ) -> Result<(), SemanticError> {
        if self.scopes.contains_locally(definition.name()) {
            return Err(SemanticError::DuplicateFunctionName(
                definition.name().to_string(),
            ));
        }

        let function_symbol_id = next_symbol_id();
        self.scopes
            .define(definition.name.to_string(), function_symbol_id);
        self.state.add_global_function(
            function_symbol_id,
            FunctionMetadata::new(
                definition.name.to_string(),
                definition.parameters.len(),
                definition.return_type.is_some(),
            ),
        );

        self.scopes.begin_scope();
        for parameter in definition.parameters() {
            let parameter_name = parameter.name();
            if self.scopes.contains_locally(parameter_name) {
                return Err(SemanticError::DuplicateVariable(parameter_name.to_string()));
            }
            self.scopes
                .define(parameter_name.to_string(), next_symbol_id());
        }
        self.visit_statements(definition.body())?;
        self.scopes.end_scope();
        self.state.exited_function();
        self.state.reset_return();
        Ok(())
    }

    fn visit_function_call(&mut self, call: &Expression) -> Result<(), SemanticError> {
        let ExpressionKind::FunctionCall(..) = &call.kind else {
            panic!("Expected ExpressionKind::FunctionCall variant");
        };
        call.kind.accept(self)
    }

    fn visit_break(&mut self) -> Result<(), SemanticError> {
        if !self.state.is_in_loop() {
            return Err(SemanticError::BreakOutsideLoop);
        }
        self.state.encountered_break();
        Ok(())
    }

    fn visit_return(&mut self, return_statement: &Return) -> Result<(), SemanticError> {
        if let Some(expression_kind) = return_statement.expression() {
            expression_kind.accept(self)?;
        }
        match self.state.current_function() {
            None => Err(SemanticError::ReturnOutsideFunction),
            Some(function_metadata) => {
                if return_statement.expression().is_none() && function_metadata.has_return_type {
                    return Err(SemanticError::MissingReturnExpression);
                }
                if return_statement.expression().is_some() && !function_metadata.has_return_type {
                    return Err(SemanticError::UnexpectedReturnExpression);
                }
                self.state.encountered_return();
                Ok(())
            }
        }
    }

    fn visit_print(&mut self, print_statement: &Print) -> Result<(), SemanticError> {
        for expression_kind in print_statement.arguments() {
            expression_kind.accept(self)?;
        }
        Ok(())
    }
}

impl ExpressionVisitor for SymbolResolutionVisitor {
    fn visit_identifier(&mut self, name: &str, node_id: NodeId) -> Result<(), SemanticError> {
        let symbol_id = self
            .scopes
            .get(name)
            .ok_or_else(|| SemanticError::UndefinedVariable(name.to_string()))?;

        self.resolution_table.resolve(node_id, symbol_id);
        Ok(())
    }

    fn visit_function_call(
        &mut self,
        callee: &ExpressionKind,
        arguments: &[ExpressionKind],
    ) -> Result<(), SemanticError> {
        let ExpressionKind::Identifier(ref name, callee_node_id) = callee else {
            return Err(SemanticError::NotAFunction("".to_string()));
        };

        match self.scopes.get(name) {
            None => self
                .state
                .add_pending_call(name.clone(), arguments.len(), *callee_node_id),

            Some(symbol_id) => {
                self.validate_function_call(name, arguments.len())?;
                self.resolution_table.resolve(*callee_node_id, symbol_id);
            }
        }
        for argument_expression_kind in arguments {
            argument_expression_kind.accept(self)?
        }
        Ok(())
    }

    fn visit_unary(&mut self, expr: &ExpressionKind) -> Result<(), SemanticError> {
        expr.accept(self)
    }

    fn visit_binary(
        &mut self,
        left: &ExpressionKind,
        right: &ExpressionKind,
    ) -> Result<(), SemanticError> {
        left.accept(self)?;
        right.accept(self)?;
        Ok(())
    }

    fn visit_grouped(&mut self, expr: &ExpressionKind) -> Result<(), SemanticError> {
        expr.accept(self)
    }
}

#[cfg(test)]
mod var_declaration_tests {
    use super::*;
    use crate::ast::statement::Statement;
    use crate::semantic::SymbolId;

    #[test]
    fn accepts_a_valid_variable_declaration() {
        let mut visitor = SymbolResolutionVisitor::new();
        let declaration = Statement::variable_declaration(VariableDeclaration::new(
            "username".to_string(),
            None,
            None,
        ));

        let result = declaration.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(visitor.scopes.contains("username"));
    }

    #[test]
    fn detects_duplicate_variable_declarations_in_the_same_scope() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.scopes.define("username".to_string(), SymbolId(1));

        let second_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "username".to_string(),
            None,
            None,
        ));
        let result = second_declaration.accept(&mut visitor);

        assert!(matches!(
            result,
            Err(SemanticError::DuplicateVariable(ref name)) if name == "username"
        ));
    }

    #[test]
    fn var_declaration_resolves_identifiers_in_initializer() {
        let mut visitor = SymbolResolutionVisitor::new();
        let bonus_symbol_id = SymbolId(10);
        visitor.scopes.define("bonus".to_string(), bonus_symbol_id);

        let initializer_expression_kind = ExpressionKind::identifier("bonus".to_string());
        let bonus_node_id = initializer_expression_kind.node_id().unwrap();

        let declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            Some(Expression::new(initializer_expression_kind, 0)),
        ));

        let result = declaration.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(visitor.scopes.contains("score"));
        assert_eq!(
            visitor.resolution_table.get(&bonus_node_id),
            Some(bonus_symbol_id)
        );
    }

    #[test]
    fn var_declaration_fails_if_initializer_has_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();

        let initializer_expression_kind = ExpressionKind::identifier("bonus".to_string());
        let declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            Some(Expression::new(initializer_expression_kind, 0)),
        ));

        let result = declaration.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("bonus".to_string()))
        );
        assert!(!visitor.scopes.contains("score"));
    }

    #[test]
    fn var_declaration_initializer_cannot_self_reference() {
        let mut visitor = SymbolResolutionVisitor::new();

        let initializer_expression_kind = ExpressionKind::identifier("score".to_string());
        let declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            Some(Expression::new(initializer_expression_kind, 0)),
        ));

        let result = declaration.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
        assert!(!visitor.scopes.contains("score"));
    }
}

#[cfg(test)]
mod assignment_tests {
    use super::*;
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::{Assignment, Statement};
    use crate::semantic::SymbolId;

    #[test]
    fn assignment_to_a_defined_variable_succeeds_and_records_resolution() {
        let mut visitor = SymbolResolutionVisitor::new();

        let expected_symbol_id = SymbolId(10);
        visitor
            .scopes
            .define("score".to_string(), expected_symbol_id);

        let assignment = Statement::assignment(Assignment::new(
            "score".to_string(),
            Expression::new(ExpressionKind::I32(100), 0),
        ));
        let assignment_id = assignment.id();

        let result = assignment.accept(&mut visitor);

        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&assignment_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn assignment_to_an_undefined_variable_fails_with_semantic_error() {
        let mut visitor = SymbolResolutionVisitor::new();

        let assignment = Statement::assignment(Assignment::new(
            "score".to_string(),
            Expression::new(ExpressionKind::I32(100), 0),
        ));

        let result = assignment.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }

    #[test]
    fn assignment_resolves_identifiers_in_expression() {
        let mut visitor = SymbolResolutionVisitor::new();
        let score_symbol_id = SymbolId(10);
        visitor.scopes.define("score".to_string(), score_symbol_id);

        let bonus_symbol_id = SymbolId(20);
        visitor.scopes.define("bonus".to_string(), bonus_symbol_id);

        let expression_kind = ExpressionKind::identifier("bonus".to_string());
        let bonus_node_id = expression_kind.node_id().unwrap();

        let assignment = Statement::assignment(Assignment::new(
            "score".to_string(),
            Expression::new(expression_kind, 0),
        ));
        let assignment_id = assignment.id();

        let result = assignment.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&assignment_id),
            Some(score_symbol_id)
        );
        assert_eq!(
            visitor.resolution_table.get(&bonus_node_id),
            Some(bonus_symbol_id)
        );
    }

    #[test]
    fn assignment_fails_if_expression_has_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.scopes.define("score".to_string(), SymbolId(10));

        let expression_kind = ExpressionKind::identifier("bonus".to_string());
        let assignment = Statement::assignment(Assignment::new(
            "score".to_string(),
            Expression::new(expression_kind, 0),
        ));

        let result = assignment.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("bonus".to_string()))
        );
    }
}

#[cfg(test)]
mod if_tests {
    use super::*;
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::{Assignment, Block, If, Statement, VariableDeclaration};
    use crate::semantic::SymbolId;

    #[test]
    fn variables_declared_inside_then_block_are_inaccessible_after_if_statement_exits() {
        let mut visitor = SymbolResolutionVisitor::new();

        let then_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "then_var".to_string(),
            None,
            None,
        ));
        let if_statement = Statement::conditional(If::new(
            Expression::new(ExpressionKind::Boolean(true), 0),
            Block::new(vec![then_declaration]),
            None,
        ));
        assert!(if_statement.accept(&mut visitor).is_ok());

        let assignment = Statement::assignment(Assignment::new(
            "then_var".to_string(),
            Expression::new(ExpressionKind::I32(10), 0),
        ));
        let result = assignment.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("then_var".to_string()))
        );
    }

    #[test]
    fn variables_declared_inside_else_block_are_inaccessible_after_if_statement_exits() {
        let mut visitor = SymbolResolutionVisitor::new();

        let else_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "else_var".to_string(),
            None,
            None,
        ));
        let if_statement = Statement::conditional(If::new(
            Expression::new(ExpressionKind::Boolean(false), 0),
            Block::new(vec![]),
            Some(Block::new(vec![else_declaration])),
        ));
        assert!(if_statement.accept(&mut visitor).is_ok());

        let assignment = Statement::assignment(Assignment::new(
            "else_var".to_string(),
            Expression::new(ExpressionKind::I32(10), 0),
        ));
        let result = assignment.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("else_var".to_string()))
        );
    }

    #[test]
    fn variables_declared_inside_then_block_are_not_accessible_within_else_block() {
        let mut visitor = SymbolResolutionVisitor::new();

        let then_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "first_name".to_string(),
            None,
            None,
        ));

        let else_assign = Statement::assignment(Assignment::new(
            "first_name".to_string(),
            Expression::new(ExpressionKind::I32(10), 0),
        ));

        let if_statement = Statement::conditional(If::new(
            Expression::new(ExpressionKind::Boolean(true), 0),
            Block::new(vec![then_declaration]),
            Some(Block::new(vec![else_assign])),
        ));

        let result = if_statement.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("first_name".to_string()))
        );
    }

    #[test]
    fn then_and_else_blocks_can_access_variables_declared_in_outer_scope() {
        let mut visitor = SymbolResolutionVisitor::new();

        let expected_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("outer_var".to_string(), expected_symbol_id);

        let then_assign = Statement::assignment(Assignment::new(
            "outer_var".to_string(),
            Expression::new(ExpressionKind::I32(10), 0),
        ));
        let then_assign_id = then_assign.id();

        let else_assign = Statement::assignment(Assignment::new(
            "outer_var".to_string(),
            Expression::new(ExpressionKind::I32(20), 0),
        ));
        let else_assign_id = else_assign.id();

        let if_statement = Statement::conditional(If::new(
            Expression::new(ExpressionKind::Boolean(true), 0),
            Block::new(vec![then_assign]),
            Some(Block::new(vec![else_assign])),
        ));

        assert!(if_statement.accept(&mut visitor).is_ok());
        assert_eq!(
            visitor.resolution_table.get(&then_assign_id),
            Some(expected_symbol_id)
        );
        assert_eq!(
            visitor.resolution_table.get(&else_assign_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn if_condition_resolves_identifiers() {
        let mut visitor = SymbolResolutionVisitor::new();
        let expected_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("score".to_string(), expected_symbol_id);

        let condition_expression_kind = ExpressionKind::identifier("score".to_string());
        let score_node_id = condition_expression_kind.node_id().unwrap();

        let if_statement = Statement::conditional(If::new(
            Expression::new(condition_expression_kind, 0),
            Block::new(vec![]),
            None,
        ));

        let result = if_statement.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&score_node_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn if_condition_fails_given_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();

        let condition_expression_kind = ExpressionKind::identifier("score".to_string());
        let if_statement = Statement::conditional(If::new(
            Expression::new(condition_expression_kind, 0),
            Block::new(vec![]),
            None,
        ));

        let result = if_statement.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }
}

#[cfg(test)]
mod loop_tests {
    use super::*;
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::{Assignment, Block, Break, Loop, Statement, VariableDeclaration};

    #[test]
    fn entering_a_loop_updates_the_state_to_be_inside_a_loop() {
        let mut visitor = SymbolResolutionVisitor::new();

        let break_statement = Statement::control_flow(Break::new());
        let loop_statement = Statement::iteration(Loop::new(Block::new(vec![break_statement])));

        let result = loop_statement.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(!visitor.state.is_in_loop());
    }

    #[test]
    fn nested_loops_track_state_depth_correctly() {
        let mut visitor = SymbolResolutionVisitor::new();

        let inner_break = Statement::control_flow(Break::new());
        let inner_loop = Statement::iteration(Loop::new(Block::new(vec![inner_break])));

        let outer_break = Statement::control_flow(Break::new());
        let outer_loop = Statement::iteration(Loop::new(Block::new(vec![inner_loop, outer_break])));

        let result = outer_loop.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(!visitor.state.is_in_loop());
    }

    #[test]
    fn variables_declared_inside_loop_are_inaccessible_after_loop_exits() {
        let mut visitor = SymbolResolutionVisitor::new();

        let var_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "name".to_string(),
            None,
            None,
        ));
        let loop_statement = Statement::iteration(Loop::new(Block::new(vec![var_declaration])));
        assert!(loop_statement.accept(&mut visitor).is_ok());

        let assignment = Statement::assignment(Assignment::new(
            "name".to_string(),
            Expression::new(ExpressionKind::String("John".to_string()), 0),
        ));
        let result = assignment.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("name".to_string()))
        );
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::{Assignment, Block, Statement, VariableDeclaration};
    use crate::semantic::SymbolId;

    #[test]
    fn block_creates_a_new_lexical_scope_allowing_shadowing() {
        let mut visitor = SymbolResolutionVisitor::new();

        let outer_symbol_id = SymbolId(1);
        visitor.scopes.define("score".to_string(), outer_symbol_id);

        let inner_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));

        let block = Statement::block(Block::new(vec![inner_declaration]));
        assert!(block.accept(&mut visitor).is_ok());
        assert_eq!(visitor.scopes.get("score"), Some(outer_symbol_id));
    }

    #[test]
    fn variables_declared_inside_block_are_inaccessible_after_block_exits() {
        let mut visitor = SymbolResolutionVisitor::new();

        let declaration = Statement::variable_declaration(VariableDeclaration::new(
            "temp".to_string(),
            None,
            None,
        ));
        let block = Statement::block(Block::new(vec![declaration]));
        assert!(block.accept(&mut visitor).is_ok());

        // Assign to "temp" outside the block.
        let assignment = Statement::assignment(Assignment::new(
            "temp".to_string(),
            Expression::new(ExpressionKind::I32(42), 0),
        ));

        let result = assignment.accept(&mut visitor);

        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("temp".to_string()))
        );
    }

    #[test]
    fn inner_block_can_access_variables_declared_in_enclosing_scope() {
        let mut visitor = SymbolResolutionVisitor::new();

        let expected_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("score".to_string(), expected_symbol_id);

        let inner_assignment = Statement::assignment(Assignment::new(
            "score".to_string(),
            Expression::new(ExpressionKind::I32(50), 0),
        ));
        let assignment_id = inner_assignment.id();
        let block = Statement::block(Block::new(vec![inner_assignment]));

        assert!(block.accept(&mut visitor).is_ok());
        assert_eq!(
            visitor.resolution_table.get(&assignment_id),
            Some(expected_symbol_id)
        );
    }
}

#[cfg(test)]
mod function_definition_tests {
    use super::*;
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::{
        Assignment, Block, FunctionDefinition, FunctionParameter, Statement,
    };
    use crate::semantic::SymbolId;

    #[test]
    fn accepts_a_valid_function_definition() {
        let mut visitor = SymbolResolutionVisitor::new();

        let first_parameter =
            FunctionParameter::new("first_score".to_string(), Some("i32".to_string()));
        let second_parameter =
            FunctionParameter::new("second_score".to_string(), Some("i32".to_string()));

        let function_definition = Statement::function_definition(FunctionDefinition::new(
            "calculate_total".to_string(),
            vec![first_parameter, second_parameter],
            Some("i32".to_string()),
            Block::new(vec![]),
        ));

        let result = function_definition.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(visitor.scopes.contains("calculate_total"));
    }

    #[test]
    fn detects_duplicate_function_names_in_the_same_scope() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor
            .scopes
            .define("calculate_total".to_string(), SymbolId(1));

        let second_function = Statement::function_definition(FunctionDefinition::new(
            "calculate_total".to_string(),
            vec![],
            None,
            Block::new(vec![]),
        ));
        let result = second_function.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::DuplicateFunctionName(
                "calculate_total".to_string()
            ))
        );
    }

    #[test]
    fn rejects_function_definitions_with_duplicate_parameter_names() {
        let mut visitor = SymbolResolutionVisitor::new();

        let first_parameter = FunctionParameter::new("score".to_string(), Some("int".to_string()));
        let second_parameter = FunctionParameter::new("score".to_string(), Some("int".to_string()));

        let function_definition = Statement::function_definition(FunctionDefinition::new(
            "calculate_total".to_string(),
            vec![first_parameter, second_parameter],
            None,
            Block::new(vec![]),
        ));

        let result = function_definition.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::DuplicateVariable("score".to_string()))
        );
    }

    #[test]
    fn parameters_shadow_outer_scope_variables_inside_function_body() {
        let mut visitor = SymbolResolutionVisitor::new();
        let outer_symbol_id = SymbolId(1);

        visitor.scopes.define("score".to_string(), outer_symbol_id);

        let function_parameter =
            FunctionParameter::new("score".to_string(), Some("int".to_string()));
        let inner_assignment = Statement::assignment(Assignment::new(
            "score".to_string(),
            Expression::new(ExpressionKind::I32(100), 0),
        ));
        let assignment_id = inner_assignment.id();

        let function_definition = Statement::function_definition(FunctionDefinition::new(
            "calculate_total".to_string(),
            vec![function_parameter],
            None,
            Block::new(vec![inner_assignment]),
        ));
        assert!(function_definition.accept(&mut visitor).is_ok());

        let inner_symbol_id = visitor.resolution_table.get(&assignment_id).unwrap();
        assert_ne!(inner_symbol_id, outer_symbol_id);
    }

    #[test]
    fn registers_global_function_with_parameter_count_in_state() {
        let mut visitor = SymbolResolutionVisitor::new();

        let parameter = FunctionParameter::new("name".to_string(), Some("String".to_string()));
        let function_definition = Statement::function_definition(FunctionDefinition::new(
            "greeting".to_string(),
            vec![parameter],
            None,
            Block::new(vec![]),
        ));

        assert!(function_definition.accept(&mut visitor).is_ok());

        let symbol_id = visitor.scopes.get("greeting").unwrap();
        let metadata = visitor.state.get_global_function(&symbol_id).unwrap();
        assert_eq!(metadata.name, "greeting");
        assert_eq!(metadata.parameter_count, 1);
    }
}

#[cfg(test)]
mod function_call_tests {
    use super::*;
    use crate::ast::expr::ExpressionKind;
    use crate::ast::statement::Statement;
    use crate::semantic::state::PendingCall;
    use crate::semantic::SymbolId;

    #[test]
    fn accepts_valid_function_call() {
        let mut visitor = SymbolResolutionVisitor::new();

        let function_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("calculate_total".to_string(), function_symbol_id);
        visitor.state.add_global_function(
            function_symbol_id,
            FunctionMetadata::new("calculate_total".to_string(), 0, false),
        );

        let callee_expression_kind = ExpressionKind::identifier("calculate_total".to_string());
        let call_expression_kind = ExpressionKind::function_call(callee_expression_kind, vec![]);
        let call_statement = Statement::function_call(Expression::new(call_expression_kind, 0));

        let result = call_statement.accept(&mut visitor);
        assert!(result.is_ok());
        assert!(visitor.state.pending_calls.is_empty());
    }

    #[test]
    fn detects_arity_mismatch_on_function_call() {
        let mut visitor = SymbolResolutionVisitor::new();

        let function_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("calculate_total".to_string(), function_symbol_id);
        visitor.state.add_global_function(
            function_symbol_id,
            FunctionMetadata::new("calculate_total".to_string(), 1, false),
        );

        let callee_expression_kind = ExpressionKind::identifier("calculate_total".to_string());
        let call_expression_kind = ExpressionKind::function_call(callee_expression_kind, vec![]);
        let call_statement = Statement::function_call(Expression::new(call_expression_kind, 0));

        let result = call_statement.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::ArgumentCountMismatch(
                "calculate_total".to_string(),
                1,
                0
            ))
        );
    }

    #[test]
    fn detects_shadowed_function_call_on_variable() {
        let mut visitor = SymbolResolutionVisitor::new();

        let variable_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("calculate_total".to_string(), variable_symbol_id);

        let callee_expression_kind = ExpressionKind::identifier("calculate_total".to_string());
        let call_expression_kind = ExpressionKind::function_call(callee_expression_kind, vec![]);
        let call_statement = Statement::function_call(Expression::new(call_expression_kind, 0));

        let result = call_statement.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::NotAFunction("calculate_total".to_string()))
        );
    }

    #[test]
    fn defers_unresolved_function_call_to_pending_calls() {
        let mut visitor = SymbolResolutionVisitor::new();

        let callee_expression_kind = ExpressionKind::identifier("calculate_total".to_string());
        let expected_callee_node_id = callee_expression_kind.node_id().unwrap();
        let call_expression_kind =
            ExpressionKind::function_call(callee_expression_kind, vec![ExpressionKind::I32(42)]);
        let call_statement = Statement::function_call(Expression::new(call_expression_kind, 0));

        let result = call_statement.accept(&mut visitor);
        assert!(result.is_ok());

        assert_eq!(
            visitor.state.pending_calls,
            vec![PendingCall {
                name: "calculate_total".to_string(),
                argument_count: 1,
                callee_node_id: expected_callee_node_id,
            }]
        );
    }

    #[test]
    fn detects_non_identifier_callee_as_not_a_function() {
        let mut visitor = SymbolResolutionVisitor::new();

        let callee_expression_kind = ExpressionKind::I32(42);
        let call_expression_kind = ExpressionKind::function_call(callee_expression_kind, vec![]);
        let call_statement = Statement::function_call(Expression::new(call_expression_kind, 0));

        let result = call_statement.accept(&mut visitor);
        assert_eq!(result, Err(SemanticError::NotAFunction("".to_string())));
    }

    #[test]
    #[should_panic(expected = "Expected ExpressionKind::FunctionCall variant")]
    fn panics_on_non_function_call_expression_variant() {
        let mut visitor = SymbolResolutionVisitor::new();
        let expression_kind = ExpressionKind::I32(42);
        let expression = Expression::new(expression_kind, 0);
        let _ = StatementVisitor::visit_function_call(&mut visitor, &expression);
    }

    #[test]
    fn visitor_resolves_immediate_function_call_in_resolution_table() {
        let mut visitor = SymbolResolutionVisitor::new();

        let function_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("calculate_total".to_string(), function_symbol_id);
        visitor.state.add_global_function(
            function_symbol_id,
            FunctionMetadata::new("calculate_total".to_string(), 0, false),
        );

        let callee_expression_kind = ExpressionKind::identifier("calculate_total".to_string());
        let callee_node_id = callee_expression_kind.node_id().unwrap();
        let call_expression_kind = ExpressionKind::function_call(callee_expression_kind, vec![]);

        let result = call_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&callee_node_id),
            Some(function_symbol_id)
        );
    }

    #[test]
    fn visitor_resolves_deferred_function_call_in_resolution_table_on_pending_calls_resolve() {
        let mut visitor = SymbolResolutionVisitor::new();

        let callee_expression_kind = ExpressionKind::identifier("calculate_total".to_string());
        let callee_node_id = callee_expression_kind.node_id().unwrap();
        let call_expression_kind = ExpressionKind::function_call(callee_expression_kind, vec![]);

        // Visit call, which defers it
        let result = call_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(visitor.resolution_table.get(&callee_node_id), None);

        // Define the function globally later
        let function_symbol_id = SymbolId(2);
        visitor
            .scopes
            .define("calculate_total".to_string(), function_symbol_id);

        visitor.state.add_global_function(
            function_symbol_id,
            FunctionMetadata::new("calculate_total".to_string(), 0, false),
        );

        // Resolve pending calls
        let resolve_result = visitor.resolve_pending_calls();
        assert!(resolve_result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&callee_node_id),
            Some(function_symbol_id)
        );
    }

    #[test]
    fn visitor_resolves_identifier_inside_function_call_arguments() {
        let mut visitor = SymbolResolutionVisitor::new();

        let function_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("calculate_total".to_string(), function_symbol_id);
        visitor.state.add_global_function(
            function_symbol_id,
            FunctionMetadata::new("calculate_total".to_string(), 1, false),
        );

        let variable_symbol_id = SymbolId(2);
        visitor
            .scopes
            .define("score".to_string(), variable_symbol_id);

        let callee_expression_kind = ExpressionKind::identifier("calculate_total".to_string());
        let callee_node_id = callee_expression_kind.node_id().unwrap();

        let argument_expression_kind = ExpressionKind::identifier("score".to_string());
        let argument_node_id = argument_expression_kind.node_id().unwrap();
        let call_expression_kind =
            ExpressionKind::function_call(callee_expression_kind, vec![argument_expression_kind]);

        let result = call_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&callee_node_id),
            Some(function_symbol_id)
        );
        assert_eq!(
            visitor.resolution_table.get(&argument_node_id),
            Some(variable_symbol_id)
        );
    }
}

#[cfg(test)]
mod break_tests {
    use super::*;
    use crate::ast::statement::{Break, Statement};

    #[test]
    fn break_statement_outside_any_loop_is_invalid() {
        let mut visitor = SymbolResolutionVisitor::new();
        let break_statement = Statement::control_flow(Break::new());

        let result = break_statement.accept(&mut visitor);
        assert_eq!(result, Err(SemanticError::BreakOutsideLoop));
    }

    #[test]
    fn break_statement_inside_a_loop_is_valid() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.state.entered_loop();

        let break_statement = Statement::control_flow(Break::new());
        let result = break_statement.accept(&mut visitor);

        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod return_tests {
    use super::*;
    use crate::ast::expr::ExpressionKind;
    use crate::ast::statement::Statement;
    use crate::semantic::state::FunctionMetadata;
    use crate::semantic::SymbolId;

    #[test]
    fn return_statement_outside_any_function_is_invalid() {
        let mut visitor = SymbolResolutionVisitor::new();

        let return_statement = Statement::return_(Return::new(None));
        let result = return_statement.accept(&mut visitor);

        assert_eq!(result, Err(SemanticError::ReturnOutsideFunction));
    }

    #[test]
    fn empty_return_statement_in_a_function_with_return_type_is_invalid() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.state.add_global_function(
            crate::semantic::SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), 0, true),
        );

        let return_statement = Statement::return_(Return::new(None));
        let result = return_statement.accept(&mut visitor);

        assert_eq!(result, Err(SemanticError::MissingReturnExpression));
    }

    #[test]
    fn return_statement_with_value_in_a_function_with_no_return_type_is_invalid() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.state.add_global_function(
            crate::semantic::SymbolId(0),
            FunctionMetadata::new("log_message".to_string(), 0, false),
        );

        let return_statement = Statement::return_(Return::new(Some(Expression::new(
            ExpressionKind::I32(100),
            0,
        ))));
        let result = return_statement.accept(&mut visitor);

        assert_eq!(result, Err(SemanticError::UnexpectedReturnExpression));
    }

    #[test]
    fn empty_return_statement_in_a_function_with_no_return_type_is_valid() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.state.add_global_function(
            crate::semantic::SymbolId(0),
            FunctionMetadata::new("log_message".to_string(), 0, false),
        );

        let return_statement = Statement::return_(Return::new(None));
        let result = return_statement.accept(&mut visitor);

        assert!(result.is_ok());
    }

    #[test]
    fn return_statement_with_value_in_a_function_with_return_type_is_valid() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.state.add_global_function(
            crate::semantic::SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), 0, true),
        );

        let return_statement = Statement::return_(Return::new(Some(Expression::new(
            ExpressionKind::I32(100),
            0,
        ))));
        let result = return_statement.accept(&mut visitor);

        assert!(result.is_ok());
    }

    #[test]
    fn return_resolves_identifiers_in_expression() {
        let mut visitor = SymbolResolutionVisitor::new();
        let expected_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("score".to_string(), expected_symbol_id);
        visitor.state.add_global_function(
            SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), 0, true),
        );

        let expression_kind = ExpressionKind::identifier("score".to_string());
        let score_node_id = expression_kind.node_id().unwrap();
        let return_statement =
            Statement::return_(Return::new(Some(Expression::new(expression_kind, 0))));
        let result = return_statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&score_node_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn return_fails_if_expression_has_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.state.add_global_function(
            SymbolId(0),
            FunctionMetadata::new("calculate".to_string(), 0, true),
        );

        let expression_kind = ExpressionKind::identifier("score".to_string());
        let return_statement =
            Statement::return_(Return::new(Some(Expression::new(expression_kind, 0))));
        let result = return_statement.accept(&mut visitor);

        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }
}

#[cfg(test)]
mod print_tests {
    use super::*;
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::{Print, Statement};
    use crate::semantic::SymbolId;

    #[test]
    fn print_resolves_identifiers_in_arguments() {
        let mut visitor = SymbolResolutionVisitor::new();
        let expected_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("score".to_string(), expected_symbol_id);

        let argument_expression_kind = ExpressionKind::identifier("score".to_string());
        let score_node_id = argument_expression_kind.node_id().unwrap();
        let print_statement = Statement::print(Print::new(vec![Expression::new(
            argument_expression_kind,
            0,
        )]));
        let result = print_statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&score_node_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn print_fails_if_argument_has_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();

        let argument_expression_kind = ExpressionKind::identifier("score".to_string());
        let print_statement = Statement::print(Print::new(vec![Expression::new(
            argument_expression_kind,
            0,
        )]));
        let result = print_statement.accept(&mut visitor);

        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }
}

#[cfg(test)]
mod unreachable_code_tests {
    use super::*;
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::{
        Block, Break, FunctionDefinition, Return, Statement, VariableDeclaration,
    };

    #[test]
    fn unreachable_statement_after_return_in_function_body_returns_error() {
        let mut visitor = SymbolResolutionVisitor::new();

        let return_statement = Statement::return_(Return::new(None));
        let variable_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));

        let function_definition = Statement::function_definition(FunctionDefinition::new(
            "calculate".to_string(),
            vec![],
            None,
            Block::new(vec![return_statement, variable_declaration]),
        ));

        let result = function_definition.accept(&mut visitor);
        assert_eq!(result, Err(SemanticError::UnreachableCode));
    }

    #[test]
    fn unreachable_statement_after_break_in_loop_body_returns_error() {
        let mut visitor = SymbolResolutionVisitor::new();

        let break_statement = Statement::control_flow(Break::new());
        let variable_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));

        let loop_statement = Statement::iteration(Loop::new(Block::new(vec![
            break_statement,
            variable_declaration,
        ])));
        let result = loop_statement.accept(&mut visitor);
        assert_eq!(result, Err(SemanticError::UnreachableCode));
    }

    #[test]
    fn unreachable_nested_block_after_return_in_function_body_returns_error() {
        let mut visitor = SymbolResolutionVisitor::new();

        let return_statement = Statement::return_(Return::new(None));
        let nested_block = Statement::block(Block::new(vec![]));

        let function_definition = Statement::function_definition(FunctionDefinition::new(
            "calculate".to_string(),
            vec![],
            None,
            Block::new(vec![return_statement, nested_block]),
        ));

        let result = function_definition.accept(&mut visitor);
        assert_eq!(result, Err(SemanticError::UnreachableCode));
    }

    #[test]
    fn statement_after_conditional_return_in_if_is_reachable() {
        let mut visitor = SymbolResolutionVisitor::new();

        let return_statement = Statement::return_(Return::new(None));
        let if_statement = Statement::conditional(crate::ast::statement::If::new(
            Expression::new(ExpressionKind::Boolean(true), 0),
            Block::new(vec![return_statement]),
            None,
        ));

        let variable_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));

        let function_definition = Statement::function_definition(FunctionDefinition::new(
            "calculate".to_string(),
            vec![],
            None,
            Block::new(vec![if_statement, variable_declaration]),
        ));

        let result = function_definition.accept(&mut visitor);
        assert!(result.is_ok());
    }

    #[test]
    fn statement_after_loop_with_break_is_reachable() {
        let mut visitor = SymbolResolutionVisitor::new();

        let break_statement = Statement::control_flow(Break::new());
        let loop_statement =
            Statement::iteration(crate::ast::statement::Loop::new(Block::new(vec![
                break_statement,
            ])));

        let variable_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));

        let function_definition = Statement::function_definition(FunctionDefinition::new(
            "calculate".to_string(),
            vec![],
            None,
            Block::new(vec![loop_statement, variable_declaration]),
        ));

        let result = function_definition.accept(&mut visitor);
        assert!(result.is_ok());
    }

    #[test]
    fn return_in_first_function_does_not_affect_second_function_reachability() {
        let mut visitor = SymbolResolutionVisitor::new();

        let return_statement = Statement::return_(Return::new(None));
        let first_function = Statement::function_definition(FunctionDefinition::new(
            "first".to_string(),
            vec![],
            None,
            Block::new(vec![return_statement]),
        ));

        let variable_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));
        let second_function = Statement::function_definition(FunctionDefinition::new(
            "second".to_string(),
            vec![],
            None,
            Block::new(vec![variable_declaration]),
        ));

        assert!(first_function.accept(&mut visitor).is_ok());
        assert!(second_function.accept(&mut visitor).is_ok());
    }
}

#[cfg(test)]
mod identifier_expression_tests {
    use super::*;
    use crate::ast::expr::ExpressionKind;
    use crate::semantic::SymbolId;

    #[test]
    fn visitor_resolves_valid_identifier_expression() {
        let mut visitor = SymbolResolutionVisitor::new();

        let expected_symbol_id = SymbolId(10);
        visitor
            .scopes
            .define("score".to_string(), expected_symbol_id);

        let identifier_expression_kind = ExpressionKind::identifier("score".to_string());
        let node_id = identifier_expression_kind.node_id().unwrap();

        let result = identifier_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&node_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn visitor_fails_for_undefined_identifier_expression() {
        let mut visitor = SymbolResolutionVisitor::new();

        let identifier_expression_kind = ExpressionKind::identifier("score".to_string());
        let result = identifier_expression_kind.accept(&mut visitor);

        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }
}

#[cfg(test)]
mod unary_expression_tests {
    use super::*;
    use crate::ast::expr::{ExpressionKind, UnaryOperator};
    use crate::semantic::SymbolId;

    #[test]
    fn visitor_resolves_identifier_inside_unary_expression() {
        let mut visitor = SymbolResolutionVisitor::new();

        let expected_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("score".to_string(), expected_symbol_id);

        let operand_expression_kind = ExpressionKind::identifier("score".to_string());
        let score_node_id = operand_expression_kind.node_id().unwrap();

        let unary_expression_kind =
            ExpressionKind::Unary(Box::new(operand_expression_kind), UnaryOperator::Minus);

        let result = unary_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&score_node_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn visitor_fails_if_operand_has_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();

        let operand_expression_kind = ExpressionKind::identifier("score".to_string());
        let unary_expression_kind =
            ExpressionKind::Unary(Box::new(operand_expression_kind), UnaryOperator::Minus);

        let result = unary_expression_kind.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }
}

#[cfg(test)]
mod binary_expression_tests {
    use super::*;
    use crate::ast::expr::{BinaryOperator, ExpressionKind};
    use crate::semantic::SymbolId;

    #[test]
    fn visitor_resolves_identifiers_inside_binary_expression() {
        let mut visitor = SymbolResolutionVisitor::new();

        let score_symbol_id = SymbolId(1);
        visitor.scopes.define("score".to_string(), score_symbol_id);

        let bonus_symbol_id = SymbolId(2);
        visitor.scopes.define("bonus".to_string(), bonus_symbol_id);

        let left_expression_kind = ExpressionKind::identifier("score".to_string());
        let score_node_id = left_expression_kind.node_id().unwrap();

        let right_expression_kind = ExpressionKind::identifier("bonus".to_string());
        let bonus_node_id = right_expression_kind.node_id().unwrap();

        let binary_expression_kind = ExpressionKind::Binary(
            Box::new(left_expression_kind),
            BinaryOperator::Plus,
            Box::new(right_expression_kind),
        );

        let result = binary_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&score_node_id),
            Some(score_symbol_id)
        );
        assert_eq!(
            visitor.resolution_table.get(&bonus_node_id),
            Some(bonus_symbol_id)
        );
    }

    #[test]
    fn visitor_fails_if_left_operand_has_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.scopes.define("bonus".to_string(), SymbolId(1));

        let left_expression_kind = ExpressionKind::identifier("score".to_string());
        let right_expression_kind = ExpressionKind::identifier("bonus".to_string());

        let binary_expression_kind = ExpressionKind::Binary(
            Box::new(left_expression_kind),
            BinaryOperator::Plus,
            Box::new(right_expression_kind),
        );

        let result = binary_expression_kind.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }

    #[test]
    fn visitor_fails_if_right_operand_has_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();
        visitor.scopes.define("score".to_string(), SymbolId(1));

        let left_expression_kind = ExpressionKind::identifier("score".to_string());
        let right_expression_kind = ExpressionKind::identifier("bonus".to_string());

        let binary_expression_kind = ExpressionKind::Binary(
            Box::new(left_expression_kind),
            BinaryOperator::Plus,
            Box::new(right_expression_kind),
        );

        let result = binary_expression_kind.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("bonus".to_string()))
        );
    }
}

#[cfg(test)]
mod grouped_expression_tests {
    use super::*;
    use crate::ast::expr::ExpressionKind;
    use crate::semantic::SymbolId;

    #[test]
    fn visitor_resolves_identifier_inside_grouped_expression() {
        let mut visitor = SymbolResolutionVisitor::new();

        let expected_symbol_id = SymbolId(1);
        visitor
            .scopes
            .define("score".to_string(), expected_symbol_id);

        let operand_expression_kind = ExpressionKind::identifier("score".to_string());
        let score_node_id = operand_expression_kind.node_id().unwrap();

        let grouped_expression_kind = ExpressionKind::Grouped(Box::new(operand_expression_kind));

        let result = grouped_expression_kind.accept(&mut visitor);
        assert!(result.is_ok());
        assert_eq!(
            visitor.resolution_table.get(&score_node_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn visitor_fails_if_operand_has_undefined_variable() {
        let mut visitor = SymbolResolutionVisitor::new();

        let operand_expression_kind = ExpressionKind::identifier("score".to_string());
        let grouped_expression_kind = ExpressionKind::Grouped(Box::new(operand_expression_kind));

        let result = grouped_expression_kind.accept(&mut visitor);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }
}
