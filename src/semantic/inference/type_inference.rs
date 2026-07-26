use crate::ast::expr::ExpressionKind;
use crate::ast::statement::NodeId;
use crate::semantic::error::SemanticError;
use crate::semantic::inference::constraints::Constraints;
use crate::semantic::inference::type_table::TypeTable;
use crate::semantic::inference::Type;
use crate::semantic::resolution_table::ResolutionTable;
use crate::semantic::visitor::ExpressionVisitor;
use crate::semantic::SymbolId;

pub(crate) struct TypeInferenceVisitor<'symbols> {
    constraints: Constraints,
    types: TypeTable,
    symbols: &'symbols ResolutionTable,
    current_type: Option<Type>,
}

impl<'symbols> TypeInferenceVisitor<'symbols> {
    pub(crate) fn new(symbols: &'symbols ResolutionTable) -> Self {
        Self {
            constraints: Constraints::new(),
            types: TypeTable::new(),
            symbols,
            current_type: None,
        }
    }

    pub(crate) fn infer(&mut self, expression: &ExpressionKind) -> Result<Type, SemanticError> {
        match expression {
            ExpressionKind::I32(_) => Ok(Type::Int32),
            ExpressionKind::String(_) => Ok(Type::String),
            ExpressionKind::Boolean(_) => Ok(Type::Bool),
            _ => {
                expression.accept(self)?;
                Ok(self
                    .current_type
                    .take()
                    .expect("Visitor should have set current_type"))
            }
        }
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
    ) -> Result<(), SemanticError> {
        todo!()
    }

    fn visit_unary(&mut self, _expr: &ExpressionKind) -> Result<(), SemanticError> {
        todo!()
    }

    fn visit_binary(
        &mut self,
        _left: &ExpressionKind,
        _right: &ExpressionKind,
    ) -> Result<(), SemanticError> {
        todo!()
    }

    fn visit_grouped(&mut self, _expr: &ExpressionKind) -> Result<(), SemanticError> {
        todo!()
    }
}

#[cfg(test)]
mod identifier_expression_tests {
    use super::*;

    #[test]
    fn visit_identifier_when_symbol_is_present() {
        let identifier_kind = ExpressionKind::Identifier("first_name".to_string(), NodeId(1));
        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let mut visitor = TypeInferenceVisitor::new(&resolution_table);
        visitor.types.add(SymbolId(10), Type::Bool);

        let result = identifier_kind.accept(&mut visitor);
        assert!(result.is_ok());
    }

    #[test]
    fn visit_identifier_sets_current_type_to_symbol_type() {
        let identifier_kind = ExpressionKind::Identifier("first_name".to_string(), NodeId(1));
        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let mut visitor = TypeInferenceVisitor::new(&resolution_table);
        visitor.types.add(SymbolId(10), Type::Bool);

        let _ = identifier_kind.accept(&mut visitor);
        assert_eq!(visitor.current_type, Some(Type::Bool));
    }

    #[test]
    fn infer_identifier_returns_symbol_type() {
        let identifier_kind = ExpressionKind::Identifier("first_name".to_string(), NodeId(1));
        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let mut visitor = TypeInferenceVisitor::new(&resolution_table);
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
        let mut visitor = TypeInferenceVisitor::new(&resolution_table);

        let inferred = visitor.infer(&i32_kind);
        assert_eq!(inferred, Ok(Type::Int32));
    }

    #[test]
    fn infer_string_literal_returns_string_type() {
        let string_kind = ExpressionKind::String("john".to_string());
        let resolution_table = ResolutionTable::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table);

        let inferred = visitor.infer(&string_kind);
        assert_eq!(inferred, Ok(Type::String));
    }

    #[test]
    fn infer_bool_literal_returns_bool_type() {
        let bool_kind = ExpressionKind::Boolean(true);
        let resolution_table = ResolutionTable::new();
        let mut visitor = TypeInferenceVisitor::new(&resolution_table);

        let inferred = visitor.infer(&bool_kind);
        assert_eq!(inferred, Ok(Type::Bool));
    }
}
