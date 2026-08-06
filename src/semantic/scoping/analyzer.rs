use crate::ast::program::Program;
use crate::semantic::error::SemanticError;
use crate::semantic::inference::type_inference::TypeInferenceVisitor;
use crate::semantic::inference::unifier::Unifier;
use crate::semantic::scoping::symbol_resolution::SymbolResolutionVisitor;

/// The compiler driver for running semantic analysis passes.
///
/// It orchestrates identifier lookup checks, scoping, loop control validation,
/// unreachable code analysis, and resolves forward-referenced function calls.
pub(crate) struct Analyzer {
    symbol_resolution: SymbolResolutionVisitor,
}

impl Analyzer {
    /// Creates a new `Analyzer`.
    pub(crate) fn new() -> Self {
        Analyzer {
            symbol_resolution: SymbolResolutionVisitor::new(),
        }
    }

    /// Performs semantic analysis on the abstract syntax tree of the program,
    /// orchestrating symbol resolution, name checks, loop constraint verification,
    /// type inference, and unification constraint solving.
    ///
    /// # Errors
    ///
    /// Returns a `SemanticError` if any scoping rule, variable declaration,
    /// or type compatibility constraint is violated.
    pub(crate) fn analyze(&mut self, program: &Program) -> Result<(), SemanticError> {
        self.resolve_symbols(program)?;
        self.infer_types(program)?;
        Ok(())
    }

    fn resolve_symbols(&mut self, program: &Program) -> Result<(), SemanticError> {
        self.symbol_resolution
            .visit_statements(program.statements())?;
        self.symbol_resolution.resolve_pending_calls()?;
        Ok(())
    }

    fn infer_types(&mut self, program: &Program) -> Result<(), SemanticError> {
        let mut type_inference_visitor = TypeInferenceVisitor::new(
            self.symbol_resolution.resolution_table(),
            self.symbol_resolution.global_functions(),
        );
        for statement in program.statements() {
            type_inference_visitor.visit(statement)?;
        }
        Unifier::new().solve(type_inference_visitor.constraints)?;
        Ok(())
    }
}

#[cfg(test)]
mod scoping_resolution {
    use super::*;

    #[cfg(test)]
    mod var_declaration_tests {
        use super::*;

        #[test]
        fn analyze_valid_variable_declaration() {
            let mut analyzer = Analyzer::new();
            let declaration = variable_declaration!("username");
            let program = Program::new(vec![declaration]);
            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }
    }

    #[cfg(test)]
    mod assignment_tests {
        use super::*;

        #[test]
        fn analyze_valid_assignment() {
            let mut analyzer = Analyzer::new();
            let declaration = variable_declaration!("score");
            let assignment = assignment!("score", expression_i32!(100, 0));
            let program = Program::new(vec![declaration, assignment]);

            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }
    }

    #[cfg(test)]
    mod if_tests {
        use super::*;

        #[test]
        fn analyze_valid_if_statement() {
            let mut analyzer = Analyzer::new();
            let if_statement = conditional!(expression_boolean!(true, 0), block!());
            let program = Program::new(vec![if_statement]);

            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }
    }

    #[cfg(test)]
    mod loop_tests {
        use super::*;

        #[test]
        fn analyze_valid_loop() {
            let mut analyzer = Analyzer::new();
            let loop_statement = iteration!(block!());
            let program = Program::new(vec![loop_statement]);

            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }
    }

    #[cfg(test)]
    mod block_tests {
        use super::*;

        #[test]
        fn analyze_valid_block() {
            let mut analyzer = Analyzer::new();
            let block = block_statement!();
            let program = Program::new(vec![block]);

            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }
    }

    #[cfg(test)]
    mod function_definition_tests {
        use super::*;

        #[test]
        fn analyze_valid_function_definition() {
            let mut analyzer = Analyzer::new();
            let definition = function_definition!("calculate_total", vec![], block!());
            let program = Program::new(vec![definition]);

            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }
    }

    #[cfg(test)]
    mod pending_call_tests {
        use super::*;
        use crate::ast::program::Program;

        #[test]
        fn detects_shadowed_deferred_call_on_variable() {
            let mut analyzer = Analyzer::new();

            let call_statement = function_call!(expression_function_call!(
                expression_identifier!("calculate_total"),
                vec![],
                0
            ));

            let variable_declaration = variable_declaration!("calculate_total");
            let program = Program::new(vec![call_statement, variable_declaration]);

            let result = analyzer.analyze(&program);
            assert_eq!(
                result,
                Err(SemanticError::NotAFunction("calculate_total".to_string()))
            );
        }

        #[test]
        fn successfully_resolves_valid_pending_call() {
            let mut analyzer = Analyzer::new();

            let call_statement = function_call!(expression_function_call!(
                expression_identifier!("calculate_total"),
                vec![],
                0
            ));

            let function_definition = function_definition!("calculate_total", vec![], block!());

            let program = Program::new(vec![call_statement, function_definition]);

            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }

        #[test]
        fn detects_arity_mismatch_on_deferred_call() {
            let mut analyzer = Analyzer::new();

            let call_statement = function_call!(expression_function_call!(
                expression_identifier!("calculate_total"),
                vec![expression_i32!(10)],
                0
            ));

            let function_definition = function_definition!("calculate_total", vec![], block!());

            let program = Program::new(vec![call_statement, function_definition]);

            let result = analyzer.analyze(&program);
            assert_eq!(
                result,
                Err(SemanticError::ArgumentCountMismatch(
                    "calculate_total".to_string(),
                    0,
                    1
                ))
            );
        }

        #[test]
        fn detects_undefined_deferred_call() {
            let mut analyzer = Analyzer::new();

            let call_statement = function_call!(expression_function_call!(
                expression_identifier!("calculate_total"),
                vec![],
                0
            ));

            let program = Program::new(vec![call_statement]);

            let result = analyzer.analyze(&program);
            assert_eq!(
                result,
                Err(SemanticError::UndefinedVariable(
                    "calculate_total".to_string()
                ))
            );
        }
    }

    #[cfg(test)]
    mod break_tests {
        use super::*;

        #[test]
        fn analyze_valid_break_inside_loop() {
            let mut analyzer = Analyzer::new();
            let break_statement = break_statement!();
            let loop_statement = iteration!(block!(break_statement));
            let program = Program::new(vec![loop_statement]);

            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }
    }

    #[cfg(test)]
    mod return_tests {
        use super::*;

        #[test]
        fn analyze_valid_return_inside_function() {
            let mut analyzer = Analyzer::new();
            let return_statement = return_statement!();
            let definition =
                function_definition!("calculate_total", vec![], block!(return_statement));
            let program = Program::new(vec![definition]);

            let result = analyzer.analyze(&program);
            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod type_inference {
    use super::*;
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::statement::{
        Assignment, Block, FunctionDefinition, FunctionParameter, NodeId, Statement,
        VariableDeclaration,
    };
    use crate::semantic::inference::Type;

    #[test]
    fn analyze_type_mismatch_in_assignment_fails() {
        let mut analyzer = Analyzer::new();
        let declaration = Statement::VariableDeclaration(
            VariableDeclaration::new("age".to_string(), Some("i32".to_string()), None),
            NodeId(1),
        );
        let assignment = Statement::Assignment(
            Assignment::new(
                "age".to_string(),
                Expression::new(ExpressionKind::String("hello".to_string()), 1),
            ),
            NodeId(4),
        );
        let program = Program::new(vec![declaration, assignment]);

        let result = analyzer.analyze(&program);
        assert_eq!(
            result,
            Err(SemanticError::TypeMismatch(Type::Int32, Type::String))
        );
    }

    #[test]
    fn analyze_valid_type_inference_flow_succeeds() {
        let mut analyzer = Analyzer::new();
        let first_value_declaration = Statement::VariableDeclaration(
            VariableDeclaration::new("first_value".to_string(), None, None),
            NodeId(1),
        );
        let copied_value_declaration = Statement::VariableDeclaration(
            VariableDeclaration::new(
                "copied_value".to_string(),
                None,
                Some(Expression::new(
                    ExpressionKind::Identifier("first_value".to_string(), NodeId(2)),
                    1,
                )),
            ),
            NodeId(3),
        );
        let first_value_assignment = Statement::Assignment(
            Assignment::new(
                "first_value".to_string(),
                Expression::new(ExpressionKind::I32(10), 1),
            ),
            NodeId(6),
        );
        let program = Program::new(vec![
            first_value_declaration,
            copied_value_declaration,
            first_value_assignment,
        ]);

        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn analyze_function_call_with_ill_typed_argument_fails() {
        let mut analyzer = Analyzer::new();
        let parameter = FunctionParameter::new("age".to_string(), Some("i32".to_string()));
        let definition = FunctionDefinition::new(
            "print_age".to_string(),
            vec![parameter],
            None,
            Block::new(vec![]),
        );
        let call_statement = Statement::FunctionCall(
            Expression::new(
                ExpressionKind::FunctionCall(
                    Box::new(ExpressionKind::Identifier(
                        "print_age".to_string(),
                        NodeId(1),
                    )),
                    vec![ExpressionKind::String("twenty".to_string())],
                    NodeId(3),
                ),
                1,
            ),
            NodeId(4),
        );
        let program = Program::new(vec![
            Statement::FunctionDefinition(definition, NodeId(5)),
            call_statement,
        ]);

        let result = analyzer.analyze(&program);
        assert_eq!(
            result,
            Err(SemanticError::TypeMismatch(Type::String, Type::Int32))
        );
    }
}
