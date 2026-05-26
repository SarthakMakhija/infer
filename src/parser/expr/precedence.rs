use crate::lexer::token::TokenType;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum Precedence {
    None = 0,
    Plus = 10,
    Star = 20,
}

impl Precedence {
    pub(crate) fn of(token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::Plus | TokenType::Minus => Precedence::Plus,
            TokenType::Star | TokenType::Slash => Precedence::Star,
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
}
