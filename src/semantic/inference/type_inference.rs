use crate::ast::expr::{BinaryOperator, ExpressionKind, UnaryOperator};
use crate::ast::statement::NodeId;
use crate::semantic::error::SemanticError;
use crate::semantic::inference::constraints::{Constraint, Constraints};
use crate::semantic::inference::type_table::TypeTable;
use crate::semantic::inference::{BinaryOperatorSignature, Type, UnaryOperatorSignature};
use crate::semantic::resolution_table::ResolutionTable;
use crate::semantic::scoping::state::FunctionMetadata;
use crate::semantic::visitor::ExpressionVisitor;
use crate::semantic::SymbolId;
use std::collections::HashMap;

pub(crate) struct TypeInferenceVisitor<'symbols> {
    constraints: Constraints,
    types: TypeTable,
    symbols: &'symbols ResolutionTable,
    functions: &'symbols HashMap<SymbolId, FunctionMetadata>,
    current_type: Option<Type>,
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

    fn symbol_id(&self, node_id: &NodeId) -> SymbolId {
        self.symbols
            .get(node_id)
            .unwrap_or_else(|| panic!("symbol not found for {:?}", node_id))
    }
}

impl<'symbols> ExpressionVisitor for TypeInferenceVisitor<'symbols> {
    fn visit_identifier(&mut self, _name: &str, node_id: NodeId) -> Result<(), SemanticError> {
        let symbol_id = self.symbol_id(&node_id);
        let symbol_type = self.types.get_or_panic(&symbol_id);
        self.current_type = Some(symbol_type);
        Ok(())
    }

    fn visit_function_call(
        &mut self,
        _callee: &ExpressionKind,
        _arguments: &[ExpressionKind],
        _node_id: NodeId,
    ) -> Result<(), SemanticError> {
        todo!()
    }

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

    fn visit_grouped(&mut self, expr: &ExpressionKind) -> Result<(), SemanticError> {
        let inner_type = self.infer(expr)?;
        self.current_type = Some(inner_type);
        Ok(())
    }

    fn visit_i32(&mut self, _value: i32) -> Result<(), SemanticError> {
        self.current_type = Some(Type::Int32);
        Ok(())
    }

    fn visit_string(&mut self, _value: &str) -> Result<(), SemanticError> {
        self.current_type = Some(Type::String);
        Ok(())
    }

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
