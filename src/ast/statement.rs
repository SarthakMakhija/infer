use crate::ast::expr::Expression;
use std::cell::Cell;

thread_local! {
    static STATEMENT_ID: Cell<usize> = Cell::new(0);
}

fn next_statement_id() -> usize {
    STATEMENT_ID.with(|counter| {
        let next = counter.get() + 1;
        counter.set(next);
        next
    })
}

/// Represents a structural statement in the toy language's Abstract Syntax Tree (AST).
#[derive(Debug)]
pub enum Statement {
    /// A variable declaration statement (e.g. `var age: int = 30;`).
    VariableDeclaration(VariableDeclaration, usize),
    /// A variable assignment statement (e.g. `age = 31;`).
    Assignment(Assignment, usize),
    /// An if-else conditional block.
    If(If, usize),
    /// A loop iteration block.
    Loop(Loop, usize),
    /// A break control flow statement.
    Break(Break, usize),
    /// A function definition statement.
    FunctionDefinition(FunctionDefinition, usize),
    /// A standalone expression evaluated as a statement (typically a function call).
    FunctionCall(Expression, usize),
    /// A standalone block statement containing a sequence of statements (e.g. `{ var score = 10; }`).
    Block(Block, usize),
    /// A return statement.
    Return(Return, usize),
}

impl PartialEq for Statement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Statement::VariableDeclaration(this, _), Statement::VariableDeclaration(other, _)) => {
                this == other
            }
            (Statement::Assignment(this, _), Statement::Assignment(other, _)) => this == other,
            (Statement::If(this, _), Statement::If(other, _)) => this == other,
            (Statement::Loop(this, _), Statement::Loop(other, _)) => this == other,
            (Statement::Break(this, _), Statement::Break(other, _)) => this == other,
            (Statement::FunctionDefinition(this, _), Statement::FunctionDefinition(other, _)) => {
                this == other
            }
            (Statement::FunctionCall(this, _), Statement::FunctionCall(other, _)) => this == other,
            (Statement::Block(this, _), Statement::Block(other, _)) => this == other,
            (Statement::Return(this, _), Statement::Return(other, _)) => this == other,
            _ => false,
        }
    }
}

impl Statement {
    /// Wraps a [`VariableDeclaration`] into a [`Statement::VariableDeclaration`].
    pub(crate) fn variable_declaration(statement: VariableDeclaration) -> Self {
        Statement::VariableDeclaration(statement, Self::statement_id())
    }

    /// Wraps an [`Assignment`] into a [`Statement::Assignment`].
    pub(crate) fn assignment(statement: Assignment) -> Self {
        Statement::Assignment(statement, Self::statement_id())
    }

    /// Wraps an [`If`] into a [`Statement::If`].
    pub(crate) fn conditional(statement: If) -> Self {
        Statement::If(statement, Self::statement_id())
    }

    /// Wraps a [`Loop`] into a [`Statement::Loop`].
    pub(crate) fn iteration(statement: Loop) -> Self {
        Statement::Loop(statement, Self::statement_id())
    }

    /// Wraps a [`Break`] into a [`Statement::Break`].
    pub(crate) fn control_flow(statement: Break) -> Self {
        Statement::Break(statement, Self::statement_id())
    }

    /// Wraps a [`Return`] into a [`Statement::Return`].
    pub(crate) fn return_(statement: Return) -> Self {
        Statement::Return(statement, Self::statement_id())
    }

    /// Wraps a [`FunctionDefinition`] into a [`Statement::FunctionDefinition`].
    pub(crate) fn function_definition(statement: FunctionDefinition) -> Self {
        Statement::FunctionDefinition(statement, Self::statement_id())
    }

    /// Wraps a function call [`Expression`] into a [`Statement::FunctionCall`].
    ///
    /// The caller is expected to pass a [`Expression::FunctionCall`] variant.
    pub(crate) fn function_call(expression: Expression) -> Self {
        Statement::FunctionCall(expression, Self::statement_id())
    }

    /// Wraps a [`Block`] into a [`Statement::Block`].
    pub(crate) fn block(block: Block) -> Self {
        Statement::Block(block, Self::statement_id())
    }

    /// Returns the unique id of the statement.
    pub fn id(&self) -> usize {
        match self {
            Statement::VariableDeclaration(_, id) => *id,
            Statement::Assignment(_, id) => *id,
            Statement::If(_, id) => *id,
            Statement::Loop(_, id) => *id,
            Statement::Break(_, id) => *id,
            Statement::FunctionDefinition(_, id) => *id,
            Statement::FunctionCall(_, id) => *id,
            Statement::Block(_, id) => *id,
            Statement::Return(_, id) => *id,
        }
    }

    fn statement_id() -> usize {
        next_statement_id()
    }
}

/// Represents a variable declaration statement with optional type annotation and initialization.
///
/// Example: `var score: int = 100;`
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

    /// Returns the name of the declared variable.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Returns the explicit type annotation of the variable, if provided.
    pub fn data_type(&self) -> Option<&str> {
        self.data_type.as_deref()
    }

    /// Returns the initialization expression of the variable, if provided.
    pub fn expression(&self) -> Option<&Expression> {
        self.expression.as_ref()
    }
}

/// Represents a variable assignment statement.
///
/// Example: `score = 200;`
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

    /// Returns the name of the variable being assigned to.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Returns the expression being assigned to the variable.
    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}

/// Represents an if-else conditional block in the AST.
///
/// Example: `if x == 10 { var y = 1; } else { var y = 2; }`
#[derive(Debug, PartialEq)]
pub struct If {
    pub(crate) condition: Expression,
    pub(crate) body: Block,
    pub(crate) else_body: Option<Block>,
}

impl If {
    pub(crate) fn new(condition: Expression, body: Block, else_body: Option<Block>) -> Self {
        Self {
            condition,
            body,
            else_body,
        }
    }

    /// Returns the condition expression governing the conditional execution.
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    /// Returns a slice of statements in the `then` branch body.
    pub fn body(&self) -> &[Statement] {
        &self.body.statements
    }

    /// Returns a slice of statements in the `else` branch body, if provided.
    pub fn else_body(&self) -> Option<&[Statement]> {
        self.else_body.as_ref().map(|block| block.statements())
    }
}

/// Represents a loop iteration block in the AST.
///
/// Example: `loop { break; }`
#[derive(Debug, PartialEq)]
pub struct Loop {
    pub(crate) body: Block,
}

impl Loop {
    pub(crate) fn new(body: Block) -> Self {
        Self { body }
    }

    /// Returns a slice of statements in the loop body.
    pub fn body(&self) -> &[Statement] {
        &self.body.statements
    }
}

/// Represents a break control flow statement.
///
/// Example: `break;`
#[derive(Debug, PartialEq)]
pub struct Break;

impl Break {
    pub(crate) fn new() -> Self {
        Break {}
    }
}

/// Represents a return control flow statement.
///
/// Example: `return expression;` or `return;`
#[derive(Debug, PartialEq)]
pub struct Return {
    pub(crate) expression: Option<Expression>,
}

impl Return {
    pub(crate) fn new(expression: Option<Expression>) -> Self {
        Self { expression }
    }

    /// Returns the expression being returned, if any.
    pub fn expression(&self) -> Option<&Expression> {
        self.expression.as_ref()
    }
}

/// Represents a function definition statement in the AST.
///
/// Example: `fn add(a: int, b: int): int { var sum = a + b; }`
#[derive(Debug, PartialEq)]
pub struct FunctionDefinition {
    pub(crate) name: String,
    pub(crate) parameters: Vec<FunctionParameter>,
    pub(crate) return_type: Option<String>,
    pub(crate) body: Block,
}

impl FunctionDefinition {
    pub(crate) fn new(
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: Option<String>,
        body: Block,
    ) -> Self {
        Self {
            name,
            parameters,
            return_type,
            body,
        }
    }

    /// Returns the name of the function.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a slice of the function's parameters.
    pub fn parameters(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    /// Returns the explicit return type annotation, if provided.
    pub fn return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }

    /// Returns a slice of statements in the function's body.
    pub fn body(&self) -> &[Statement] {
        &self.body.statements
    }
}

/// Represents a parameter in a function definition signature.
///
/// Example: `first: int`
#[derive(Debug, PartialEq)]
pub struct FunctionParameter {
    pub(crate) name: String,
    pub(crate) data_type: Option<String>,
}

impl FunctionParameter {
    pub(crate) fn new(name: String, data_type: Option<String>) -> Self {
        Self { name, data_type }
    }

    /// Returns the name of the parameter.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the explicit type annotation of the parameter, if provided.
    pub fn data_type(&self) -> Option<&str> {
        self.data_type.as_deref()
    }
}

/// Represents a block statement `{ ... }` in the AST.
///
/// A block contains a sequence of statements executed in a new lexical scope.
#[derive(Debug, PartialEq)]
pub struct Block {
    pub(crate) statements: Vec<Statement>,
}

impl Block {
    /// Creates a new `Block` enclosing the given statements.
    pub(crate) fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    /// Returns a slice of statements in the block.
    pub fn statements(&self) -> &[Statement] {
        &self.statements
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
