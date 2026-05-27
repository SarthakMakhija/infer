use crate::ast::expr::Expression;

#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    VariableDeclaration(VariableDeclaration),
    Assignment(Assignment),
    Conditional(Conditional),
}

impl Statement {
    pub(crate) fn variable_declaration(statement: VariableDeclaration) -> Self {
        Statement::VariableDeclaration(statement)
    }

    pub(crate) fn assignment(statement: Assignment) -> Self {
        Statement::Assignment(statement)
    }

    pub(crate) fn conditional(statement: Conditional) -> Self {
        Statement::Conditional(statement)
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

#[derive(Debug, PartialEq)]
pub(crate) struct Assignment {
    pub(crate) variable: String,
    pub(crate) expression: Expression,
}

impl Assignment {
    pub(crate) fn new(variable: String, expression: Expression) -> Self {
        Self {
            variable,
            expression,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Conditional {
    pub(crate) condition: Expression,
    pub(crate) body: Vec<Statement>,
    pub(crate) else_body: Option<Vec<Statement>>,
}

impl Conditional {
    pub(crate) fn new(
        condition: Expression,
        body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    ) -> Self {
        Self {
            condition,
            body,
            else_body,
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
