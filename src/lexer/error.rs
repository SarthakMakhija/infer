/// Represents errors that can occur during the lexical analysis (tokenization) phase.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum LexError {
    /// Encountered a character that does not match any valid token.
    UnrecognizedChar(char, usize),
    /// A string literal was opened but not closed before the end of the file or line.
    UnterminatedStringLiteral(String, usize),
    /// Encountered an identifier matching a keyword format but which is currently unsupported.
    UnsupportedKeyword(String, usize),
}
