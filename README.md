# Type Inference Compiler (infer) (WIP)
[![CI](https://github.com/SarthakMakhija/infer/actions/workflows/build.yml/badge.svg)](https://github.com/SarthakMakhija/infer/actions/workflows/build.yml)

_infer_ is an educational compiler project for a statically typed toy programming language implemented in Rust. **Note: We are not building a full-featured, production-ready programming language.** The primary objective of this project is to serve as an educational showcase for designing and implementing **Constraint-Based Type Inference (Hindley-Milner style)** from scratch over a minimal set of language features.

The target language is a tiny programming language supporting:
*   **Variable Declarations** (with optional explicit type annotations).
*   **Expressions** (arithmetic operations, variable lookups, etc.).
*   **Conditionals** (`if`-`else` branches).
*   **For Loops** (iterative loops).
*   **Functions** (function declarations, parameters, and invocation).

You can read the formal specification of the language in the [EBNF grammar file](./docs/grammar.ebnf).

---

## The Pipeline

_infer_ is built as a step-by-step linear pipeline. Each phase of the process is cleanly decoupled:

```text
Source Code
    │
    ▼
┌───────────┐    Token Stream    ┌────────────┐    Untyped AST    ┌───────────────────┐    Typed AST
│ 1. Lexer  │ ─────────────────> │ 2. Parser  │ ────────────────> │ 3. Type Inference │ ──────────────> Output
└───────────┘                    └────────────┘                   └───────────────────┘
```

### 1. Lexer (Scanner): *Complete*
*   **Input:** A raw string of source code characters (`&str`).
*   **Output:** An on-demand stream of lexical tokens (`Iterator<Item = LexResult>`).
*   **Role:** Performs character chunking, handles whitespace, extracts keywords, integers, strings, and symbols, and keeps track of source location (line numbers) for robust error reporting.

### 2. Parser & AST: *Next Phase*
*   **Input:** The lazy stream of tokens from the Lexer.
*   **Output:** An Abstract Syntax Tree (AST).
*   **Role:** Analyzes structural syntax using recursive descent parsing to construct a nested, hierarchical tree representation of the program (e.g., assignment nodes, conditional nodes, function definition nodes).

### 3. Type Inference Engine: *Core Objective*
*   **Input:** An untyped AST.
*   **Output:** A fully typed AST (or type errors).
*   **Role:** Collects type equations/constraints by traversing the AST, and then unifies those equations (similar to Hindley-Milner unification) to determine the exact type of every expression without requiring the programmer to write explicit types!

---

## Running Tests

The compiler pipeline is covered by a comprehensive suite of unit tests.

To run all tests:

```bash
cargo test
```
