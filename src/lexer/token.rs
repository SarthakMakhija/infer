use crate::lexer::error::LexError;
use std::ops::Range;

/// The catalog of different syntactic categories (token types) supported by the language.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum TokenType {
    /// The `=` assignment operator.
    Equals,
    /// The `;` statement terminator.
    Semicolon,
    /// The `:` type annotation separator.
    Colon,
    /// The `,` comma parameter/argument separator.
    Comma,
    /// A user-defined identifier (e.g. variable names or type names).
    Identifier,
    /// The `var` structural keyword.
    Var,
    /// A numeric integer literal.
    WholeNumber,
    /// A double-quoted string literal.
    StringLiteral,
    /// The boolean literal, true/false.
    BooleanLiteral(bool),
    /// The `+` operator.
    Plus,
    /// The `-` operator.
    Minus,
    /// The `*` operator.
    Star,
    /// The `/` operator.
    Slash,
    /// The `!` logical NOT operator.
    Bang,
    /// The `(` opening parentheses.
    LeftParentheses,
    /// The `)` closing parentheses.
    RightParentheses,
    /// The `{` opening curly brace.
    LeftBrace,
    /// The `}` closing curly brace.
    RightBrace,
    /// The `>` comparison operator.
    GreaterThan,
    /// The `<` comparison operator.
    LessThan,
    /// The `>=` comparison operator.
    GreaterThanEquals,
    /// The `<=` comparison operator.
    LessThanEquals,
    /// The `==` comparison operator.
    EqualsEquals,
    /// The `!=` comparison operator.
    BangEquals,
    /// The `if` conditional keyword.
    If,
    /// The `else` conditional keyword.
    Else,
    /// The `loop` iteration keyword.
    Loop,
    /// The `break` control flow keyword.
    Break,
    /// The `fn` function keyword.
    Fn,
    /// The `and` keyword.
    And,
    /// The `or` keyword.
    Or,
    /// The `return` control flow keyword.
    Return,
}

impl TokenType {
    /// Maps a string slice lexeme to its corresponding reserved keyword `TokenType`.
    /// Returns `LexError::UnsupportedKeyword` if the slice is not a recognized keyword.
    pub(crate) fn keyword_type(token: &str, line: usize) -> Result<Self, LexError> {
        match token {
            "var" => Ok(TokenType::Var),
            "true" => Ok(TokenType::BooleanLiteral(true)),
            "false" => Ok(TokenType::BooleanLiteral(false)),
            "if" => Ok(TokenType::If),
            "else" => Ok(TokenType::Else),
            "loop" => Ok(TokenType::Loop),
            "break" => Ok(TokenType::Break),
            "fn" => Ok(TokenType::Fn),
            "and" => Ok(TokenType::And),
            "or" => Ok(TokenType::Or),
            "return" => Ok(TokenType::Return),
            _ => Err(LexError::UnsupportedKeyword(token.to_string(), line)),
        }
    }
}

/// Represents a single lexical token scanned from the source code.
///
/// A `Token` keeps track of its type, its byte range in the original source,
/// the line number it was found on, and holds a reference to the source itself.
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Token<'src> {
    /// The syntactic category of this token.
    pub(crate) token_type: TokenType,
    /// The byte range span of this token in the source code.
    pub(crate) range: Range<usize>,
    /// The 1-based line number where this token is located.
    pub(crate) line: usize,
    /// A reference to the original source code slice.
    pub(crate) source: &'src str,
}

impl<'src> Token<'src> {
    /// Creates a generic new `Token`.
    pub(crate) fn new(
        token_type: TokenType,
        range: Range<usize>,
        line: usize,
        source: &'src str,
    ) -> Self {
        Self {
            token_type,
            range,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `=` operator.
    pub(crate) fn equals(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Equals,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `;` terminator.
    pub(crate) fn semicolon(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Semicolon,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `:` separator.
    pub(crate) fn colon(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Colon,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `,` comma separator.
    pub(crate) fn comma(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Comma,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `+` operator.
    pub(crate) fn plus(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Plus,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `-` operator.
    pub(crate) fn minus(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Minus,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `*` operator.
    pub(crate) fn star(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Star,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `/` operator.
    pub(crate) fn slash(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Slash,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `!` operator.
    pub(crate) fn bang(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::Bang,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `(` character.
    pub(crate) fn left_parentheses(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::LeftParentheses,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `)` character.
    pub(crate) fn right_parentheses(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::RightParentheses,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `{` character.
    pub(crate) fn left_brace(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::LeftBrace,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `}` character.
    pub(crate) fn right_brace(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::RightBrace,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `>` character.
    pub(crate) fn greater_than(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::GreaterThan,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `<` character.
    pub(crate) fn less_than(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::LessThan,
            range: index..index + 1,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `>=` sequence.
    pub(crate) fn greater_than_equals(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::GreaterThanEquals,
            range: index..index + 2,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `<=` sequence.
    pub(crate) fn less_than_equals(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::LessThanEquals,
            range: index..index + 2,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `==` sequence.
    pub(crate) fn equals_equals(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::EqualsEquals,
            range: index..index + 2,
            line,
            source,
        }
    }

    /// Creates a new `Token` representing the `!=` sequence.
    pub(crate) fn bang_equals(source: &'src str, index: usize, line: usize) -> Self {
        Self {
            token_type: TokenType::BangEquals,
            range: index..index + 2,
            line,
            source,
        }
    }

    /// Returns the text slice of this token from the original source code.
    pub(crate) fn value(&self) -> &str {
        &self.source[self.range.start..self.range.end]
    }

    /// Returns the owned value of this token from the original source code.
    pub(crate) fn owned_value(&self) -> String {
        self.value().to_string()
    }

    /// Returns the string value of a string literal token, stripping the surrounding double quotes.
    pub(crate) fn string_value(&self) -> &str {
        let val = self.value();
        if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
            &val[1..val.len() - 1]
        } else {
            val
        }
    }
}

#[cfg(test)]
mod token_type_tests {
    use super::*;

    #[test]
    fn keyword_type_var() {
        assert_eq!(TokenType::keyword_type("var", 1).unwrap(), TokenType::Var);
    }

    #[test]
    fn keyword_type_true() {
        assert_eq!(
            TokenType::keyword_type("true", 1).unwrap(),
            TokenType::BooleanLiteral(true)
        );
    }

    #[test]
    fn keyword_type_false() {
        assert_eq!(
            TokenType::keyword_type("false", 1).unwrap(),
            TokenType::BooleanLiteral(false)
        );
    }

    #[test]
    fn keyword_type_if() {
        assert_eq!(TokenType::keyword_type("if", 1).unwrap(), TokenType::If);
    }

    #[test]
    fn keyword_type_else() {
        assert_eq!(TokenType::keyword_type("else", 1).unwrap(), TokenType::Else);
    }

    #[test]
    fn keyword_type_loop() {
        assert_eq!(TokenType::keyword_type("loop", 1).unwrap(), TokenType::Loop);
    }

    #[test]
    fn keyword_type_break() {
        assert_eq!(
            TokenType::keyword_type("break", 1).unwrap(),
            TokenType::Break
        );
    }

    #[test]
    fn keyword_type_fn() {
        assert_eq!(TokenType::keyword_type("fn", 1).unwrap(), TokenType::Fn);
    }

    #[test]
    fn keyword_type_and() {
        assert_eq!(TokenType::keyword_type("and", 1).unwrap(), TokenType::And);
    }

    #[test]
    fn keyword_type_or() {
        assert_eq!(TokenType::keyword_type("or", 1).unwrap(), TokenType::Or);
    }

    #[test]
    fn keyword_type_return() {
        assert_eq!(
            TokenType::keyword_type("return", 1).unwrap(),
            TokenType::Return
        );
    }

    #[test]
    fn keyword_type_invalid() {
        assert!(matches!(
            TokenType::keyword_type("unknown", 1),
            Err(LexError::UnsupportedKeyword(ref word, 1)) if word == "unknown"
        ));
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test]
    fn token_equals() {
        let token = Token::equals("=", 0, 1);
        assert_eq!(token.token_type, TokenType::Equals);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "=");
    }

    #[test]
    fn token_semicolon() {
        let token = Token::semicolon(";", 0, 2);
        assert_eq!(token.token_type, TokenType::Semicolon);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 2);
        assert_eq!(token.source, ";");
    }

    #[test]
    fn token_colon() {
        let token = Token::colon(":", 0, 1);
        assert_eq!(token.token_type, TokenType::Colon);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, ":");
    }

    #[test]
    fn token_comma() {
        let token = Token::comma(",", 0, 1);
        assert_eq!(token.token_type, TokenType::Comma);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, ",");
    }

    #[test]
    fn new_token() {
        let token = Token::new(TokenType::Identifier, 4..8, 1, "var name = 10;");
        assert_eq!(token.token_type, TokenType::Identifier);
        assert_eq!(token.range, 4..8);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "var name = 10;");
    }

    #[test]
    fn token_value() {
        let token = Token::new(TokenType::Identifier, 4..8, 1, "var name = 10;");
        assert_eq!(token.value(), "name");
    }

    #[test]
    fn token_owned_value() {
        let token = Token::new(TokenType::Identifier, 4..8, 1, "var name = 10;");
        assert_eq!(token.owned_value(), String::from("name"));
    }

    #[test]
    fn token_string_value() {
        let token = Token::new(TokenType::StringLiteral, 0..7, 1, "\"infer\"");
        assert_eq!(token.string_value(), "infer");

        let token2 = Token::new(TokenType::StringLiteral, 1..6, 1, "\"infer\"");
        assert_eq!(token2.string_value(), "infer");
    }

    #[test]
    fn token_plus() {
        let token = Token::plus("+", 0, 1);
        assert_eq!(token.token_type, TokenType::Plus);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "+");
    }

    #[test]
    fn token_minus() {
        let token = Token::minus("-", 0, 1);
        assert_eq!(token.token_type, TokenType::Minus);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "-");
    }

    #[test]
    fn token_star() {
        let token = Token::star("*", 0, 1);
        assert_eq!(token.token_type, TokenType::Star);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "*");
    }

    #[test]
    fn token_slash() {
        let token = Token::slash("/", 0, 1);
        assert_eq!(token.token_type, TokenType::Slash);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "/");
    }

    #[test]
    fn token_bang() {
        let token = Token::bang("!", 0, 1);
        assert_eq!(token.token_type, TokenType::Bang);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "!");
    }

    #[test]
    fn token_left_parentheses() {
        let token = Token::left_parentheses("(", 0, 1);
        assert_eq!(token.token_type, TokenType::LeftParentheses);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "(");
    }

    #[test]
    fn token_right_parentheses() {
        let token = Token::right_parentheses(")", 0, 1);
        assert_eq!(token.token_type, TokenType::RightParentheses);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, ")");
    }

    #[test]
    fn token_left_brace() {
        let token = Token::left_brace("{", 0, 1);
        assert_eq!(token.token_type, TokenType::LeftBrace);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "{");
    }

    #[test]
    fn token_right_brace() {
        let token = Token::right_brace("}", 0, 1);
        assert_eq!(token.token_type, TokenType::RightBrace);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "}");
    }

    #[test]
    fn token_greater_than() {
        let token = Token::greater_than(">", 0, 1);
        assert_eq!(token.token_type, TokenType::GreaterThan);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, ">");
    }

    #[test]
    fn token_less_than() {
        let token = Token::less_than("<", 0, 1);
        assert_eq!(token.token_type, TokenType::LessThan);
        assert_eq!(token.range, 0..1);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "<");
    }

    #[test]
    fn token_greater_than_equals() {
        let token = Token::greater_than_equals(">=", 0, 1);
        assert_eq!(token.token_type, TokenType::GreaterThanEquals);
        assert_eq!(token.range, 0..2);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, ">=");
    }

    #[test]
    fn token_less_than_equals() {
        let token = Token::less_than_equals("<=", 0, 1);
        assert_eq!(token.token_type, TokenType::LessThanEquals);
        assert_eq!(token.range, 0..2);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "<=");
    }

    #[test]
    fn token_equals_equals() {
        let token = Token::equals_equals("==", 0, 1);
        assert_eq!(token.token_type, TokenType::EqualsEquals);
        assert_eq!(token.range, 0..2);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "==");
    }

    #[test]
    fn token_bang_equals() {
        let token = Token::bang_equals("!=", 0, 1);
        assert_eq!(token.token_type, TokenType::BangEquals);
        assert_eq!(token.range, 0..2);
        assert_eq!(token.line, 1);
        assert_eq!(token.source, "!=");
    }
}
