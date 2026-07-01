use crate::ast::statement::{Block, FunctionDefinition, FunctionParameter, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::block::BlockParser;
use crate::parser::error::ParseError;
use crate::parser::stream::ParserStream;

/// A sub-parser responsible for parsing function definition statements.
///
/// Optional return type and parameter type annotations are supported.
/// See [grammar.ebnf](../../docs/grammar.ebnf) for the full language grammar.
pub(crate) struct FnParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> FnParser<'src, 'stream, I> {
    /// Creates a new `FnParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses a complete function definition from the token stream.
    ///
    /// Returns a [`Statement::FunctionDefinition`] containing the function's name,
    /// parameters (with optional type annotations), optional return type, and body.
    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::Fn)?;
        let name = self.stream.expect(TokenType::Identifier)?.owned_value();
        self.stream.expect(TokenType::LeftParentheses)?;

        let parameters = self.parse_parameters()?;
        self.stream.expect(TokenType::RightParentheses)?;
        let return_type = self.maybe_parse_return_type()?;
        let body = self.parse_body()?;

        Ok(Statement::function_definition(FunctionDefinition::new(
            name,
            parameters,
            return_type,
            body,
        )))
    }

    fn parse_parameters(&mut self) -> Result<Vec<FunctionParameter>, ParseError> {
        let mut parameters = Vec::new();
        while let Some(next) = self.stream.peek()? {
            if next.token_type == TokenType::RightParentheses {
                break;
            }
            self.parse_parameter(&mut parameters)?;

            if !self.stream.maybe_matches(TokenType::Comma) {
                if let Some(after) = self.stream.peek()? {
                    if after.token_type != TokenType::RightParentheses {
                        return Err(ParseError::UnexpectedTokenType(
                            TokenType::RightParentheses,
                            after.token_type,
                            after.line,
                        ));
                    }
                }
            } else if let Some(after) = self.stream.peek()? {
                if after.token_type == TokenType::RightParentheses {
                    return Err(ParseError::TrailingComma(after.line));
                }
            }
        }
        Ok(parameters)
    }

    fn parse_parameter(
        &mut self,
        parameters: &mut Vec<FunctionParameter>,
    ) -> Result<(), ParseError> {
        let parameter_name = self.stream.expect(TokenType::Identifier)?.owned_value();
        let data_type = self.maybe_parse_data_type()?;
        parameters.push(FunctionParameter::new(parameter_name, data_type));
        Ok(())
    }

    fn maybe_parse_data_type(&mut self) -> Result<Option<String>, ParseError> {
        let mut data_type = None;
        if self.stream.maybe_matches(TokenType::Colon) {
            let type_token = self.stream.expect(TokenType::Identifier)?;
            data_type = Some(type_token.owned_value());
        }
        Ok(data_type)
    }

    fn maybe_parse_return_type(&mut self) -> Result<Option<String>, ParseError> {
        let mut return_type = None;
        if self.stream.maybe_matches(TokenType::Colon) {
            let type_token = self.stream.expect(TokenType::Identifier)?;
            return_type = Some(type_token.owned_value());
        }
        Ok(return_type)
    }

    fn parse_body(&mut self) -> Result<Block, ParseError> {
        BlockParser::new(self.stream).parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_empty_function_without_types() {
        let lexer = Lexer::new("fn calculate() {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::function_definition(FunctionDefinition::new(
                "calculate".to_string(),
                vec![],
                None,
                block!()
            ))
        );
    }

    #[test]
    fn parse_function_with_typed_parameter_and_return_type() {
        let lexer = Lexer::new("fn compute_tax(income: i32): i32 { }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::function_definition(FunctionDefinition::new(
                "compute_tax".to_string(),
                vec![FunctionParameter::new(
                    "income".to_string(),
                    Some("i32".to_string())
                )],
                Some("i32".to_string()),
                block!()
            ))
        );
    }

    #[test]
    fn parse_function_with_untyped_parameter_and_no_return_type() {
        let lexer = Lexer::new("fn determine_grade(score) { }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::function_definition(FunctionDefinition::new(
                "determine_grade".to_string(),
                vec![FunctionParameter::new("score".to_string(), None)],
                None,
                block!()
            ))
        );
    }

    #[test]
    fn parse_function_with_multiple_assignments() {
        let lexer = Lexer::new(
            "fn assign() { height = 200; weight = 300; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        assert_eq!(
            statement,
            Statement::function_definition(FunctionDefinition::new(
                "assign".to_string(),
                vec![],
                None,
                block!(
                    assignment!("height", expression_i32!(200, line)),
                    assignment!("weight", expression_i32!(300, line)),
                )
            ))
        );
    }

    #[test]
    fn parse_function_with_variable_declaration_and_assignment() {
        let lexer = Lexer::new(
            "fn test_func() { var id = 100; id = 200; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        let expected = Statement::function_definition(FunctionDefinition::new(
            "test_func".to_string(),
            vec![],
            None,
            block!(
                variable_declaration!("id", value: expression_i32!(100, line)),
                assignment!("id", expression_i32!(200, line)),
            ),
        ));
        assert_eq!(statement, expected);
    }

    #[test]
    fn parse_function_with_conditional() {
        let lexer = Lexer::new(
            "fn test_func() { if discount_rate > 0 { final_price = regular_price - savings; } }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        let expected = Statement::function_definition(FunctionDefinition::new(
            "test_func".to_string(),
            vec![],
            None,
            block!(conditional!(
                expression_binary!(
                    expression_identifier!("discount_rate"),
                    GreaterThan,
                    expression_i32!(0),
                    line
                ),
                Block::new(vec![assignment!(
                    "final_price",
                    expression_binary!(
                        expression_identifier!("regular_price"),
                        Minus,
                        expression_identifier!("savings"),
                        line
                    )
                )])
            )),
        ));
        assert_eq!(statement, expected);
    }

    #[test]
    fn parse_function_with_assignment_binary_expression() {
        let lexer = Lexer::new(
            "fn test_func() { total_price = base_price + tax_rate * quantity; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        let expected = Statement::function_definition(FunctionDefinition::new(
            "test_func".to_string(),
            vec![],
            None,
            block!(assignment!(
                "total_price",
                expression_binary!(
                    expression_identifier!("base_price"),
                    Plus,
                    expression_binary!(
                        expression_identifier!("tax_rate"),
                        Multiply,
                        expression_identifier!("quantity")
                    ),
                    line
                )
            )),
        ));
        assert_eq!(statement, expected);
    }

    #[test]
    fn parse_function_with_assignment_grouped_expression() {
        let lexer = Lexer::new(
            "fn test_func() { adjusted_score = (base_points + bonus_points) * multiplier; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        let expected = Statement::function_definition(FunctionDefinition::new(
            "test_func".to_string(),
            vec![],
            None,
            block!(assignment!(
                "adjusted_score",
                expression_binary!(
                    expression_grouped!(expression_binary!(
                        expression_identifier!("base_points"),
                        Plus,
                        expression_identifier!("bonus_points")
                    )),
                    Multiply,
                    expression_identifier!("multiplier"),
                    line
                )
            )),
        ));
        assert_eq!(statement, expected);
    }

    #[test]
    fn parse_function_with_declaration_complex_expression() {
        let lexer = Lexer::new(
            "fn test_func() { var total_cost = fixed_cost + variable_unit_cost * quantity; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;
        let expected = Statement::function_definition(FunctionDefinition::new(
            "test_func".to_string(),
            vec![],
            None,
            block!(variable_declaration!(
                "total_cost",
                value: expression_binary!(
                    expression_identifier!("fixed_cost"),
                    Plus,
                    expression_binary!(
                        expression_identifier!("variable_unit_cost"),
                        Multiply,
                        expression_identifier!("quantity")
                    ),
                    line
                )
            )),
        ));
        assert_eq!(statement, expected);
    }

    #[test]
    fn parse_function_with_declaration_and_assignment_complex_expressions() {
        let lexer = Lexer::new(
            "fn test_func() { var net_salary = gross_salary - deductions; net_salary = net_salary + yearly_bonus; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        let expected = Statement::function_definition(FunctionDefinition::new(
            "test_func".to_string(),
            vec![],
            None,
            block!(
                variable_declaration!(
                    "net_salary",
                    value: expression_binary!(
                        expression_identifier!("gross_salary"),
                        Minus,
                        expression_identifier!("deductions"),
                        line
                    )
                ),
                assignment!(
                    "net_salary",
                    expression_binary!(
                        expression_identifier!("net_salary"),
                        Plus,
                        expression_identifier!("yearly_bonus"),
                        line
                    )
                ),
            ),
        ));
        assert_eq!(statement, expected);
    }

    #[test]
    fn parse_function_missing_comma_between_parameters() {
        let lexer = Lexer::new("fn verify_status(risk_level status) { }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::RightParentheses, TokenType::Identifier, 1)
        );
    }

    #[test]
    fn parse_function_missing_name() {
        let lexer = Lexer::new("fn () {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::Identifier, TokenType::LeftParentheses, 1)
        );
    }

    #[test]
    fn parse_function_missing_left_parenthesis() {
        let lexer = Lexer::new("fn calculate) {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(
                TokenType::LeftParentheses,
                TokenType::RightParentheses,
                1
            )
        );
    }

    #[test]
    fn parse_function_missing_right_parenthesis() {
        let lexer = Lexer::new("fn calculate(a: int {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::RightParentheses, TokenType::LeftBrace, 1)
        );
    }

    #[test]
    fn parse_function_missing_left_brace() {
        let lexer = Lexer::new("fn calculate() }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::LeftBrace, TokenType::RightBrace, 1)
        );
    }

    #[test]
    fn parse_function_missing_right_brace() {
        let lexer = Lexer::new("fn calculate() {", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_function_missing_parameter_colon() {
        let lexer = Lexer::new("fn calculate(a int) {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::RightParentheses, TokenType::Identifier, 1)
        );
    }

    #[test]
    fn parse_function_trailing_comma_single_parameter() {
        let lexer = Lexer::new("fn calculate(a,) {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::TrailingComma(1));
    }

    #[test]
    fn parse_function_trailing_comma_multiple_parameters() {
        let lexer = Lexer::new("fn calculate(a: i32, b: i32,) {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = FnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::TrailingComma(1));
    }
}
