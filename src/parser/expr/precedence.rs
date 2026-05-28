use crate::lexer::token::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum Precedence {
    None = 0,

    // or
    Or = 10,

    // and
    And = 20,

    // ==, !=
    Equality = 30,

    // >, <, >=, <=
    Comparison = 40,

    // + -
    Plus = 50,

    // * /
    Star = 60,

    // ! -
    Unary = 70,

    // f()
    Call = 80,
}

impl Precedence {
    pub(crate) fn of(token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::LeftParentheses => Precedence::Call,
            TokenType::Plus | TokenType::Minus => Precedence::Plus,
            TokenType::Star | TokenType::Slash => Precedence::Star,
            TokenType::EqualsEquals | TokenType::BangEquals => Precedence::Equality,
            TokenType::GreaterThan
            | TokenType::GreaterThanEquals
            | TokenType::LessThan
            | TokenType::LessThanEquals => Precedence::Comparison,
            TokenType::And => Precedence::And,
            TokenType::Or => Precedence::Or,
            _ => Precedence::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence_of_plus() {
        assert_eq!(Precedence::of(TokenType::Plus), Precedence::Plus);
    }

    #[test]
    fn precedence_of_minus() {
        assert_eq!(Precedence::of(TokenType::Minus), Precedence::Plus);
    }

    #[test]
    fn precedence_of_star() {
        assert_eq!(Precedence::of(TokenType::Star), Precedence::Star);
    }

    #[test]
    fn precedence_of_slash() {
        assert_eq!(Precedence::of(TokenType::Slash), Precedence::Star);
    }

    #[test]
    fn precedence_of_equals_is_none() {
        assert_eq!(Precedence::of(TokenType::Equals), Precedence::None);
    }

    #[test]
    fn precedence_of_semicolon_is_none() {
        assert_eq!(Precedence::of(TokenType::Semicolon), Precedence::None);
    }

    #[test]
    fn precedence_of_identifier_is_none() {
        assert_eq!(Precedence::of(TokenType::Identifier), Precedence::None);
    }

    #[test]
    fn precedence_of_var_is_none() {
        assert_eq!(Precedence::of(TokenType::Var), Precedence::None);
    }

    #[test]
    fn none_precedence_is_less_than_plus() {
        assert!(Precedence::None < Precedence::Plus);
    }

    #[test]
    fn plus_precedence_is_less_than_star() {
        assert!(Precedence::Plus < Precedence::Star);
    }

    #[test]
    fn none_precedence_is_less_than_star() {
        assert!(Precedence::None < Precedence::Star);
    }

    #[test]
    fn plus_precedence_equals_plus() {
        assert_eq!(Precedence::Plus, Precedence::Plus);
    }

    #[test]
    fn precedence_of_equals_equals() {
        assert_eq!(
            Precedence::of(TokenType::EqualsEquals),
            Precedence::Equality
        );
    }

    #[test]
    fn precedence_of_bang_equals() {
        assert_eq!(Precedence::of(TokenType::BangEquals), Precedence::Equality);
    }

    #[test]
    fn precedence_of_greater_than() {
        assert_eq!(
            Precedence::of(TokenType::GreaterThan),
            Precedence::Comparison
        );
    }

    #[test]
    fn precedence_of_less_than() {
        assert_eq!(Precedence::of(TokenType::LessThan), Precedence::Comparison);
    }

    #[test]
    fn precedence_of_greater_than_equals() {
        assert_eq!(
            Precedence::of(TokenType::GreaterThanEquals),
            Precedence::Comparison
        );
    }

    #[test]
    fn precedence_of_less_than_equals() {
        assert_eq!(
            Precedence::of(TokenType::LessThanEquals),
            Precedence::Comparison
        );
    }

    #[test]
    fn precedence_of_left_parentheses() {
        assert_eq!(Precedence::of(TokenType::LeftParentheses), Precedence::Call);
    }

    #[test]
    fn precedence_of_and() {
        assert_eq!(Precedence::of(TokenType::And), Precedence::And);
    }

    #[test]
    fn precedence_of_or() {
        assert_eq!(Precedence::of(TokenType::Or), Precedence::Or);
    }

    #[test]
    fn none_precedence_is_less_than_equality() {
        assert!(Precedence::None < Precedence::Equality);
    }

    #[test]
    fn equality_precedence_is_less_than_comparison() {
        assert!(Precedence::Equality < Precedence::Comparison);
    }

    #[test]
    fn comparison_precedence_is_less_than_plus() {
        assert!(Precedence::Comparison < Precedence::Plus);
    }
}
