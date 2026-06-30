use crate::ast::statement::{Break, Return, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::stream::ParserStream;

/// A sub-parser responsible for parsing `break` statements inside loops.
///
/// See [grammar.ebnf](../../docs/grammar.ebnf) for the full language grammar.
pub(crate) struct BreakParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> BreakParser<'src, 'stream, I> {
    /// Creates a new `BreakParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses a `break;` statement.
    ///
    /// Expects the `break` keyword followed by a `;` terminator.
    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::Break)?;
        self.stream.expect(TokenType::Semicolon)?;
        Ok(Statement::control_flow(Break::new()))
    }
}

/// A sub-parser responsible for parsing `return` control flow statements.
///
/// Supports both returning an expression (`return expr;`) and returning nothing (`return;`).
pub(crate) struct ReturnParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> ReturnParser<'src, 'stream, I> {
    /// Creates a new `ReturnParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses a `return [Expression];` statement.
    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        self.stream.expect(TokenType::Return)?;

        let expression = if self.stream.maybe_matches(TokenType::Semicolon) {
            None
        } else {
            let expr = ExpressionParser::new(self.stream).parse()?;
            self.stream.expect(TokenType::Semicolon)?;
            Some(expr)
        };

        Ok(Statement::return_(Return::new(expression)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::ExpressionKind;
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_break_statement() {
        let lexer = Lexer::new("break;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = BreakParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(statement, Statement::control_flow(Break::new()));
    }

    #[test]
    fn parse_break_statement_missing_semicolon() {
        let lexer = Lexer::new("break", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = BreakParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_break_statement_unexpected_token_instead_of_semicolon() {
        let lexer = Lexer::new("break var income = 50000;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = BreakParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::Semicolon, TokenType::Var, 1)
        );
    }

    #[test]
    fn parse_empty_return_statement() {
        let lexer = Lexer::new("return;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ReturnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(statement, Statement::return_(Return::new(None)));
    }

    #[test]
    fn parse_return_with_expression() {
        let lexer = Lexer::new("return 42;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ReturnParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::return_(Return::new(Some(ExpressionKind::I32(42))))
        );
    }

    #[test]
    fn parse_return_statement_missing_semicolon() {
        let lexer = Lexer::new("return 42", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ReturnParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }
}
