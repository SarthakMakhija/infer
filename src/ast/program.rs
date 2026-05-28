use crate::ast::statement::Statement;

/// Represents a fully parsed toy language program.
///
/// A `Program` consists of a list of top-level structural statements (like functions
/// and variable declarations) that represent the root of the parsed AST.
#[derive(Debug, PartialEq)]
pub struct Program {
    statements: Vec<Statement>,
}

impl Program {
    pub(crate) fn new(statements: Vec<Statement>) -> Self {
        Program { statements }
    }

    /// Returns a read-only slice of all top-level statements in this program.
    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }
}

pub(crate) struct ProgramBuilder {
    statements: Vec<Statement>,
}

impl ProgramBuilder {
    pub(crate) fn new() -> Self {
        ProgramBuilder {
            statements: Vec::new(),
        }
    }

    pub(crate) fn add(mut self, statement: Statement) -> ProgramBuilder {
        self.statements.push(statement);
        self
    }

    pub(crate) fn build(self) -> Program {
        Program::new(self.statements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::statement::VariableDeclaration;

    #[test]
    fn create_a_program_with_no_statements() {
        let program = ProgramBuilder::new().build();
        assert_eq!(program.statements.len(), 0);
    }

    #[test]
    fn create_a_program_with_a_single_variable_declaration_statement() {
        let statement = Statement::variable_declaration(VariableDeclaration::new_with_variable(
            "id".to_string(),
        ));
        let program = ProgramBuilder::new().add(statement).build();
        assert_eq!(program.statements.len(), 1);

        let actual_statement = program.statements.first().unwrap();
        let expected_statement = Statement::variable_declaration(
            VariableDeclaration::new_with_variable("id".to_string()),
        );
        assert_eq!(actual_statement, &expected_statement);
    }

    #[test]
    fn create_a_program_with_a_single_variable_declaration_statement_with_name_and_type() {
        let statement = Statement::variable_declaration(
            VariableDeclaration::new_with_variable_and_type("id".to_string(), "i32".to_string()),
        );
        let program = ProgramBuilder::new().add(statement).build();
        assert_eq!(program.statements.len(), 1);

        let actual_statement = program.statements.first().unwrap();
        let expected_statement = Statement::variable_declaration(
            VariableDeclaration::new_with_variable_and_type("id".to_string(), "i32".to_string()),
        );
        assert_eq!(actual_statement, &expected_statement);
    }
}
