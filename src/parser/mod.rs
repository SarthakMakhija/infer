use crate::ast::program::{Program, ProgramBuilder};
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::statement::StatementParser;
use crate::parser::stream::ParserStream;

pub(crate) mod assignment;
pub(crate) mod conditional;
pub(crate) mod declaration;
pub(crate) mod error;
pub(crate) mod expr;
pub(crate) mod statement;
pub(crate) mod stream;

pub(crate) struct Parser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> Parser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Program, ParseError> {
        let mut builder = ProgramBuilder::new();

        while self.stream.peek()?.is_some() {
            let statement = StatementParser::new(self.stream).parse()?;
            builder = builder.add(statement);
        }
        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{BinaryOperator, Expression};
    use crate::ast::statement::VariableDeclaration;
    use crate::ast::statement::{Assignment, Conditional, Statement};
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn parse_empty_program() {
        let lexer = Lexer::new("", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        assert_eq!(program, ProgramBuilder::new().build());
    }

    #[test]
    fn parse_single_variable_declaration() {
        let lexer = Lexer::new("var greeting = \"hello\";", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "greeting".to_string(),
                None,
                Some(Expression::String("hello".to_string())),
            )))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_multiple_variable_declarations() {
        let lexer = Lexer::new("var x = 100; var flag = true;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "x".to_string(),
                None,
                Some(Expression::I32(100)),
            )))
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "flag".to_string(),
                None,
                Some(Expression::Boolean(true)),
            )))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_lex_error() {
        let lexer = Lexer::new("var x = 100; ?", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let res = parser.parse();
        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?', 1))
        ));
    }

    #[test]
    fn parse_single_assignment() {
        let lexer = Lexer::new("id = 200;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::assignment(
                crate::ast::statement::Assignment::new("id".to_string(), Expression::I32(200)),
            ))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_multiple_assignments() {
        let lexer = Lexer::new("height = 200; weight = 300;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::assignment(
                crate::ast::statement::Assignment::new("height".to_string(), Expression::I32(200)),
            ))
            .add(Statement::assignment(
                crate::ast::statement::Assignment::new("weight".to_string(), Expression::I32(300)),
            ))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_variable_declaration_and_assignment() {
        let lexer = Lexer::new("var id = 100; id = 200;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "id".to_string(),
                None,
                Some(Expression::I32(100)),
            )))
            .add(Statement::assignment(
                crate::ast::statement::Assignment::new("id".to_string(), Expression::I32(200)),
            ))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_program_with_conditional() {
        let lexer = Lexer::new(
            "if discount_rate > 0 { final_price = regular_price - savings; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::conditional(Conditional::new(
                Expression::Binary(
                    Box::new(Expression::Identifier("discount_rate".to_string())),
                    BinaryOperator::GreaterThan,
                    Box::new(Expression::I32(0)),
                ),
                vec![Statement::assignment(Assignment::new(
                    "final_price".to_string(),
                    Expression::Binary(
                        Box::new(Expression::Identifier("regular_price".to_string())),
                        BinaryOperator::Minus,
                        Box::new(Expression::Identifier("savings".to_string())),
                    ),
                ))],
            )))
            .build();
        assert_eq!(program, expected);
    }
}

#[cfg(test)]
mod assignment_expression_tests {
    use super::*;
    use crate::ast::expr::{BinaryOperator, Expression};
    use crate::ast::statement::Statement;
    use crate::ast::statement::{Assignment, VariableDeclaration};
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn parse_assignment_with_binary_expression() {
        let lexer = Lexer::new(
            "total_price = base_price + tax_rate * quantity;",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::assignment(Assignment::new(
                "total_price".to_string(),
                Expression::Binary(
                    Box::new(Expression::Identifier("base_price".to_string())),
                    BinaryOperator::Plus,
                    Box::new(Expression::Binary(
                        Box::new(Expression::Identifier("tax_rate".to_string())),
                        BinaryOperator::Multiply,
                        Box::new(Expression::Identifier("quantity".to_string())),
                    )),
                ),
            )))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_assignment_with_grouped_expression() {
        let lexer = Lexer::new(
            "adjusted_score = (base_points + bonus_points) * multiplier;",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::assignment(Assignment::new(
                "adjusted_score".to_string(),
                Expression::Binary(
                    Box::new(Expression::Grouped(Box::new(Expression::Binary(
                        Box::new(Expression::Identifier("base_points".to_string())),
                        BinaryOperator::Plus,
                        Box::new(Expression::Identifier("bonus_points".to_string())),
                    )))),
                    BinaryOperator::Multiply,
                    Box::new(Expression::Identifier("multiplier".to_string())),
                ),
            )))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_declaration_with_complex_expression() {
        let lexer = Lexer::new(
            "var total_cost = fixed_cost + variable_unit_cost * quantity;",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "total_cost".to_string(),
                None,
                Some(Expression::Binary(
                    Box::new(Expression::Identifier("fixed_cost".to_string())),
                    BinaryOperator::Plus,
                    Box::new(Expression::Binary(
                        Box::new(Expression::Identifier("variable_unit_cost".to_string())),
                        BinaryOperator::Multiply,
                        Box::new(Expression::Identifier("quantity".to_string())),
                    )),
                )),
            )))
            .build();
        assert_eq!(program, expected);
    }

    #[test]
    fn parse_declaration_and_assignment_with_complex_expressions() {
        let lexer = Lexer::new(
            "var net_salary = gross_salary - deductions; net_salary = net_salary + yearly_bonus;",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = Parser::new(&mut stream);

        let program = parser.parse().unwrap();
        let expected = ProgramBuilder::new()
            .add(Statement::variable_declaration(VariableDeclaration::new(
                "net_salary".to_string(),
                None,
                Some(Expression::Binary(
                    Box::new(Expression::Identifier("gross_salary".to_string())),
                    BinaryOperator::Minus,
                    Box::new(Expression::Identifier("deductions".to_string())),
                )),
            )))
            .add(Statement::assignment(Assignment::new(
                "net_salary".to_string(),
                Expression::Binary(
                    Box::new(Expression::Identifier("net_salary".to_string())),
                    BinaryOperator::Plus,
                    Box::new(Expression::Identifier("yearly_bonus".to_string())),
                ),
            )))
            .build();
        assert_eq!(program, expected);
    }
}
