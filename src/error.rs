#[derive(Debug)]
pub(crate) enum LexError {
    UnrecognizedChar(char),
}