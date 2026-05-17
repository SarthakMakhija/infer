#[derive(Debug)]
pub(crate) enum LexError {
    UnrecognizedChar(char),
    UnterminatedStringLiteral(String)
}
