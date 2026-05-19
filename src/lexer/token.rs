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
    /// A user-defined identifier (e.g. variable names or type names).
    Identifier,
    /// The `var` structural keyword.
    Var,
    /// A numeric integer literal.
    WholeNumber,
    /// A double-quoted string literal.
    StringLiteral,
    /// The `true` boolean literal keyword.
    True,
    /// The `false` boolean literal keyword.
    False,
    /// The `+` operator.
    Plus,
    /// The `-` operator.
    Minus,
    /// The `*` operator.
    Star,
    /// The `/` operator.
    Slash,
}

impl TokenType {
    /// Maps a string slice lexeme to its corresponding reserved keyword `TokenType`.
    /// Returns `LexError::UnsupportedKeyword` if the slice is not a recognized keyword.
    pub(crate) fn keyword_type(token: &str, line: usize) -> Result<Self, LexError> {
        match token {
            "var" => Ok(TokenType::Var),
            "true" => Ok(TokenType::True),
            "false" => Ok(TokenType::False),
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

    /// Returns `true` if this token's type is `TokenType::Var`.
    pub(crate) fn is_var(&self) -> bool {
        matches!(self.token_type, TokenType::Var)
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
        assert_eq!(TokenType::keyword_type("true", 1).unwrap(), TokenType::True);
    }

    #[test]
    fn keyword_type_false() {
        assert_eq!(
            TokenType::keyword_type("false", 1).unwrap(),
            TokenType::False
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
    fn token_is_var() {
        let token = Token::new(TokenType::Var, 0..3, 1, "var");
        assert!(token.is_var());
    }

    #[test]
    fn token_is_not_var() {
        let token = Token::new(TokenType::Identifier, 0..4, 1, "name");
        assert!(!token.is_var());
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
}
