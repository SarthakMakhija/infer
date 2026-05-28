use crate::ast::expr::Expression;

#[derive(Debug, PartialEq)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    Assignment(Assignment),
    If(If),
    Loop(Loop),
    Break(Break),
    FunctionDefinition(FunctionDefinition),
    FunctionCall(Expression),
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

    pub(crate) fn function_definition(statement: FunctionDefinition) -> Self {
        Statement::FunctionDefinition(statement)
    }

    pub(crate) fn function_call(expression: Expression) -> Self {
        Statement::FunctionCall(expression)
    }
}

#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
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

    pub fn variable(&self) -> &str {
        &self.variable
    }

    pub fn data_type(&self) -> Option<&str> {
        self.data_type.as_deref()
    }

    pub fn expression(&self) -> Option<&Expression> {
        self.expression.as_ref()
    }
}

#[derive(Debug, PartialEq)]
pub struct Assignment {
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

    pub fn variable(&self) -> &str {
        &self.variable
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}

#[derive(Debug, PartialEq)]
pub struct If {
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

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn body(&self) -> &[Statement] {
        &self.body
    }

    pub fn else_body(&self) -> Option<&[Statement]> {
        self.else_body.as_deref()
    }
}

#[derive(Debug, PartialEq)]
pub struct Loop {
    pub(crate) body: Vec<Statement>,
}

impl Loop {
    pub(crate) fn new(body: Vec<Statement>) -> Self {
        Self { body }
    }

    pub fn body(&self) -> &[Statement] {
        &self.body
    }
}

#[derive(Debug, PartialEq)]
pub struct Break;

impl Break {
    pub(crate) fn new() -> Self {
        Break {}
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionDefinition {
    pub(crate) name: String,
    pub(crate) parameters: Vec<FunctionParameter>,
    pub(crate) return_type: Option<String>,
    pub(crate) body: Vec<Statement>,
}

impl FunctionDefinition {
    pub(crate) fn new(
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: Option<String>,
        body: Vec<Statement>,
    ) -> Self {
        Self {
            name,
            parameters,
            return_type,
            body,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parameters(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    pub fn return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }

    pub fn body(&self) -> &[Statement] {
        &self.body
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionParameter {
    pub(crate) name: String,
    pub(crate) data_type: Option<String>,
}

impl FunctionParameter {
    pub(crate) fn new(name: String, data_type: Option<String>) -> Self {
        Self { name, data_type }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> Option<&str> {
        self.data_type.as_deref()
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
