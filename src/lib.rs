pub mod ast;
pub mod infer;

pub use infer::{Infer, InferenceError};

// Keep compiler internal phases private to the crate
pub(crate) mod lexer;
pub(crate) mod parser;
