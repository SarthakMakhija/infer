use crate::ast::expr::Expression;
use crate::ast::expr::{BinaryOperator, ExpressionKind, UnaryOperator};
use crate::ast::statement::NodeId;
use crate::ast::statement::{
    Assignment, Block, FunctionDefinition, If, Loop, Print, Return, Statement, VariableDeclaration,
};
use crate::semantic::error::SemanticError;
use crate::semantic::inference::constraints::{Constraint, Constraints};
use crate::semantic::inference::type_table::TypeTable;
use crate::semantic::inference::{BinaryOperatorSignature, Type, UnaryOperatorSignature};
use crate::semantic::resolution_table::ResolutionTable;
use crate::semantic::scoping::state::FunctionMetadata;
use crate::semantic::visitor::{ExpressionVisitor, StatementVisitor};
use crate::semantic::SymbolId;
use std::collections::HashMap;

/// AST expression visitor that performs constraint-based type inference.
///
/// It traverses expressions, generates constraints based on the operators and identifiers,
/// and returns the inferred types of expressions.
pub(crate) struct TypeInferenceVisitor<'symbols> {
    constraints: Constraints,
    types: TypeTable,
    symbols: &'symbols ResolutionTable,
    functions: &'symbols HashMap<SymbolId, FunctionMetadata>,
    current_type: Option<Type>,
    current_function_return_type: Option<Type>,
}

impl<'symbols> TypeInferenceVisitor<'symbols> {
    pub(crate) fn new(
        symbols: &'symbols ResolutionTable,
        functions: &'symbols HashMap<SymbolId, FunctionMetadata>,
    ) -> Self {
        Self {
            constraints: Constraints::new(),
            types: TypeTable::new(),
            symbols,
            functions,
            current_type: None,
            current_function_return_type: None,
        }
    }

    /// The entry point to infer the type of any expression.
    pub(crate) fn infer(&mut self, expr: &ExpressionKind) -> Result<Type, SemanticError> {
        expr.accept(self)?;
        Ok(self
            .current_type
            .take()
            .expect("Visitor should have set current_type"))
    }

    /// Recursively visits any statement node to perform type inference and constraint collection.
    pub(crate) fn visit(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        statement.accept(self)
    }

    fn symbol_id(&self, node_id: &NodeId) -> SymbolId {
        self.symbols
            .get(node_id)
            .unwrap_or_else(|| panic!("symbol not found for {:?}", node_id))
    }
}

impl<'symbols> StatementVisitor for TypeInferenceVisitor<'symbols> {
    /// Performs type inference for a variable declaration statement.
    ///
    /// Generates a type for the variable (using the type annotation, or a fresh type placeholder
    /// if unannotated) and records it in the type table. If an initializer expression is present,
    /// its type is inferred and a constraint is added asserting that the initializer type must equal
    /// the variable's type.
    fn visit_var_declaration(
        &mut self,
        variable_declaration: &VariableDeclaration,
        node_id: NodeId,
    ) -> Result<(), SemanticError> {
        let symbol_id = self.symbol_id(&node_id);

        let data_type = match variable_declaration.data_type() {
            None => super::next_type_id(),
            Some(type_str) => Type::try_from(type_str)?,
        };
        if let Some(ref expression) = variable_declaration.expression {
            let initialized_data_type = self.infer(&expression.kind)?;
            self.constraints
                .add(Constraint::new(initialized_data_type, data_type));
        }
        self.types.add(symbol_id, data_type);
        Ok(())
    }

    /// Performs type inference for a variable assignment statement.
    ///
    /// Resolves the target variable's type from the type table using the symbol ID
    /// registered under the assignment node's ID. Then, recursively infers the type of
    /// the assigned expression, and generates a constraint equating the target variable's
    /// type to the expression's type.
    fn visit_assignment(
        &mut self,
        assignment: &Assignment,
        node_id: NodeId,
    ) -> Result<(), SemanticError> {
        let symbol_id = self.symbol_id(&node_id);
        let variable_data_type = self.types.get_or_panic(&symbol_id);

        let inferred_data_type = self.infer(&assignment.expression.kind)?;
        self.constraints
            .add(Constraint::new(variable_data_type, inferred_data_type));
        Ok(())
    }

    /// Performs type inference for an if-else statement.
    ///
    /// Infers the type of the condition expression and constrains it to `Type::Bool`.
    /// Then recursively visits the statements in both the `then` block body and the
    /// `else` block body (if present) to collect constraints from them.
    fn visit_if(&mut self, if_statement: &If) -> Result<(), SemanticError> {
        let condition_data_type = self.infer(&if_statement.condition.kind)?;
        self.constraints
            .add(Constraint::new(condition_data_type, Type::Bool));

        for statement in if_statement.body() {
            self.visit(statement)?;
        }
        if let Some(else_stmts) = if_statement.else_body() {
            for statement in else_stmts {
                self.visit(statement)?;
            }
        }
        Ok(())
    }

    /// Performs type inference for a loop statement.
    ///
    /// Recursively visits all statements in the loop body to collect constraints.
    fn visit_loop(&mut self, block: &Loop) -> Result<(), SemanticError> {
        for statement in block.body() {
            self.visit(statement)?;
        }
        Ok(())
    }

    /// Performs type inference for a block statement.
    ///
    /// Recursively visits all statements inside the block to collect constraints.
    fn visit_block(&mut self, block: &Block) -> Result<(), SemanticError> {
        for statement in block.statements() {
            self.visit(statement)?;
        }
        Ok(())
    }

    /// Performs type inference for a function definition.
    ///
    /// Resolves the expected return type and registers the parameters in the type table using their SymbolIds.
    /// Then recursively visits all statements inside the function body.
    fn visit_function_definition(
        &mut self,
        definition: &FunctionDefinition,
        node_id: NodeId,
    ) -> Result<(), SemanticError> {
        let function_symbol_id = self.symbol_id(&node_id);
        let metadata = self.functions.get(&function_symbol_id).unwrap();

        let expected_return_type = match &metadata.return_type {
            Some(type_str) => Some(Type::try_from(type_str.as_str())?),
            None => None,
        };
        self.current_function_return_type = expected_return_type;

        for (index, parameter_symbol_id) in metadata.parameter_symbols.iter().enumerate() {
            let parameter_type = match &metadata.parameter_types[index] {
                None => super::next_type_id(),
                Some(type_str) => Type::try_from(type_str.as_str())?,
            };
            self.types.add(*parameter_symbol_id, parameter_type);
        }
        for statement in definition.body() {
            self.visit(statement)?;
        }
        self.current_function_return_type = None;
        Ok(())
    }

    /// Performs type inference for a statement-level function call.
    ///
    /// Delegates to expression inference on the call expression's kind.
    fn visit_function_call(&mut self, call: &Expression) -> Result<(), SemanticError> {
        self.infer(&call.kind)?;
        Ok(())
    }

    /// Performs type inference for a break statement.
    ///
    /// A break statement has no type constraints, so this is a no-op.
    fn visit_break(&mut self) -> Result<(), SemanticError> {
        Ok(())
    }

    /// Performs type inference for a return statement.
    ///
    /// Infers the type of the returned value expression and adds a constraint
    /// equating it to the expected return type of the enclosing function.
    fn visit_return(&mut self, return_statement: &Return) -> Result<(), SemanticError> {
        if let Some(ref expression) = return_statement.expression {
            let returned_type = self.infer(&expression.kind)?;
            let expected_type = self
                .current_function_return_type
                .expect("Return statement outside function");
            self.constraints
                .add(Constraint::new(returned_type, expected_type));
        }
        Ok(())
    }

    /// Performs type inference for a print statement.
    ///
    /// Recursively infers the type of all printed argument expressions to collect
    /// constraints from them.
    fn visit_print(&mut self, print_statement: &Print) -> Result<(), SemanticError> {
        for argument in print_statement.arguments() {
            self.infer(&argument.kind)?;
        }
        Ok(())
    }
}

impl<'symbols> ExpressionVisitor for TypeInferenceVisitor<'symbols> {
    /// Infers the type of identifier expression.
    ///
    /// It queries the resolution table using the identifier's `NodeId` to find its unique `SymbolId`.
    /// Then, it looks up the associated `Type` (which may be a concrete type or a type variable placeholder)
    /// in the type table, and sets it as the current expression type.
    fn visit_identifier(&mut self, _name: &str, node_id: NodeId) -> Result<(), SemanticError> {
        let symbol_id = self.symbol_id(&node_id);
        let symbol_type = self.types.get_or_panic(&symbol_id);
        self.current_type = Some(symbol_type);
        Ok(())
    }

    /// Infers the type of function call expression and generates constraints.
    ///
    /// It resolves the callee identifier to its `SymbolId` and retrieves its function metadata.
    /// For each argument expression:
    /// 1. Recursively infers the argument's type.
    /// 2. Resolves the expected parameter type (parsing the string annotation, or generating a fresh type placeholder if unannotated).
    /// 3. Generates a new constraint asserting that the inferred argument type must equal the expected parameter type.
    ///
    /// Finally, it resolves the function's return type (parsing the annotation, or generating a fresh placeholder if unannotated)
    /// and sets it as the type of the entire call expression.
    fn visit_function_call(
        &mut self,
        callee: &ExpressionKind,
        arguments: &[ExpressionKind],
    ) -> Result<(), SemanticError> {
        let ExpressionKind::Identifier(ref _name, callee_node_id) = callee else {
            return Err(SemanticError::NotAFunction("".to_string()));
        };

        let callee_symbol_id = self.symbol_id(callee_node_id);
        let metadata = self
            .functions
            .get(&callee_symbol_id)
            .ok_or_else(|| SemanticError::NotAFunction(_name.to_string()))?;

        for (index, argument) in arguments.iter().enumerate() {
            let inferred_argument_type = self.infer(argument)?;
            let parameter_symbol_id = metadata.parameter_symbols[index];
            let expected_type = self.types.get_or_panic(&parameter_symbol_id);
            self.constraints
                .add(Constraint::new(inferred_argument_type, expected_type));
        }

        let return_type = match &metadata.return_type {
            None => super::next_type_id(),
            Some(type_str) => Type::try_from(type_str.as_str())?,
        };
        self.current_type = Some(return_type);
        Ok(())
    }

    /// Infers the type of unary expression and generates operand constraints.
    ///
    /// It recursively infers the operand expression's type and looks up the expected signature of the operator.
    /// A constraint is added asserting that the operand's type must equal the expected operand type (e.g. `Type::Int32` for numeric negation,
    /// or `Type::Bool` for logical negation). The type of the unary expression is set to the signature's result type.
    fn visit_unary(
        &mut self,
        operator: &UnaryOperator,
        expr: &ExpressionKind,
    ) -> Result<(), SemanticError> {
        let operand_type = self.infer(expr)?;
        let signature = UnaryOperatorSignature::of(operator);

        self.constraints
            .add(Constraint::new(operand_type, signature.operand));
        self.current_type = Some(signature.result);

        Ok(())
    }

    /// Infers the type of binary expression and generates operand constraints.
    ///
    /// It recursively infers the types of both the left and right operand expressions, and retrieves the operator's signature.
    /// Two constraints are added asserting that:
    /// 1. The left operand's type equals the signature's expected left type.
    /// 2. The right operand's type equals the signature's expected right type.
    ///
    /// The type of the binary expression is set to the signature's result type (e.g. `Type::Bool` for comparisons, or `Type::Int32` for arithmetic).
    fn visit_binary(
        &mut self,
        left: &ExpressionKind,
        operator: &BinaryOperator,
        right: &ExpressionKind,
    ) -> Result<(), SemanticError> {
        let left_type = self.infer(left)?;
        let right_type = self.infer(right)?;

        let signature = BinaryOperatorSignature::of(operator);

        self.constraints
            .add(Constraint::new(left_type, signature.left));
        self.constraints
            .add(Constraint::new(right_type, signature.right));
        self.current_type = Some(signature.result);

        Ok(())
    }

    /// Infers the type of grouped (parenthesized) expression.
    ///
    /// Since grouping does not alter types, it recursively infers the type of the inner expression and propagates it.
    fn visit_grouped(&mut self, expr: &ExpressionKind) -> Result<(), SemanticError> {
        let inner_type = self.infer(expr)?;
        self.current_type = Some(inner_type);
        Ok(())
    }

    /// Infers the type of 32-bit signed integer literal, setting it to `Type::Int32`.
    fn visit_i32(&mut self, _value: i32) -> Result<(), SemanticError> {
        self.current_type = Some(Type::Int32);
        Ok(())
    }

    /// Infers the type of UTF-8 string literal, setting it to `Type::String`.
    fn visit_string(&mut self, _value: &str) -> Result<(), SemanticError> {
        self.current_type = Some(Type::String);
        Ok(())
    }

    /// Infers the type of boolean literal, setting it to `Type::Bool`.
    fn visit_bool(&mut self, _value: bool) -> Result<(), SemanticError> {
        self.current_type = Some(Type::Bool);
        Ok(())
    }
}

#[cfg(test)]
mod identifier_expression_tests {
    use super::*;

    #[test]
    fn visit_identifier_when_symbol_present() {
        let identifier_kind = ExpressionKind::Identifier("first_name".to_string(), NodeId(1));
        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Bool);

        let result = identifier_kind.accept(&mut visitor);
        assert!(result.is_ok());
    }

    #[test]
    fn visit_identifier_sets_current_type_to_symbol_type() {
        let identifier_kind = ExpressionKind::Identifier("first_name".to_string(), NodeId(1));
        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Bool);

        let _ = identifier_kind.accept(&mut visitor);
        assert_eq!(visitor.current_type, Some(Type::Bool));
    }

    #[test]
    fn infer_identifier_returns_symbol_type() {
        let identifier_kind = ExpressionKind::Identifier("first_name".to_string(), NodeId(1));
        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Int32);

        let inferred = visitor.infer(&identifier_kind);
        assert_eq!(inferred, Ok(Type::Int32));
    }
}

#[cfg(test)]
mod function_call_tests {
    use super::*;

    #[test]
    fn infer_function_call_with_declared_return_type() {
        let callee = ExpressionKind::Identifier("calculate".to_string(), NodeId(1));
        let call_kind = ExpressionKind::FunctionCall(Box::new(callee), vec![], NodeId(2));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let mut global_functions = HashMap::new();
        global_functions.insert(
            SymbolId(10),
            FunctionMetadata::new("calculate".to_string(), vec![], Some("i32".to_string())),
        );

        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &global_functions);
        let inferred = visitor.infer(&call_kind);

        assert_eq!(inferred, Ok(Type::Int32));
    }

    #[test]
    fn infer_function_call_constrains_arguments_to_parameter_types() {
        let callee = ExpressionKind::Identifier("print_age".to_string(), NodeId(1));
        let arg = ExpressionKind::Identifier("age".to_string(), NodeId(2));
        let call_kind = ExpressionKind::FunctionCall(Box::new(callee), vec![arg], NodeId(3));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));
        resolution_table.resolve(NodeId(2), SymbolId(20));

        let mut global_functions = HashMap::new();
        global_functions.insert(
            SymbolId(10),
            FunctionMetadata::new("print_age".to_string(), vec![Some("i32".to_string())], None)
                .with_symbols(vec![SymbolId(11)]),
        );

        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &global_functions);
        visitor.types.add(SymbolId(11), Type::Int32);
        visitor.types.add(SymbolId(20), Type::Placeholder(1));

        let _ = visitor.infer(&call_kind);
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(1), Type::Int32)
        );
    }

    #[test]
    fn infer_function_call_generates_placeholder_for_absent_parameter_type() {
        let callee = ExpressionKind::Identifier("identity".to_string(), NodeId(1));
        let arg = ExpressionKind::Identifier("value".to_string(), NodeId(2));
        let call_kind = ExpressionKind::FunctionCall(Box::new(callee), vec![arg], NodeId(3));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));
        resolution_table.resolve(NodeId(2), SymbolId(20));

        let mut global_functions = HashMap::new();
        global_functions.insert(
            SymbolId(10),
            FunctionMetadata::new("identity".to_string(), vec![None], None)
                .with_symbols(vec![SymbolId(11)]),
        );

        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &global_functions);
        visitor.types.add(SymbolId(11), Type::Placeholder(2));
        visitor.types.add(SymbolId(20), Type::Placeholder(1));

        let _ = visitor.infer(&call_kind);
        let constraint = visitor.constraints.entry_at(0);
        assert_eq!(constraint.left, Type::Placeholder(1));
    }

    #[test]
    fn infer_function_call_generates_placeholder_for_absent_return_type() {
        let callee = ExpressionKind::Identifier("execute".to_string(), NodeId(1));
        let call_kind = ExpressionKind::FunctionCall(Box::new(callee), vec![], NodeId(2));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let mut global_functions = HashMap::new();
        global_functions.insert(
            SymbolId(10),
            FunctionMetadata::new("execute".to_string(), vec![], None),
        );

        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &global_functions);
        let inferred_type = visitor.infer(&call_kind).unwrap();

        assert!(matches!(inferred_type, Type::Placeholder(_)));
    }
}

#[cfg(test)]
mod literal_expression_tests {
    use super::*;

    #[test]
    fn infer_i32_literal_returns_i32_type() {
        let i32_kind = ExpressionKind::I32(42);
        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&i32_kind);
        assert_eq!(inferred, Ok(Type::Int32));
    }

    #[test]
    fn infer_string_literal_returns_string_type() {
        let string_kind = ExpressionKind::String("hello".to_string());
        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&string_kind);
        assert_eq!(inferred, Ok(Type::String));
    }

    #[test]
    fn infer_bool_literal_returns_bool_type() {
        let bool_kind = ExpressionKind::Boolean(true);
        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&bool_kind);
        assert_eq!(inferred, Ok(Type::Bool));
    }
}

#[cfg(test)]
mod binary_expression_tests {
    use super::*;

    #[test]
    fn infer_binary_with_plus_returns_int32_type() {
        let left = ExpressionKind::I32(10);
        let right = ExpressionKind::I32(20);
        let binary_kind =
            ExpressionKind::Binary(Box::new(left), BinaryOperator::Plus, Box::new(right));

        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&binary_kind);
        assert_eq!(inferred, Ok(Type::Int32));
    }

    #[test]
    fn infer_binary_with_plus_constrains_left_operand_to_int32() {
        let left = ExpressionKind::Identifier("age".to_string(), NodeId(1));
        let right = ExpressionKind::I32(20);
        let binary_kind =
            ExpressionKind::Binary(Box::new(left), BinaryOperator::Plus, Box::new(right));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(1));

        let _ = visitor.infer(&binary_kind);
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(1), Type::Int32)
        );
    }

    #[test]
    fn infer_binary_with_plus_constrains_right_operand_to_int32() {
        let left = ExpressionKind::I32(10);
        let right = ExpressionKind::Identifier("bonus".to_string(), NodeId(1));
        let binary_kind =
            ExpressionKind::Binary(Box::new(left), BinaryOperator::Plus, Box::new(right));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(1));

        let _ = visitor.infer(&binary_kind);
        assert_eq!(
            *visitor.constraints.entry_at(1),
            Constraint::new(Type::Placeholder(1), Type::Int32)
        );
    }

    #[test]
    fn infer_binary_with_greater_than_returns_bool_type() {
        let left = ExpressionKind::I32(10);
        let right = ExpressionKind::I32(20);
        let binary_kind =
            ExpressionKind::Binary(Box::new(left), BinaryOperator::GreaterThan, Box::new(right));

        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&binary_kind);
        assert_eq!(inferred, Ok(Type::Bool));
    }

    #[test]
    fn infer_binary_with_and_returns_bool_type() {
        let left = ExpressionKind::Boolean(true);
        let right = ExpressionKind::Boolean(false);
        let binary_kind =
            ExpressionKind::Binary(Box::new(left), BinaryOperator::And, Box::new(right));

        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&binary_kind);
        assert_eq!(inferred, Ok(Type::Bool));
    }

    #[test]
    fn infer_binary_with_and_constrains_left_operand_to_bool() {
        let left = ExpressionKind::Identifier("is_active".to_string(), NodeId(1));
        let right = ExpressionKind::Boolean(false);
        let binary_kind =
            ExpressionKind::Binary(Box::new(left), BinaryOperator::And, Box::new(right));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(1));

        let _ = visitor.infer(&binary_kind);
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(1), Type::Bool)
        );
    }

    #[test]
    fn infer_binary_with_and_constrains_right_operand_to_bool() {
        let left = ExpressionKind::Boolean(true);
        let right = ExpressionKind::Identifier("has_permission".to_string(), NodeId(1));
        let binary_kind =
            ExpressionKind::Binary(Box::new(left), BinaryOperator::And, Box::new(right));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(1));

        let _ = visitor.infer(&binary_kind);
        assert_eq!(
            *visitor.constraints.entry_at(1),
            Constraint::new(Type::Placeholder(1), Type::Bool)
        );
    }
}

#[cfg(test)]
mod unary_expression_tests {
    use super::*;

    #[test]
    fn infer_unary_minus_returns_int32_type() {
        let operand = ExpressionKind::I32(10);
        let unary_kind = ExpressionKind::Unary(Box::new(operand), UnaryOperator::Minus);

        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&unary_kind);
        assert_eq!(inferred, Ok(Type::Int32));
    }

    #[test]
    fn infer_unary_minus_constrains_operand_to_int32() {
        let operand = ExpressionKind::Identifier("age".to_string(), NodeId(1));
        let unary_kind = ExpressionKind::Unary(Box::new(operand), UnaryOperator::Minus);

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(1));

        let _ = visitor.infer(&unary_kind);
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(1), Type::Int32)
        );
    }

    #[test]
    fn infer_unary_negation_returns_bool_type() {
        let operand = ExpressionKind::Boolean(true);
        let unary_kind = ExpressionKind::Unary(Box::new(operand), UnaryOperator::Negation);

        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&unary_kind);
        assert_eq!(inferred, Ok(Type::Bool));
    }

    #[test]
    fn infer_unary_negation_constrains_operand_to_bool() {
        let operand = ExpressionKind::Identifier("is_active".to_string(), NodeId(1));
        let unary_kind = ExpressionKind::Unary(Box::new(operand), UnaryOperator::Negation);

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(1));

        let _ = visitor.infer(&unary_kind);
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(1), Type::Bool)
        );
    }
}

#[cfg(test)]
mod grouped_expression_tests {
    use super::*;

    #[test]
    fn infer_grouped_returns_inner_expression_type() {
        let inner = ExpressionKind::I32(42);
        let grouped_kind = ExpressionKind::Grouped(Box::new(inner));

        let resolution_table = ResolutionTable::new();
        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let inferred = visitor.infer(&grouped_kind);
        assert_eq!(inferred, Ok(Type::Int32));
    }

    #[test]
    fn infer_grouped_retains_constraints_of_inner_expression() {
        let inner_operand = ExpressionKind::Identifier("age".to_string(), NodeId(1));
        let inner_unary = ExpressionKind::Unary(Box::new(inner_operand), UnaryOperator::Minus);
        let grouped_kind = ExpressionKind::Grouped(Box::new(inner_unary));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(1));

        let _ = visitor.infer(&grouped_kind);
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(1), Type::Int32)
        );
    }
}

#[cfg(test)]
mod variable_declaration_tests {
    use super::*;

    #[test]
    fn infer_var_declaration_with_explicit_annotation() {
        let var_declaration =
            VariableDeclaration::new("score".to_string(), Some("i32".to_string()), None);

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let _ = visitor.visit_var_declaration(&var_declaration, NodeId(1));
        assert_eq!(visitor.types.get_or_panic(&SymbolId(10)), Type::Int32);
    }

    #[test]
    fn infer_var_declaration_without_annotation_generates_placeholder_and_adds_it() {
        let var_declaration = VariableDeclaration::new("score".to_string(), None, None);

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let _ = visitor.visit_var_declaration(&var_declaration, NodeId(1));

        let inferred_type = visitor.types.get_or_panic(&SymbolId(10));
        assert!(matches!(inferred_type, Type::Placeholder(_)));
    }

    #[test]
    fn infer_var_declaration_with_annotation_and_initializer_adds_annotated_type_and_constrains_initializer(
    ) {
        let initializer = Expression {
            kind: ExpressionKind::I32(100),
            line: 1,
        };
        let var_declaration = VariableDeclaration::new(
            "score".to_string(),
            Some("i32".to_string()),
            Some(initializer),
        );

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let _ = visitor.visit_var_declaration(&var_declaration, NodeId(1));

        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Int32, Type::Int32)
        );
    }

    #[test]
    fn infer_var_declaration_without_annotation_and_with_initializer_adds_placeholder_and_constrains_to_initializer(
    ) {
        let initializer = Expression {
            kind: ExpressionKind::I32(100),
            line: 1,
        };
        let var_decl = VariableDeclaration::new("score".to_string(), None, Some(initializer));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let _ = visitor.visit_var_declaration(&var_decl, NodeId(1));

        let inferred_type = visitor.types.get_or_panic(&SymbolId(10));
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Int32, inferred_type)
        );
    }
}

#[cfg(test)]
mod assignment_tests {
    use super::*;

    #[test]
    fn infer_assignment_generates_constraint_equating_target_variable_to_expression_type() {
        let val_expression = Expression {
            kind: ExpressionKind::I32(200),
            line: 1,
        };
        let assignment = Assignment::new("score".to_string(), val_expression);

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(5));

        let _ = visitor.visit_assignment(&assignment, NodeId(1));

        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(5), Type::Int32)
        );
    }

    #[test]
    fn infer_assignment_between_variables_generates_constraint_equating_placeholders() {
        let val_expression = Expression {
            kind: ExpressionKind::Identifier("rank".to_string(), NodeId(2)),
            line: 1,
        };
        let assignment = Assignment::new("score".to_string(), val_expression);

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));
        resolution_table.resolve(NodeId(2), SymbolId(20));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(5));
        visitor.types.add(SymbolId(20), Type::Placeholder(6));

        let _ = visitor.visit_assignment(&assignment, NodeId(1));

        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(5), Type::Placeholder(6))
        );
    }
}

#[cfg(test)]
mod if_tests {
    use super::*;

    #[test]
    fn infer_if_constrains_condition_to_boolean_and_visits_branches() {
        let condition_expression = Expression {
            kind: ExpressionKind::Identifier("condition".to_string(), NodeId(1)),
            line: 1,
        };
        let then_stmt = Statement::VariableDeclaration(
            VariableDeclaration::new("score".to_string(), Some("i32".to_string()), None),
            NodeId(2),
        );
        let else_stmt = Statement::VariableDeclaration(
            VariableDeclaration::new("rank".to_string(), Some("bool".to_string()), None),
            NodeId(3),
        );
        let if_stmt = If::new(
            condition_expression,
            Block::new(vec![then_stmt]),
            Some(Block::new(vec![else_stmt])),
        );

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));
        resolution_table.resolve(NodeId(2), SymbolId(20));
        resolution_table.resolve(NodeId(3), SymbolId(30));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(1));

        let _ = visitor.visit_if(&if_stmt);

        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(1), Type::Bool)
        );
        assert_eq!(visitor.types.get_or_panic(&SymbolId(20)), Type::Int32);
        assert_eq!(visitor.types.get_or_panic(&SymbolId(30)), Type::Bool);
    }
}

#[cfg(test)]
mod loop_tests {
    use super::*;
    use crate::ast::statement::Break;

    #[test]
    fn infer_loop_statements_in_body() {
        let body_statement = Statement::VariableDeclaration(
            VariableDeclaration::new("score".to_string(), Some("i32".to_string()), None),
            NodeId(1),
        );
        let break_statement = Statement::Break(Break::new(), NodeId(2));
        let loop_statement = Loop::new(Block::new(vec![body_statement, break_statement]));

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let _ = visitor.visit_loop(&loop_statement);
        assert_eq!(visitor.types.get_or_panic(&SymbolId(10)), Type::Int32);
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn infer_block_statements() {
        let body_statement = Statement::VariableDeclaration(
            VariableDeclaration::new("score".to_string(), Some("i32".to_string()), None),
            NodeId(1),
        );
        let block_statement = Block::new(vec![body_statement]);

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);

        let _ = visitor.visit_block(&block_statement);
        assert_eq!(visitor.types.get_or_panic(&SymbolId(10)), Type::Int32);
    }
}

#[cfg(test)]
mod print_tests {
    use super::*;

    #[test]
    fn infer_print_expressions() {
        let left_expression = ExpressionKind::Identifier("score".to_string(), NodeId(1));
        let right_expression = ExpressionKind::I32(10);
        let add_expression = Expression {
            kind: ExpressionKind::Binary(
                Box::new(left_expression),
                BinaryOperator::Plus,
                Box::new(right_expression),
            ),
            line: 1,
        };
        let print_stmt = Print::new(vec![add_expression]);

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let functions = HashMap::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        visitor.types.add(SymbolId(10), Type::Placeholder(5));

        let _ = visitor.visit_print(&print_stmt);
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Placeholder(5), Type::Int32)
        );
    }
}

#[cfg(test)]
mod function_tests {
    use super::*;
    use crate::ast::statement::FunctionParameter;

    #[test]
    fn infer_function_definition_registers_parameters_in_type_table() {
        let parameter = FunctionParameter::new("value".to_string(), Some("i32".to_string()));
        let function_definition = FunctionDefinition::new(
            "identity".to_string(),
            vec![parameter],
            Some("i32".to_string()),
            Block::new(vec![]),
        );

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let mut functions = HashMap::new();
        let metadata = FunctionMetadata::new(
            "identity".to_string(),
            vec![Some("i32".to_string())],
            Some("i32".to_string()),
        )
        .with_symbols(vec![SymbolId(11)]);

        functions.insert(SymbolId(10), metadata);

        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        let _ = visitor.visit_function_definition(&function_definition, NodeId(1));
        assert_eq!(visitor.types.get_or_panic(&SymbolId(11)), Type::Int32);
    }

    #[test]
    fn infer_function_definition_constrains_return_type_to_expected_type() {
        let parameter = FunctionParameter::new("value".to_string(), Some("i32".to_string()));
        let return_statement = Statement::Return(
            Return::new(Some(Expression {
                kind: ExpressionKind::Identifier("value".to_string(), NodeId(2)),
                line: 1,
            })),
            NodeId(3),
        );
        let function_definition = FunctionDefinition::new(
            "identity".to_string(),
            vec![parameter],
            Some("i32".to_string()),
            Block::new(vec![return_statement]),
        );

        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));
        resolution_table.resolve(NodeId(2), SymbolId(11));

        let mut functions = HashMap::new();
        let metadata = FunctionMetadata::new(
            "identity".to_string(),
            vec![Some("i32".to_string())],
            Some("i32".to_string()),
        )
        .with_symbols(vec![SymbolId(11)]);

        functions.insert(SymbolId(10), metadata);

        let mut visitor = TypeInferenceVisitor::new(&resolution_table, &functions);
        let _ = visitor.visit_function_definition(&function_definition, NodeId(1));
        assert_eq!(
            *visitor.constraints.entry_at(0),
            Constraint::new(Type::Int32, Type::Int32)
        );
    }
}
