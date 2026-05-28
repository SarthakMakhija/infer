use crate::ast::statement::Statement;
use crate::lexer::token::{Token, TokenType};
use crate::lexer::LexResult;
use crate::parser::assignment::AssignmentParser;
use crate::parser::conditional::ConditionalParser;
use crate::parser::declaration::VariableDeclarationParser;
use crate::parser::error::ParseError;
use crate::parser::iteration::LoopParser;
use crate::parser::stream::ParserStream;

pub(crate) struct StatementParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> StatementParser<'src, 'stream, I> {
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        if let Some(token_ref) = self.stream.peek()? {
            let token = token_ref.clone();
            let statement = self.statement_beginning_at(&token)?;
            return Ok(statement);
        }
        Err(ParseError::UnexpectedEof)
    }

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
            TokenType::Identifier => {
                if let Some(assignment) = self.maybe_assignment()? {
                    assignment
                } else {
                    unimplemented!()
                }
            }
            _ => unimplemented!(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Expression;
    use crate::ast::statement::{Assignment, VariableDeclaration};
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
    #[should_panic(expected = "not implemented")]
    fn parse_unsupported_statement() {
        let lexer = Lexer::new("123;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = StatementParser::new(&mut stream);

        let _ = parser.parse();
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
            Statement::iteration(crate::ast::statement::Iteration::new(vec![]))
        );
    }
}
