use crate::parser::ast::expr::Expression;

pub(crate) mod error;
pub(crate) mod expr;

#[derive(Debug, PartialEq)]
pub(crate) struct AssignmentNode {
    pub(crate) variable: String,
    pub(crate) data_type: Option<String>,
    pub(crate) expression: Option<Expression>,
}

impl AssignmentNode {
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
