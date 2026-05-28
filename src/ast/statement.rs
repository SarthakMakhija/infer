use crate::ast::expr::Expression;

#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    VariableDeclaration(VariableDeclaration),
    Assignment(Assignment),
    If(If),
    Loop(Loop),
    Break(Break),
}

impl Statement {
    pub(crate) fn variable_declaration(statement: VariableDeclaration) -> Self {
        Statement::VariableDeclaration(statement)
    }

    pub(crate) fn assignment(statement: Assignment) -> Self {
        Statement::Assignment(statement)
    }

    pub(crate) fn conditional(statement: If) -> Self {
        Statement::If(statement)
    }

    pub(crate) fn iteration(statement: Loop) -> Self {
        Statement::Loop(statement)
    }

    pub(crate) fn control_flow(statement: Break) -> Self {
        Statement::Break(statement)
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
pub(crate) struct If {
    pub(crate) condition: Expression,
    pub(crate) body: Vec<Statement>,
    pub(crate) else_body: Option<Vec<Statement>>,
}

impl If {
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

#[derive(Debug, PartialEq)]
pub(crate) struct Loop {
    pub(crate) body: Vec<Statement>,
}

impl Loop {
    pub(crate) fn new(body: Vec<Statement>) -> Self {
        Self { body }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Break;

impl Break {
    pub(crate) fn new() -> Self {
        Break {}
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
