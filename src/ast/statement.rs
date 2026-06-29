use crate::ast::expr::Expression;
use crate::semantic::error::SemanticError;
use crate::semantic::visitor::StatementVisitor;
use std::cell::Cell;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct NodeId(pub usize);

impl Deref for NodeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

thread_local! {
    /// A thread-local cell holding an auto-incrementing counter to generate unique ID numbers
    /// for each parsed AST statements and expressions.
    static ID: Cell<NodeId> = const { Cell::new(NodeId(0)) };
}

/// Generates a new, unique statement and expression identifier in a single-threaded execution.
pub(crate) fn next_id() -> NodeId {
    ID.with(|id| {
        let current = id.get();
        let next = NodeId(current.0 + 1);
        id.set(next);
        next
    })
}

/// Represents a structural statement in the toy language's Abstract Syntax Tree (AST).
#[derive(Debug)]
pub enum Statement {
    /// A variable declaration statement (e.g. `var age: int = 30;`).
    VariableDeclaration(VariableDeclaration, NodeId),
    /// A variable assignment statement (e.g. `age = 31;`).
    Assignment(Assignment, NodeId),
    /// An if-else conditional block.
    If(If, NodeId),
    /// A loop iteration block.
    Loop(Loop, NodeId),
    /// A standalone block statement containing a sequence of statements (e.g. `{ var score = 10; }`).
    Block(Block, NodeId),
    /// A function definition statement.
    FunctionDefinition(FunctionDefinition, NodeId),
    /// A standalone expression evaluated as a statement (typically a function call).
    FunctionCall(Expression, NodeId),
    /// A break control flow statement.
    Break(Break, NodeId),
    /// A return statement.
    Return(Return, NodeId),
    /// A print statement.
    Print(Print, NodeId),
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
            (Statement::Block(this, _), Statement::Block(other, _)) => this == other,
            (Statement::FunctionDefinition(this, _), Statement::FunctionDefinition(other, _)) => {
                this == other
            }
            (Statement::FunctionCall(this, _), Statement::FunctionCall(other, _)) => this == other,
            (Statement::Break(this, _), Statement::Break(other, _)) => this == other,
            (Statement::Return(this, _), Statement::Return(other, _)) => this == other,
            (Statement::Print(this, _), Statement::Print(other, _)) => this == other,
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

    /// Wraps a [`Block`] into a [`Statement::Block`].
    pub(crate) fn block(block: Block) -> Self {
        Statement::Block(block, Self::statement_id())
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

    /// Wraps a [`Break`] into a [`Statement::Break`].
    pub(crate) fn control_flow(statement: Break) -> Self {
        Statement::Break(statement, Self::statement_id())
    }

    /// Wraps a [`Return`] into a [`Statement::Return`].
    pub(crate) fn return_(statement: Return) -> Self {
        Statement::Return(statement, Self::statement_id())
    }

    /// Wraps a [`Print`] into a [`Statement::Print`].
    pub(crate) fn print(statement: Print) -> Self {
        Statement::Print(statement, Self::statement_id())
    }

    pub(crate) fn accept(&self, visitor: &mut dyn StatementVisitor) -> Result<(), SemanticError> {
        match self {
            Statement::VariableDeclaration(ref declaration, _) => {
                visitor.visit_var_declaration(declaration)
            }
            Statement::Assignment(ref assignment, id) => visitor.visit_assignment(assignment, *id),
            Statement::If(ref if_statement, _) => visitor.visit_if(if_statement),
            Statement::Loop(ref loop_statement, _) => visitor.visit_loop(loop_statement),
            Statement::Block(ref block, _) => visitor.visit_block(block),
            Statement::FunctionDefinition(ref definition, _) => {
                visitor.visit_function_definition(definition)
            }
            Statement::FunctionCall(ref expression, _) => visitor.visit_function_call(expression),
            Statement::Break(_, _) => visitor.visit_break(),
            Statement::Return(ref return_statement, _) => visitor.visit_return(return_statement),
            _ => unimplemented!(),
        }
    }

    /// Returns the unique id of the statement.
    pub fn id(&self) -> NodeId {
        match self {
            Statement::VariableDeclaration(_, id) => *id,
            Statement::Assignment(_, id) => *id,
            Statement::If(_, id) => *id,
            Statement::Loop(_, id) => *id,
            Statement::Block(_, id) => *id,
            Statement::FunctionDefinition(_, id) => *id,
            Statement::FunctionCall(_, id) => *id,
            Statement::Break(_, id) => *id,
            Statement::Return(_, id) => *id,
            Statement::Print(_, id) => *id,
        }
    }

    fn statement_id() -> NodeId {
        next_id()
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

/// Represents a print statement.
///
/// Example: `print name, age;`
#[derive(Debug, PartialEq)]
pub struct Print {
    pub(crate) arguments: Vec<Expression>,
}

impl Print {
    pub(crate) fn new(arguments: Vec<Expression>) -> Self {
        Self { arguments }
    }

    /// Returns the arguments of the print statement.
    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_declaration_id() {
        let statement = Statement::variable_declaration(VariableDeclaration::new(
            "user_score".to_string(),
            None,
            None,
        ));

        assert!(*statement.id() > 0);
    }

    #[test]
    fn assignment_id() {
        let statement = Statement::assignment(Assignment::new(
            "user_score".to_string(),
            Expression::I32(100),
        ));

        assert!(*statement.id() > 0);
    }

    #[test]
    fn if_id() {
        let statement =
            Statement::conditional(If::new(Expression::Boolean(true), Block::new(vec![]), None));
        assert!(*statement.id() > 0);
    }

    #[test]
    fn loop_id() {
        let statement = Statement::iteration(Loop::new(Block::new(vec![])));
        assert!(*statement.id() > 0);
    }

    #[test]
    fn block_id() {
        let statement = Statement::block(Block::new(vec![]));
        assert!(*statement.id() > 0);
    }

    #[test]
    fn function_definition_id() {
        let statement = Statement::function_definition(FunctionDefinition::new(
            "calculate_total".to_string(),
            vec![],
            None,
            Block::new(vec![]),
        ));
        assert!(*statement.id() > 0);
    }

    #[test]
    fn function_call_id() {
        let statement = Statement::function_call(Expression::I32(42));
        assert!(*statement.id() > 0);
    }

    #[test]
    fn break_id() {
        let statement = Statement::control_flow(Break::new());
        assert!(*statement.id() > 0);
    }

    #[test]
    fn return_id() {
        let statement = Statement::return_(Return::new(Some(Expression::I32(1))));
        assert!(*statement.id() > 0);
    }

    #[test]
    fn print_id() {
        let statement = Statement::print(Print::new(vec![Expression::I32(1)]));
        assert!(*statement.id() > 0);
    }

    #[test]
    fn variable_declaration_id_for_two_statements() {
        let first = Statement::variable_declaration(VariableDeclaration::new(
            "user_score".to_string(),
            None,
            None,
        ));
        let second = Statement::variable_declaration(VariableDeclaration::new(
            "user_score".to_string(),
            None,
            None,
        ));

        assert_eq!(first.id(), NodeId(1));
        assert_eq!(second.id(), NodeId(2));
    }
}

#[cfg(test)]
mod accept_tests {
    use crate::ast::statement::{
        Assignment, Block, Break, FunctionDefinition, If, Loop, NodeId, Return, Statement,
        VariableDeclaration,
    };
    use crate::semantic::error::SemanticError;
    use crate::semantic::visitor::StatementVisitor;

    struct TestVisitor {
        visited_var_declaration: bool,
        visited_assignment: bool,
        visited_if: bool,
        visited_loop: bool,
        visited_block: bool,
        visited_function_definition: bool,
        visited_break: bool,
        visited_return: bool,
    }

    impl StatementVisitor for TestVisitor {
        fn visit_var_declaration(
            &mut self,
            _variable_declaration: &VariableDeclaration,
        ) -> Result<(), SemanticError> {
            self.visited_var_declaration = true;
            Ok(())
        }

        fn visit_assignment(
            &mut self,
            _assignment: &Assignment,
            _node_id: NodeId,
        ) -> Result<(), SemanticError> {
            self.visited_assignment = true;
            Ok(())
        }

        fn visit_if(&mut self, _if_statement: &If) -> Result<(), SemanticError> {
            self.visited_if = true;
            Ok(())
        }

        fn visit_loop(&mut self, _loop_statement: &Loop) -> Result<(), SemanticError> {
            self.visited_loop = true;
            Ok(())
        }

        fn visit_block(&mut self, _block: &Block) -> Result<(), SemanticError> {
            self.visited_block = true;
            Ok(())
        }

        fn visit_function_definition(
            &mut self,
            _definition: &FunctionDefinition,
        ) -> Result<(), SemanticError> {
            self.visited_function_definition = true;
            Ok(())
        }

        fn visit_function_call(
            &mut self,
            _call: &crate::ast::expr::Expression,
        ) -> Result<(), SemanticError> {
            Ok(())
        }

        fn visit_break(&mut self) -> Result<(), SemanticError> {
            self.visited_break = true;
            Ok(())
        }

        fn visit_return(&mut self, _return_statement: &Return) -> Result<(), SemanticError> {
            self.visited_return = true;
            Ok(())
        }
    }

    #[test]
    fn statement_accept_dispatches_variable_declaration_to_visitor() {
        let statement = Statement::variable_declaration(VariableDeclaration::new(
            "username".to_string(),
            None,
            None,
        ));
        let mut visitor = TestVisitor {
            visited_var_declaration: false,
            visited_assignment: false,
            visited_if: false,
            visited_loop: false,
            visited_block: false,
            visited_function_definition: false,
            visited_break: false,
            visited_return: false,
        };
        let result = statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert!(visitor.visited_var_declaration);
    }

    #[test]
    fn statement_accept_dispatches_assignment_to_visitor() {
        use crate::ast::expr::Expression;
        let statement =
            Statement::assignment(Assignment::new("score".to_string(), Expression::I32(10)));
        let mut visitor = TestVisitor {
            visited_var_declaration: false,
            visited_assignment: false,
            visited_if: false,
            visited_loop: false,
            visited_block: false,
            visited_function_definition: false,
            visited_break: false,
            visited_return: false,
        };
        let result = statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert!(visitor.visited_assignment);
    }

    #[test]
    fn statement_accept_dispatches_if_to_visitor() {
        use crate::ast::expr::Expression;
        let statement =
            Statement::conditional(If::new(Expression::Boolean(true), Block::new(vec![]), None));
        let mut visitor = TestVisitor {
            visited_var_declaration: false,
            visited_assignment: false,
            visited_if: false,
            visited_loop: false,
            visited_block: false,
            visited_function_definition: false,
            visited_break: false,
            visited_return: false,
        };
        let result = statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert!(visitor.visited_if);
    }

    #[test]
    fn statement_accept_dispatches_loop_to_visitor() {
        let statement = Statement::iteration(Loop::new(Block::new(vec![])));
        let mut visitor = TestVisitor {
            visited_var_declaration: false,
            visited_assignment: false,
            visited_if: false,
            visited_loop: false,
            visited_block: false,
            visited_function_definition: false,
            visited_break: false,
            visited_return: false,
        };
        let result = statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert!(visitor.visited_loop);
    }

    #[test]
    fn statement_accept_dispatches_block_to_visitor() {
        let statement = Statement::block(Block::new(vec![]));
        let mut visitor = TestVisitor {
            visited_var_declaration: false,
            visited_assignment: false,
            visited_if: false,
            visited_loop: false,
            visited_block: false,
            visited_function_definition: false,
            visited_break: false,
            visited_return: false,
        };
        let result = statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert!(visitor.visited_block);
    }

    #[test]
    fn statement_accept_dispatches_function_definition_to_visitor() {
        let statement = Statement::function_definition(FunctionDefinition::new(
            "calculate".to_string(),
            vec![],
            None,
            Block::new(vec![]),
        ));
        let mut visitor = TestVisitor {
            visited_var_declaration: false,
            visited_assignment: false,
            visited_if: false,
            visited_loop: false,
            visited_block: false,
            visited_function_definition: false,
            visited_break: false,
            visited_return: false,
        };
        let result = statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert!(visitor.visited_function_definition);
    }

    #[test]
    fn statement_accept_dispatches_break_to_visitor() {
        let statement = Statement::control_flow(Break::new());
        let mut visitor = TestVisitor {
            visited_var_declaration: false,
            visited_assignment: false,
            visited_if: false,
            visited_loop: false,
            visited_block: false,
            visited_function_definition: false,
            visited_break: false,
            visited_return: false,
        };
        let result = statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert!(visitor.visited_break);
    }

    #[test]
    fn statement_accept_dispatches_return_to_visitor() {
        let statement = Statement::return_(Return::new(None));
        let mut visitor = TestVisitor {
            visited_var_declaration: false,
            visited_assignment: false,
            visited_if: false,
            visited_loop: false,
            visited_block: false,
            visited_function_definition: false,
            visited_break: false,
            visited_return: false,
        };
        let result = statement.accept(&mut visitor);

        assert!(result.is_ok());
        assert!(visitor.visited_return);
    }
}
