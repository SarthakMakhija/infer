use crate::ast::statement::{FunctionDefinition, FunctionParameter, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::statement::StatementParser;
use crate::parser::stream::ParserStream;

pub(crate) struct FnParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> FnParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::Fn)?;
        let name = self.stream.expect(TokenType::Identifier)?.owned_value();
        self.stream.expect(TokenType::LeftParentheses)?;

        let parameters = self.parse_parameters()?;
        self.stream.expect(TokenType::RightParentheses)?;
        let return_type = self.maybe_parse_return_type()?;

        self.stream.expect(TokenType::LeftBrace)?;
        let body = self.parse_body()?;
        self.stream.expect(TokenType::RightBrace)?;

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

    fn parse_body(&mut self) -> Result<Vec<Statement>, ParseError> {
        StatementParser::new(self.stream).parse_statements_till(TokenType::RightBrace)
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
                vec![]
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
                vec![]
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
                vec![]
            ))
        );
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
}
