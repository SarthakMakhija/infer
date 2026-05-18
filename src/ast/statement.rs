use crate::ast::expr::Expression;

#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    VariableDeclaration(VariableDeclaration),
}

impl Statement {
    pub(crate) fn variable_declaration(statement: VariableDeclaration) -> Self {
        Statement::VariableDeclaration(statement)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct VariableDeclaration {
    pub(crate) variable: String,
    pub(crate) data_type: Option<String>,
    pub(crate) expression: Option<Expression>,
}

impl VariableDeclaration {
    pub(crate) fn new(
        variable: String,
        data_type: Option<String>,
        expression: Option<Expression>,
    ) -> Self {
        Self {
            variable,
            data_type,
            expression,
        }
    }
}

#[cfg(test)]
impl VariableDeclaration {
    pub(crate) fn new_with_variable(variable: String) -> Self {
        Self::new(variable, None, None)
    }

    pub(crate) fn new_with_variable_and_type(variable: String, data_type: String) -> Self {
        Self::new(variable, Some(data_type), None)
    }
}
