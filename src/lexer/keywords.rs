use std::collections::HashSet;

const KEYWORDS: &[&str] = &[
    "var", "true", "false", "if", "else", "break", "loop", "fn", "and", "or", "return",
];

/// A collection of reserved keywords in the language.
///
/// Used by the lexer to distinguish between general identifiers and structural keywords.
pub(crate) struct Keywords {
    vocabulary: HashSet<String>,
}

impl Keywords {
    /// Creates a new `Keywords` dictionary initialized with all supported reserved keywords.
    pub(crate) fn new() -> Self {
        Self {
            vocabulary: KEYWORDS.iter().map(|word| word.to_string()).collect(),
        }
    }

    /// Checks if a given word is a reserved keyword.
    pub(crate) fn has(&self, word: &str) -> bool {
        self.vocabulary.contains(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_a_valid_keyword() {
        assert!(Keywords::new().has("var"));
        assert!(Keywords::new().has("and"));
        assert!(Keywords::new().has("or"));
    }

    #[test]
    fn fn_is_a_valid_keyword() {
        assert!(Keywords::new().has("fn"));
    }

    #[test]
    fn is_not_a_keyword() {
        assert!(!Keywords::new().has("while"));
    }
}
