use crate::ast::statement::Statement;
use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::assignment::AssignmentParser;
use crate::parser::conditional::ConditionalParser;
use crate::parser::control_flow::BreakParser;
use crate::parser::declaration::VariableDeclarationParser;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::function::FnParser;
use crate::parser::iteration::LoopParser;
use crate::parser::stream::ParserStream;

/// A dispatcher that routes statement parsing to the appropriate specialised sub-parser.
///
/// `StatementParser` reads the leading token to determine which statement construct
/// follows, then delegates to one of: `VariableDeclarationParser`, `AssignmentParser`,
/// `ConditionalParser`, `LoopParser`, `BreakParser`, `FnParser`, or expression-call handling.
pub(crate) struct StatementParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> StatementParser<'src, 'stream, I> {
    /// Creates a new `StatementParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses the next single statement from the token stream.
    ///
    /// Returns `Err(ParseError::UnexpectedEof)` if the stream is already at EOF.
    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        if let Some(token_ref) = self.stream.peek()? {
            let token = token_ref.clone();
            let statement = self.statement_beginning_at(&token)?;
            return Ok(statement);
        }
        Err(ParseError::UnexpectedEof)
    }

    /// Parses statements from the stream until the next token matches `token_type`.
    ///
    /// This is used to parse block bodies (e.g., the contents of an `if` or `loop`)
    /// where a closing `}` is the sentinel. The sentinel token is **not** consumed.
    pub(crate) fn parse_statements_till(
        &mut self,
        token_type: TokenType,
    ) -> Result<Vec<Statement>, ParseError> {
        let mut body = Vec::new();
        while let Some(next_token) = self.stream.peek()? {
            if next_token.token_type == token_type {
                break;
            }
            let statement = self.parse()?;
            body.push(statement);
        }
        Ok(body)
    }

    fn statement_beginning_at(&mut self, token: &Token) -> Result<Statement, ParseError> {
        let statement = match token.token_type {
            TokenType::Var => VariableDeclarationParser::new(self.stream).parse()?,
            TokenType::If => ConditionalParser::new(self.stream).parse()?,
            TokenType::Loop => LoopParser::new(self.stream).parse()?,
            TokenType::Break => BreakParser::new(self.stream).parse()?,
            TokenType::Fn => FnParser::new(self.stream).parse()?,
            TokenType::Identifier => {
                if let Some(assignment) = self.maybe_assignment()? {
                    assignment
                } else if let Some(call) = self.maybe_function_call()? {
                    call
                } else {
                    return Err(ParseError::UnsupportedStatement(
                        token.token_type,
                        token.line,
                    ));
                }
            }
            _ => {
                return Err(ParseError::UnsupportedStatement(
                    token.token_type,
                    token.line,
                ))
            }
        };
        Ok(statement)
    }

    fn maybe_assignment(&mut self) -> Result<Option<Statement>, ParseError> {
        if let Some(next_token) = self.stream.peek_second()? {
            if next_token.token_type == TokenType::Equals {
                let statement = AssignmentParser::new(self.stream).parse()?;
                return Ok(Some(statement));
            }
        }
        Ok(None)
    }

    fn maybe_function_call(&mut self) -> Result<Option<Statement>, ParseError> {
        if let Some(next_token) = self.stream.peek_second()? {
            if next_token.token_type == TokenType::LeftParentheses {
                let expression = ExpressionParser::new(self.stream).parse()?;
                self.stream.expect(TokenType::Semicolon)?;
                return Ok(Some(Statement::function_call(expression)));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{BinaryOperator, Expression};
    use crate::ast::statement::{
        Assignment, Block, Break, FunctionDefinition, FunctionParameter, Loop, VariableDeclaration,
    };
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn parse_variable_declaration() {
        let lexer = Lexer::new("var id = 10;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::variable_declaration(VariableDeclaration::new(
                "id".to_string(),
                None,
                Some(Expression::I32(10))
            ))
        );
    }

    #[test]
    fn parse_assignment() {
        let lexer = Lexer::new("id = 20;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::assignment(Assignment::new("id".to_string(), Expression::I32(20)))
        );
    }

    #[test]
    fn parse_unexpected_eof() {
        let lexer = Lexer::new("", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let result = parser.parse();
        assert_eq!(result.err().unwrap(), ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_unsupported_statement() {
        let lexer = Lexer::new("123;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let result = parser.parse();
        assert_eq!(
            result.err().unwrap(),
            ParseError::UnsupportedStatement(TokenType::WholeNumber, 1)
        );
    }

    #[test]
    fn parse_unsupported_identifier_statement() {
        let lexer = Lexer::new("score;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let result = parser.parse();
        assert_eq!(
            result.err().unwrap(),
            ParseError::UnsupportedStatement(TokenType::Identifier, 1)
        );
    }

    #[test]
    fn parse_assignment_statement_missing_semicolon() {
        let lexer = Lexer::new("score = 100", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let result = parser.parse();
        assert_eq!(result.err().unwrap(), ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_multiple_statements_till_right_brace() {
        let lexer = Lexer::new("var score = 100; score = 200; }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statements = parser.parse_statements_till(TokenType::RightBrace).unwrap();
        assert_eq!(
            statements,
            vec![
                Statement::variable_declaration(VariableDeclaration::new(
                    "score".to_string(),
                    None,
                    Some(Expression::I32(100))
                )),
                Statement::assignment(Assignment::new("score".to_string(), Expression::I32(200)))
            ]
        );
    }

    #[test]
    fn parse_statements_till_right_brace_for_empty_block() {
        let lexer = Lexer::new("}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statements = parser.parse_statements_till(TokenType::RightBrace).unwrap();
        assert_eq!(statements, vec![]);
    }

    #[test]
    fn parse_loop_statement() {
        let lexer = Lexer::new("loop {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::iteration(Loop::new(Block::new(vec![])))
        );
    }

    #[test]
    fn parse_break_statement() {
        let lexer = Lexer::new("break;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(statement, Statement::control_flow(Break::new()));
    }

    #[test]
    fn parse_loop_with_break_statement() {
        let lexer = Lexer::new("loop { break; }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::iteration(Loop::new(Block::new(vec![Statement::control_flow(
                Break::new()
            )])))
        );
    }

    #[test]
    fn parse_function_definition_statement() {
        let lexer = Lexer::new(
            "fn adjust_risk(score: i32): i32 { var risk_level = score; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::function_definition(FunctionDefinition::new(
                "adjust_risk".to_string(),
                vec![FunctionParameter::new(
                    "score".to_string(),
                    Some("i32".to_string())
                )],
                Some("i32".to_string()),
                Block::new(vec![Statement::variable_declaration(
                    VariableDeclaration::new(
                        "risk_level".to_string(),
                        None,
                        Some(Expression::Identifier("score".to_string()))
                    )
                )])
            ))
        );
    }

    #[test]
    fn parse_standalone_function_call_statement() {
        let lexer = Lexer::new("adjust_risk(45);", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::FunctionCall(Expression::FunctionCall(
                Box::new(Expression::Identifier("adjust_risk".to_string())),
                vec![Expression::I32(45)]
            ))
        );
    }

    #[test]
    fn parse_standalone_function_call_with_expressions() {
        let lexer = Lexer::new("adjust_risk(base_score + 10);", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::FunctionCall(Expression::FunctionCall(
                Box::new(Expression::Identifier("adjust_risk".to_string())),
                vec![Expression::Binary(
                    Box::new(Expression::Identifier("base_score".to_string())),
                    BinaryOperator::Plus,
                    Box::new(Expression::I32(10))
                )]
            ))
        );
    }

    #[test]
    fn parse_standalone_function_call_unexpected_token_instead_of_semicolon() {
        let lexer = Lexer::new("adjust_risk(45) loop", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let result = parser.parse();
        assert_eq!(
            result.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::Semicolon, TokenType::Loop, 1)
        );
    }

    #[test]
    fn parse_standalone_function_call_missing_semicolon() {
        let lexer = Lexer::new("adjust_risk(45)", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let result = parser.parse();
        assert_eq!(result.err().unwrap(), ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_statement_lex_error_in_lookahead() {
        let lexer = Lexer::new("attempts ?", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert!(matches!(
            error,
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?', 1))
        ));
    }

    #[test]
    fn parse_statements_till_lex_error() {
        let lexer = Lexer::new("var id = 10; ?", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let error = parser
            .parse_statements_till(TokenType::RightBrace)
            .unwrap_err();
        assert!(matches!(
            error,
            ParseError::LexError(crate::lexer::error::LexError::UnrecognizedChar('?', 1))
        ));
    }

    #[test]
    fn parse_statement_single_identifier_eof() {
        let lexer = Lexer::new("attempts", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnsupportedStatement(TokenType::Identifier, 1)
        );
    }
}
