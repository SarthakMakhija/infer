use crate::ast::expr::ExpressionKind;
use crate::ast::statement::NodeId;
use crate::semantic::error::SemanticError;
use crate::semantic::inference::constraints::Constraints;
use crate::semantic::inference::type_table::TypeTable;
use crate::semantic::inference::Type;
use crate::semantic::resolution_table::ResolutionTable;
use crate::semantic::SymbolId;

pub(crate) struct TypeInference<'symbols> {
    constraints: Constraints,
    types: TypeTable,
    symbols: &'symbols ResolutionTable,
}

impl<'symbols> TypeInference<'symbols> {
    pub(crate) fn new(symbols: &'symbols ResolutionTable) -> Self {
        Self {
            constraints: Constraints::new(),
            types: TypeTable::new(),
            symbols,
        }
    }

    /// The entry point to infer the type of any expression.
    pub(crate) fn infer(&mut self, expr: &ExpressionKind) -> Result<Type, SemanticError> {
        match expr {
            ExpressionKind::I32(_) => Ok(Type::Int32),
            ExpressionKind::String(_) => Ok(Type::String),
            ExpressionKind::Boolean(_) => Ok(Type::Bool),
            ExpressionKind::Identifier(_, node_id) => {
                let symbol_id = self.symbol_id(node_id);
                Ok(self.types.get_or_panic(&symbol_id))
            }
            _ => todo!("Inference for this expression kind is not implemented yet"),
        }
    }

    fn symbol_id(&self, node_id: &NodeId) -> SymbolId {
        self.symbols
            .get(node_id)
            .unwrap_or_else(|| panic!("symbol not found for {:?}", node_id))
    }
}

#[cfg(test)]
mod identifier_expression_tests {
    use super::*;

    #[test]
    fn infer_identifier_returns_symbol_type() {
        let identifier_kind = ExpressionKind::Identifier("first_name".to_string(), NodeId(1));
        let mut resolution_table = ResolutionTable::new();
        resolution_table.resolve(NodeId(1), SymbolId(10));

        let mut visitor = TypeInference::new(&resolution_table);
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
        let mut visitor = TypeInference::new(&resolution_table);

        let inferred = visitor.infer(&i32_kind);

        assert_eq!(inferred, Ok(Type::Int32));
    }

    #[test]
    fn infer_string_literal_returns_string_type() {
        let string_kind = ExpressionKind::String("john".to_string());
        let resolution_table = ResolutionTable::new();
        let mut visitor = TypeInference::new(&resolution_table);

        let inferred = visitor.infer(&string_kind);
        assert_eq!(inferred, Ok(Type::String));
    }

    #[test]
    fn infer_bool_literal_returns_bool_type() {
        let bool_kind = ExpressionKind::Boolean(true);
        let resolution_table = ResolutionTable::new();
        let mut visitor = TypeInference::new(&resolution_table);

        let inferred = visitor.infer(&bool_kind);

        assert_eq!(inferred, Ok(Type::Bool));
    }
}
