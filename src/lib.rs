//! The `infer` compiler library.
//!
//! Provides the core compiler pipeline phases (lexing, parsing, semantic analysis,
//! and type inference) for the toy language.

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
