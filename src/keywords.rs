use std::collections::HashSet;

const KEYWORDS: &[&str] = &["var"];

pub(crate) struct Keywords {
    vocabulary: HashSet<String>,
}

impl Keywords {
    pub(crate) fn new() -> Self {
        Self {
            vocabulary: KEYWORDS.iter().map(|word| word.to_string()).collect(),
        }
    }

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
    }

    #[test]
    fn is_not_a_keyword() {
        assert!(!Keywords::new().has("while"));
    }
}
