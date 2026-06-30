use crate::ast::statement::{Block, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::error::ParseError;
use crate::parser::statement::StatementParser;
use crate::parser::stream::ParserStream;

/// A parser that handles parsing block statements `{ ... }` recursively.
pub(crate) struct BlockParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> BlockParser<'src, 'stream, I> {
    /// Creates a new `BlockParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses a block statement, recursively parsing nested blocks if encountered.
    ///
    /// Consumes the opening `{` and closing `}` and returns a parsed `Block`.
    pub(crate) fn parse(&mut self) -> Result<Block, ParseError> {
        self.stream.expect(TokenType::LeftBrace)?;

        let mut body = Vec::new();
        while let Some(next_token) = self.stream.peek()? {
            if next_token.token_type == TokenType::LeftBrace {
                // If we see another LeftBrace, recurse and parse as a nested block statement
                let nested_block = self.parse()?;
                body.push(Statement::block(nested_block));
            } else if next_token.token_type == TokenType::RightBrace {
                break;
            } else {
                let statement = StatementParser::new(self.stream).parse()?;
                body.push(statement);
            }
        }

        self.stream.expect(TokenType::RightBrace)?;
        Ok(Block::new(body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::statement::{Assignment, VariableDeclaration};
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;

    #[test]
    fn parse_empty_block() {
        let lexer = Lexer::new("{}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = BlockParser::new(&mut stream);

        let block = parser.parse().unwrap();
        assert_eq!(block, Block::new(vec![]));
    }

    #[test]
    fn parse_simple_block() {
        let lexer = Lexer::new("{ var score = 10; score = 20; }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = BlockParser::new(&mut stream);

        let block = parser.parse().unwrap();
        assert_eq!(
            block,
            Block::new(vec![
                Statement::variable_declaration(VariableDeclaration::new(
                    "score".to_string(),
                    None,
                    Some(crate::ast::expr::ExpressionKind::I32(10))
                )),
                Statement::assignment(Assignment::new(
                    "score".to_string(),
                    crate::ast::expr::ExpressionKind::I32(20)
                ))
            ])
        );
    }

    #[test]
    fn parse_nested_blocks() {
        let lexer = Lexer::new(
            "{ var score = 10; { var risk_level = 20; } var threshold = 30; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = BlockParser::new(&mut stream);

        let block = parser.parse().unwrap();
        assert_eq!(
            block,
            Block::new(vec![
                Statement::variable_declaration(VariableDeclaration::new(
                    "score".to_string(),
                    None,
                    Some(crate::ast::expr::ExpressionKind::I32(10))
                )),
                Statement::block(Block::new(vec![Statement::variable_declaration(
                    VariableDeclaration::new(
                        "risk_level".to_string(),
                        None,
                        Some(crate::ast::expr::ExpressionKind::I32(20))
                    )
                )])),
                Statement::variable_declaration(VariableDeclaration::new(
                    "threshold".to_string(),
                    None,
                    Some(crate::ast::expr::ExpressionKind::I32(30))
                ))
            ])
        );
    }

    #[test]
    fn parse_consecutive_nested_blocks() {
        let lexer = Lexer::new(
            "{ { var score = 10; } { var risk_level = 20; } }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = BlockParser::new(&mut stream);

        let block = parser.parse().unwrap();
        assert_eq!(
            block,
            Block::new(vec![
                Statement::block(Block::new(vec![Statement::variable_declaration(
                    VariableDeclaration::new(
                        "score".to_string(),
                        None,
                        Some(crate::ast::expr::ExpressionKind::I32(10))
                    )
                )])),
                Statement::block(Block::new(vec![Statement::variable_declaration(
                    VariableDeclaration::new(
                        "risk_level".to_string(),
                        None,
                        Some(crate::ast::expr::ExpressionKind::I32(20))
                    )
                )]))
            ])
        );
    }

    #[test]
    fn parse_block_missing_closing_brace() {
        let lexer = Lexer::new("{ var score = 10;", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = BlockParser::new(&mut stream);

        let result = parser.parse();
        assert_eq!(result.err().unwrap(), ParseError::UnexpectedEof);
    }
}
