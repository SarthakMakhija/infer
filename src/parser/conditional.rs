use crate::ast::expr::Expression;
use crate::ast::statement::{Block, If, Statement};
use crate::lexer::token::TokenType;
use crate::lexer::LexResult;
use crate::parser::block::BlockParser;
use crate::parser::error::ParseError;
use crate::parser::expr::ExpressionParser;
use crate::parser::stream::ParserStream;

/// A sub-parser responsible for parsing `if` / `else if` / `else` conditional statements.
///
/// Recursively handles `else if` chains by delegating back to `parse`.
/// See [grammar.ebnf](../../docs/grammar.ebnf) for the full language grammar.
pub(crate) struct ConditionalParser<'src, 'stream, I: Iterator<Item = LexResult<'src>>> {
    stream: &'stream mut ParserStream<'src, I>,
}

impl<'src, 'stream, I: Iterator<Item = LexResult<'src>>> ConditionalParser<'src, 'stream, I> {
    /// Creates a new `ConditionalParser` sharing the parser stream borrow.
    pub(crate) fn new(stream: &'stream mut ParserStream<'src, I>) -> Self {
        Self { stream }
    }

    /// Parses a complete `if` statement (with optional `else if` / `else` branches).
    pub(crate) fn parse(&mut self) -> Result<Statement, ParseError> {
        let (condition, body) = self.parse_if()?;
        let else_body = self.maybe_parse_else()?;

        Ok(Statement::conditional(If::new(condition, body, else_body)))
    }

    fn parse_if(&mut self) -> Result<(Expression, Block), ParseError> {
        let if_token = self.stream.expect(TokenType::If)?;
        let condition = ExpressionParser::new(self.stream).parse()?;
        let body = self.parse_body()?;
        Ok((Expression::new(condition, if_token.line), body))
    }

    fn maybe_parse_else(&mut self) -> Result<Option<Block>, ParseError> {
        if self.stream.maybe_matches(TokenType::Else) {
            if let Some(next) = self.stream.peek()? {
                if next.token_type == TokenType::If {
                    let nested_if = self.parse()?;
                    return Ok(Some(Block::new(vec![nested_if])));
                }
            }
            return Ok(Some(self.parse_else()?));
        }
        Ok(None)
    }

    fn parse_else(&mut self) -> Result<Block, ParseError> {
        self.parse_body()
    }

    fn parse_body(&mut self) -> Result<Block, ParseError> {
        BlockParser::new(self.stream).parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::statement::{Block, Statement};
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_conditional_with_single_statement() {
        let line = 1;
        let lexer = Lexer::new(
            "if score >= minimum_score { is_eligible = true; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::conditional(If::new(
                expression_binary!(
                    expression_identifier!("score"),
                    GreaterThanEquals,
                    expression_identifier!("minimum_score"),
                    line
                ),
                Block::new(vec![assignment!(
                    "is_eligible",
                    expression_boolean!(true, 1)
                )]),
                None
            ))
        );
    }

    #[test]
    fn parse_conditional_with_multiple_statements() {
        let lexer = Lexer::new(
            "if total_price > budget { status = \"over_budget\"; charge = base_price + excess_fee; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        assert_eq!(
            statement,
            Statement::conditional(If::new(
                expression_binary!(
                    expression_identifier!("total_price"),
                    GreaterThan,
                    expression_identifier!("budget"),
                    line
                ),
                Block::new(vec![
                    assignment!("status", expression_string!("over_budget", line)),
                    assignment!(
                        "charge",
                        expression_binary!(
                            expression_identifier!("base_price"),
                            Plus,
                            expression_identifier!("excess_fee"),
                            line
                        )
                    ),
                ]),
                None
            ))
        );
    }

    #[test]
    fn parse_conditional_with_empty_body() {
        let line = 1;
        let lexer = Lexer::new("if debug_mode_enabled == true {}", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        assert_eq!(
            statement,
            Statement::conditional(If::new(
                expression_binary!(
                    expression_identifier!("debug_mode_enabled"),
                    EqualsEquals,
                    expression_boolean!(true),
                    line
                ),
                Block::new(vec![]),
                None
            ))
        );
    }

    #[test]
    fn parse_conditional_with_else() {
        let lexer = Lexer::new(
            "if total_price > budget { status = \"over_budget\"; } else { status = \"within_budget\"; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        assert_eq!(
            statement,
            Statement::conditional(If::new(
                expression_binary!(
                    expression_identifier!("total_price"),
                    GreaterThan,
                    expression_identifier!("budget"),
                    line
                ),
                Block::new(vec![assignment!(
                    "status",
                    expression_string!("over_budget", line)
                )]),
                Some(Block::new(vec![assignment!(
                    "status",
                    expression_string!("within_budget", line)
                )]))
            ))
        );
    }

    #[test]
    fn parse_conditional_missing_left_brace() {
        let lexer = Lexer::new(
            "if score >= minimum_score is_eligible = true; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let result = parser.parse();
        assert!(matches!(
            result.err().unwrap(),
            ParseError::UnexpectedTokenType(TokenType::LeftBrace, TokenType::Identifier, 1)
        ));
    }
}

#[cfg(test)]
mod else_if_tests {
    use super::*;
    use crate::ast::statement::{Block, Statement};
    use crate::lexer::keywords::Keywords;
    use crate::lexer::Lexer;
    use crate::parser::stream::ParserStream;

    #[test]
    fn parse_conditional_with_else_if() {
        let lexer = Lexer::new(
            "if score >= 90 { grade = \"A\"; } else if score >= 80 { grade = \"B\"; } else { grade = \"C\"; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        assert_eq!(
            statement,
            Statement::conditional(If::new(
                expression_binary!(
                    expression_identifier!("score"),
                    GreaterThanEquals,
                    expression_i32!(90),
                    line
                ),
                Block::new(vec![assignment!("grade", expression_string!("A", line))]),
                Some(Block::new(vec![Statement::conditional(If::new(
                    expression_binary!(
                        expression_identifier!("score"),
                        GreaterThanEquals,
                        expression_i32!(80),
                        line
                    ),
                    Block::new(vec![assignment!("grade", expression_string!("B", line))]),
                    Some(Block::new(vec![assignment!(
                        "grade",
                        expression_string!("C", line)
                    )]))
                ))]))
            ))
        );
    }

    #[test]
    fn parse_conditional_with_else_if_without_else() {
        let lexer = Lexer::new(
            "if risk_level > 8 { status = \"high\"; } else if risk_level > 4 { status = \"medium\"; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        assert_eq!(
            statement,
            Statement::conditional(If::new(
                expression_binary!(
                    expression_identifier!("risk_level"),
                    GreaterThan,
                    expression_i32!(8),
                    line
                ),
                Block::new(vec![assignment!(
                    "status",
                    expression_string!("high", line)
                )]),
                Some(Block::new(vec![Statement::conditional(If::new(
                    expression_binary!(
                        expression_identifier!("risk_level"),
                        GreaterThan,
                        expression_i32!(4),
                        line
                    ),
                    Block::new(vec![assignment!(
                        "status",
                        expression_string!("medium", line)
                    )]),
                    None
                ))]))
            ))
        );
    }

    #[test]
    fn parse_conditional_with_multiple_else_ifs() {
        let lexer = Lexer::new(
            "if income > 100000 { rate = 30; } else if income > 50000 { rate = 20; } else if income > 20000 { rate = 10; } else { rate = 5; }",
            Keywords::new(),
        );
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let statement = parser.parse().unwrap();
        let line = 1;

        assert_eq!(
            statement,
            Statement::conditional(If::new(
                expression_binary!(
                    expression_identifier!("income"),
                    GreaterThan,
                    expression_i32!(100000),
                    line
                ),
                Block::new(vec![assignment!("rate", expression_i32!(30, 1))]),
                Some(Block::new(vec![Statement::conditional(If::new(
                    expression_binary!(
                        expression_identifier!("income"),
                        GreaterThan,
                        expression_i32!(50000),
                        line
                    ),
                    Block::new(vec![assignment!("rate", expression_i32!(20, line))]),
                    Some(Block::new(vec![Statement::conditional(If::new(
                        expression_binary!(
                            expression_identifier!("income"),
                            GreaterThan,
                            expression_i32!(20000),
                            line
                        ),
                        Block::new(vec![assignment!("rate", expression_i32!(10, line))]),
                        Some(Block::new(vec![assignment!(
                            "rate",
                            expression_i32!(5, line)
                        )]))
                    ))]))
                ))]))
            ))
        );
    }

    #[test]
    fn parse_conditional_missing_right_brace() {
        let lexer = Lexer::new("if score >= 90 { grade = \"A\";", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_conditional_else_missing_left_brace() {
        let lexer = Lexer::new("if score >= 90 {} else grade = \"A\"; }", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(
            error,
            ParseError::UnexpectedTokenType(TokenType::LeftBrace, TokenType::Identifier, 1)
        );
    }

    #[test]
    fn parse_conditional_else_missing_right_brace() {
        let lexer = Lexer::new("if score >= 90 {} else { grade = \"A\";", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }

    #[test]
    fn parse_conditional_else_eof() {
        let lexer = Lexer::new("if score >= 90 {} else", Keywords::new());
        let mut stream = ParserStream::new(lexer);
        let mut parser = ConditionalParser::new(&mut stream);

        let error = parser.parse().unwrap_err();
        assert_eq!(error, ParseError::UnexpectedEof);
    }
}
