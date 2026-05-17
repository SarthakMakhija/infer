/// Represents errors that can occur during the lexical analysis (tokenization) phase.
#[derive(Debug)]
pub(crate) enum LexError {
    /// Encountered a character that does not match any valid token.
    UnrecognizedChar(char),
    /// A string literal was opened but not closed before the end of the file or line.
    UnterminatedStringLiteral(String),
    /// Encountered an identifier matching a keyword format but which is currently unsupported.
    UnsupportedKeyword,
}
