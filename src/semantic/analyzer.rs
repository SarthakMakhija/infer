use crate::ast::statement::{Assignment, Block, If, Loop, NodeId, Return, VariableDeclaration};
use crate::semantic::error::SemanticError;
use crate::semantic::next_symbol_id;
use crate::semantic::resolution::ResolutionTable;
use crate::semantic::scope::Scopes;
use crate::semantic::state::State;

pub(crate) trait Visitor {
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

    fn visit_return(&mut self, return_statement: &Return) -> Result<(), SemanticError>;

    fn visit_break(&mut self) -> Result<(), SemanticError>;
}

pub(crate) struct Analyzer {
    scopes: Scopes,
    state: State,
    resolution_table: ResolutionTable,
}

impl Analyzer {
    pub(crate) fn new() -> Self {
        Self {
            scopes: Scopes::new(),
            state: State::new(),
            resolution_table: ResolutionTable::new(),
        }
    }
}

impl Visitor for Analyzer {
    fn visit_var_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> Result<(), SemanticError> {
        //TODO: handle expression later
        let name = variable_declaration.variable();
        if self.scopes.contains_locally(name) {
            return Err(SemanticError::DuplicateVariable(name.to_string()));
        }
        self.scopes.define(name.to_string(), next_symbol_id());
        Ok(())
    }

    fn visit_assignment(
        &mut self,
        assignment: &Assignment,
        node_id: NodeId,
    ) -> Result<(), SemanticError> {
        //TODO: handle expression later
        let symbol_id = self.scopes.get(assignment.variable());
        if symbol_id.is_none() {
            return Err(SemanticError::UndefinedVariable(
                assignment.variable().to_string(),
            ));
        }
        //SAFETY: symbol_id has been checked for non-none.
        self.resolution_table.resolve(node_id, symbol_id.unwrap());
        Ok(())
    }

    fn visit_if(&mut self, if_statement: &If) -> Result<(), SemanticError> {
        //TODO: handle expression later
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
        self.scopes.begin_scope();
        for statement in block.statements() {
            statement.accept(self)?
        }
        self.scopes.end_scope();
        Ok(())
    }

    fn visit_return(&mut self, return_statement: &Return) -> Result<(), SemanticError> {
        match self.state.current_function() {
            None => Err(SemanticError::ReturnOutsideFunction),
            Some(function_metadata) => {
                if return_statement.expression().is_none() && function_metadata.has_return_type {
                    return Err(SemanticError::MissingReturnExpression);
                }
                if return_statement.expression().is_some() && !function_metadata.has_return_type {
                    return Err(SemanticError::UnexpectedReturnExpression);
                }
                Ok(())
            }
        }
    }

    fn visit_break(&mut self) -> Result<(), SemanticError> {
        if !self.state.is_in_loop() {
            return Err(SemanticError::BreakOutsideLoop);
        }
        Ok(())
    }
}

#[cfg(test)]
mod var_declaration_tests {
    use super::*;
    use crate::ast::statement::Statement;

    #[test]
    fn analyzer_accepts_a_valid_variable_declaration() {
        let mut analyzer = Analyzer::new();
        let declaration = Statement::variable_declaration(VariableDeclaration::new(
            "username".to_string(),
            None,
            None,
        ));

        let result = declaration.accept(&mut analyzer);
        assert!(result.is_ok());
        assert!(analyzer.scopes.contains("username"));
    }

    #[test]
    fn analyzer_detects_duplicate_variable_declarations_in_the_same_scope() {
        let mut analyzer = Analyzer::new();
        let first_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "username".to_string(),
            None,
            None,
        ));
        assert!(first_declaration.accept(&mut analyzer).is_ok());

        let second_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "username".to_string(),
            None,
            None,
        ));
        let result = second_declaration.accept(&mut analyzer);

        assert!(matches!(
            result,
            Err(SemanticError::DuplicateVariable(ref name)) if name == "username"
        ));
    }
}

#[cfg(test)]
mod assignment_tests {
    use super::*;
    use crate::ast::expr::Expression;
    use crate::ast::statement::{Assignment, Statement};

    #[test]
    fn assignment_to_a_defined_variable_succeeds_and_records_resolution() {
        let mut analyzer = Analyzer::new();

        let declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));
        assert!(declaration.accept(&mut analyzer).is_ok());

        let expected_symbol_id = analyzer.scopes.get("score").unwrap();

        let assignment =
            Statement::assignment(Assignment::new("score".to_string(), Expression::I32(100)));
        let assignment_id = assignment.id();

        let result = assignment.accept(&mut analyzer);

        assert!(result.is_ok());
        assert_eq!(
            analyzer.resolution_table.get(&assignment_id),
            Some(expected_symbol_id)
        );
    }

    #[test]
    fn assignment_to_an_undefined_variable_fails_with_semantic_error() {
        let mut analyzer = Analyzer::new();

        let assignment =
            Statement::assignment(Assignment::new("score".to_string(), Expression::I32(100)));

        let result = assignment.accept(&mut analyzer);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("score".to_string()))
        );
    }
}

#[cfg(test)]
mod if_tests {
    use super::*;
    use crate::ast::expr::Expression;
    use crate::ast::statement::{Assignment, Block, If, Statement, VariableDeclaration};

    #[test]
    fn variables_declared_inside_then_block_are_inaccessible_after_if_statement_exits() {
        let mut analyzer = Analyzer::new();

        let then_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "then_var".to_string(),
            None,
            None,
        ));
        let if_statement = Statement::conditional(If::new(
            Expression::Boolean(true),
            Block::new(vec![then_declaration]),
            None,
        ));
        assert!(if_statement.accept(&mut analyzer).is_ok());

        let assignment =
            Statement::assignment(Assignment::new("then_var".to_string(), Expression::I32(10)));
        let result = assignment.accept(&mut analyzer);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("then_var".to_string()))
        );
    }

    #[test]
    fn variables_declared_inside_else_block_are_inaccessible_after_if_statement_exits() {
        let mut analyzer = Analyzer::new();

        let else_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "else_var".to_string(),
            None,
            None,
        ));
        let if_statement = Statement::conditional(If::new(
            Expression::Boolean(false),
            Block::new(vec![]),
            Some(Block::new(vec![else_declaration])),
        ));
        assert!(if_statement.accept(&mut analyzer).is_ok());

        let assignment =
            Statement::assignment(Assignment::new("else_var".to_string(), Expression::I32(10)));
        let result = assignment.accept(&mut analyzer);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("else_var".to_string()))
        );
    }

    #[test]
    fn variables_declared_inside_then_block_are_not_accessible_within_else_block() {
        let mut analyzer = Analyzer::new();

        let then_decl = Statement::variable_declaration(VariableDeclaration::new(
            "first_name".to_string(),
            None,
            None,
        ));

        let else_assign = Statement::assignment(Assignment::new(
            "first_name".to_string(),
            Expression::I32(10),
        ));

        let if_statement = Statement::conditional(If::new(
            Expression::Boolean(true),
            Block::new(vec![then_decl]),
            Some(Block::new(vec![else_assign])),
        ));

        let result = if_statement.accept(&mut analyzer);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("first_name".to_string()))
        );
    }

    #[test]
    fn then_and_else_blocks_can_access_variables_declared_in_outer_scope() {
        let mut analyzer = Analyzer::new();

        let outer_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "outer_var".to_string(),
            None,
            None,
        ));
        assert!(outer_declaration.accept(&mut analyzer).is_ok());
        let expected_symbol_id = analyzer.scopes.get("outer_var").unwrap();

        let then_assign = Statement::assignment(Assignment::new(
            "outer_var".to_string(),
            Expression::I32(10),
        ));
        let then_assign_id = then_assign.id();

        let else_assign = Statement::assignment(Assignment::new(
            "outer_var".to_string(),
            Expression::I32(20),
        ));
        let else_assign_id = else_assign.id();

        let if_statement = Statement::conditional(If::new(
            Expression::Boolean(true),
            Block::new(vec![then_assign]),
            Some(Block::new(vec![else_assign])),
        ));

        assert!(if_statement.accept(&mut analyzer).is_ok());

        assert_eq!(
            analyzer.resolution_table.get(&then_assign_id),
            Some(expected_symbol_id)
        );
        assert_eq!(
            analyzer.resolution_table.get(&else_assign_id),
            Some(expected_symbol_id)
        );
    }
}

#[cfg(test)]
mod loop_tests {
    use super::*;
    use crate::ast::expr::Expression;
    use crate::ast::statement::{Assignment, Block, Break, Loop, Statement, VariableDeclaration};

    #[test]
    fn entering_a_loop_updates_the_state_to_be_inside_a_loop() {
        let mut analyzer = Analyzer::new();

        let break_statement = Statement::control_flow(Break::new());
        let loop_statement = Statement::iteration(Loop::new(Block::new(vec![break_statement])));

        let result = loop_statement.accept(&mut analyzer);
        assert!(result.is_ok());
        assert!(!analyzer.state.is_in_loop());
    }

    #[test]
    fn nested_loops_track_state_depth_correctly() {
        let mut analyzer = Analyzer::new();

        let inner_break = Statement::control_flow(Break::new());
        let inner_loop = Statement::iteration(Loop::new(Block::new(vec![inner_break])));

        let outer_break = Statement::control_flow(Break::new());
        let outer_loop = Statement::iteration(Loop::new(Block::new(vec![inner_loop, outer_break])));

        let result = outer_loop.accept(&mut analyzer);
        assert!(result.is_ok());
        assert!(!analyzer.state.is_in_loop());
    }

    #[test]
    fn variables_declared_inside_loop_are_inaccessible_after_loop_exits() {
        let mut analyzer = Analyzer::new();

        let var_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "name".to_string(),
            None,
            None,
        ));
        let loop_statement = Statement::iteration(Loop::new(Block::new(vec![var_declaration])));
        assert!(loop_statement.accept(&mut analyzer).is_ok());

        let assignment = Statement::assignment(Assignment::new(
            "name".to_string(),
            Expression::String("John".to_string()),
        ));
        let result = assignment.accept(&mut analyzer);
        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("name".to_string()))
        );
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use crate::ast::expr::Expression;
    use crate::ast::statement::{Assignment, Block, Statement, VariableDeclaration};

    #[test]
    fn block_creates_a_new_lexical_scope_allowing_shadowing() {
        let mut analyzer = Analyzer::new();

        let outer_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));
        assert!(outer_declaration.accept(&mut analyzer).is_ok());
        let outer_symbol_id = analyzer.scopes.get("score").unwrap();

        let inner_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));

        let block = Statement::block(Block::new(vec![inner_declaration]));
        assert!(block.accept(&mut analyzer).is_ok());

        assert_eq!(analyzer.scopes.get("score"), Some(outer_symbol_id));
    }

    #[test]
    fn variables_declared_inside_block_are_inaccessible_after_block_exits() {
        let mut analyzer = Analyzer::new();

        let declaration = Statement::variable_declaration(VariableDeclaration::new(
            "temp".to_string(),
            None,
            None,
        ));
        let block = Statement::block(Block::new(vec![declaration]));
        assert!(block.accept(&mut analyzer).is_ok());

        // Assign to "temp" outside the block.
        let assignment =
            Statement::assignment(Assignment::new("temp".to_string(), Expression::I32(42)));
        let result = assignment.accept(&mut analyzer);

        assert_eq!(
            result,
            Err(SemanticError::UndefinedVariable("temp".to_string()))
        );
    }

    #[test]
    fn inner_block_can_access_variables_declared_in_enclosing_scope() {
        let mut analyzer = Analyzer::new();

        let outer_declaration = Statement::variable_declaration(VariableDeclaration::new(
            "score".to_string(),
            None,
            None,
        ));
        assert!(outer_declaration.accept(&mut analyzer).is_ok());
        let expected_symbol_id = analyzer.scopes.get("score").unwrap();

        let inner_assignment =
            Statement::assignment(Assignment::new("score".to_string(), Expression::I32(50)));
        let assignment_id = inner_assignment.id();
        let block = Statement::block(Block::new(vec![inner_assignment]));

        assert!(block.accept(&mut analyzer).is_ok());
        assert_eq!(
            analyzer.resolution_table.get(&assignment_id),
            Some(expected_symbol_id)
        );
    }
}

#[cfg(test)]
mod return_tests {
    use super::*;
    use crate::ast::expr::Expression;
    use crate::ast::statement::Statement;
    use crate::semantic::state::FunctionMetadata;

    #[test]
    fn return_statement_outside_any_function_is_invalid() {
        let mut analyzer = Analyzer::new();

        let return_statement = Statement::return_(Return::new(None));
        let result = return_statement.accept(&mut analyzer);

        assert_eq!(result, Err(SemanticError::ReturnOutsideFunction));
    }

    #[test]
    fn empty_return_statement_in_a_function_with_return_type_is_invalid() {
        let mut analyzer = Analyzer::new();
        analyzer
            .state
            .entered_function(FunctionMetadata::new("calculate".to_string(), true));

        let return_statement = Statement::return_(Return::new(None));
        let result = return_statement.accept(&mut analyzer);

        assert_eq!(result, Err(SemanticError::MissingReturnExpression));
    }

    #[test]
    fn return_statement_with_value_in_a_function_with_no_return_type_is_invalid() {
        let mut analyzer = Analyzer::new();
        analyzer
            .state
            .entered_function(FunctionMetadata::new("log_message".to_string(), false));

        let return_statement = Statement::return_(Return::new(Some(Expression::I32(100))));
        let result = return_statement.accept(&mut analyzer);

        assert_eq!(result, Err(SemanticError::UnexpectedReturnExpression));
    }

    #[test]
    fn empty_return_statement_in_a_function_with_no_return_type_is_valid() {
        let mut analyzer = Analyzer::new();
        analyzer
            .state
            .entered_function(FunctionMetadata::new("log_message".to_string(), false));

        let return_statement = Statement::return_(Return::new(None));
        let result = return_statement.accept(&mut analyzer);

        assert!(result.is_ok());
    }

    #[test]
    fn return_statement_with_value_in_a_function_with_return_type_is_valid() {
        let mut analyzer = Analyzer::new();
        analyzer
            .state
            .entered_function(FunctionMetadata::new("calculate".to_string(), true));

        let return_statement = Statement::return_(Return::new(Some(Expression::I32(100))));
        let result = return_statement.accept(&mut analyzer);

        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod break_tests {
    use super::*;
    use crate::ast::statement::{Break, Statement};

    #[test]
    fn break_statement_outside_any_loop_is_invalid() {
        let mut analyzer = Analyzer::new();
        let break_statement = Statement::control_flow(Break::new());

        let result = break_statement.accept(&mut analyzer);
        assert_eq!(result, Err(SemanticError::BreakOutsideLoop));
    }

    #[test]
    fn break_statement_inside_a_loop_is_valid() {
        let mut analyzer = Analyzer::new();
        analyzer.state.entered_loop();

        let break_statement = Statement::control_flow(Break::new());
        let result = break_statement.accept(&mut analyzer);

        assert!(result.is_ok());
    }
}
