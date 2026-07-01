#![allow(dead_code)]

#[macro_use]
pub(crate) mod macros;

pub mod ast;
pub mod infer;

pub use infer::{Infer, InferenceError};

// Keep compiler internal phases private to the crate
pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod semantic;
