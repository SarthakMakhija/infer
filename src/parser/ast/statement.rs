use crate::parser::ast::expr::Expression;

#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    Assignment(Assignment),
}

impl Statement {
    pub(crate) fn assignment(statement: Assignment) -> Self {
        Statement::Assignment(statement)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Assignment {
    pub(crate) variable: String,
    pub(crate) data_type: Option<String>,
    pub(crate) expression: Option<Expression>,
}

impl Assignment {
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
