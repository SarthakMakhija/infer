use crate::ast::statement::VariableDeclaration;
use crate::semantic::error::SemanticError;
use crate::semantic::next_symbol_id;
use crate::semantic::scope::Scopes;

pub(crate) trait Visitor {
    fn visit_var_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
    ) -> Result<(), SemanticError>;
}

pub(crate) struct Analyzer {
    scopes: Scopes,
}

impl Analyzer {
    pub(crate) fn new() -> Self {
        Self {
            scopes: Scopes::new(),
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
}

#[cfg(test)]
mod tests {
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
