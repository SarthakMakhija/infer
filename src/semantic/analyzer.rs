use crate::ast::statement::{Return, VariableDeclaration};
use crate::semantic::error::SemanticError;
use crate::semantic::next_symbol_id;
use crate::semantic::scope::Scopes;
use crate::semantic::state::State;

pub(crate) trait Visitor {
    fn visit_var_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> Result<(), SemanticError>;

    fn visit_return(&mut self, return_statement: &Return) -> Result<(), SemanticError>;

    fn visit_break(&mut self) -> Result<(), SemanticError>;
}

pub(crate) struct Analyzer {
    scopes: Scopes,
    pub(crate) state: State,
}

impl Analyzer {
    pub(crate) fn new() -> Self {
        Self {
            scopes: Scopes::new(),
            state: State::new(),
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
