use crate::ast::program::Program;
use crate::semantic::error::SemanticError;
use crate::semantic::symbol_resolution::SymbolResolutionVisitor;

pub(crate) struct Analyzer {
    visitor: SymbolResolutionVisitor,
}

impl Analyzer {
    pub(crate) fn new() -> Self {
        Analyzer {
            visitor: SymbolResolutionVisitor::new(),
        }
    }

    pub(crate) fn analyze(&mut self, program: &Program) -> Result<(), SemanticError> {
        self.visitor.visit_statements(program.statements())?;
        self.visitor.resolve_pending_calls()?;
        Ok(())
    }
}

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
    use crate::ast::expr::{Expression, ExpressionKind};
    use crate::ast::program::Program;
    use crate::ast::statement::Statement;

    #[test]
    fn detects_shadowed_deferred_call_on_variable() {
        let mut analyzer = Analyzer::new();

        let callee = ExpressionKind::identifier("calculate_total".to_string());
        let call_expression = ExpressionKind::function_call(callee, vec![]);
        let call_statement = Statement::function_call(Expression::new(call_expression, 0));

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

        let callee = ExpressionKind::identifier("calculate_total".to_string());
        let call_expression = ExpressionKind::function_call(callee, vec![]);
        let call_statement = Statement::function_call(Expression::new(call_expression, 0));

        let function_definition = function_definition!("calculate_total", vec![], block!());

        let program = Program::new(vec![call_statement, function_definition]);

        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn detects_arity_mismatch_on_deferred_call() {
        let mut analyzer = Analyzer::new();

        let callee = ExpressionKind::identifier("calculate_total".to_string());
        let call_expression = ExpressionKind::function_call(callee, vec![ExpressionKind::I32(10)]);
        let call_statement = Statement::function_call(Expression::new(call_expression, 0));

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

        let callee = ExpressionKind::identifier("calculate_total".to_string());
        let call_expression = ExpressionKind::function_call(callee, vec![]);
        let call_statement = Statement::function_call(Expression::new(call_expression, 0));

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
        let definition = function_definition!("calculate_total", vec![], block!(return_statement));
        let program = Program::new(vec![definition]);

        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }
}
