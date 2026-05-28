# Type Inference Compiler (infer) (WIP)
[![CI](https://github.com/SarthakMakhija/infer/actions/workflows/build.yml/badge.svg)](https://github.com/SarthakMakhija/infer/actions/workflows/build.yml)
[![Coverage](https://codecov.io/gh/SarthakMakhija/infer/graph/badge.svg?token=YD12LVN0A9)](https://codecov.io/gh/SarthakMakhija/infer)

_infer_ is an educational project for a statically typed toy programming language implemented in Rust. **Note: We are not building a full-featured, production-ready programming language.** The primary objective of this project is to serve as an educational showcase for designing and implementing **Constraint-Based Type Inference (Hindley-Milner style)** from scratch over a minimal set of language features.

The target language is a tiny programming language supporting:
*   **Variable Declarations** (with optional explicit type annotations).
*   **Expressions** (arithmetic operations, variable lookups, comparison operators, and logical composition).
*   **Conditionals** (`if`-`else` and `else-if` branches).
*   **For Loops** (iterative loops and `break` statements).
*   **Functions** (function declarations, parameters, return types, and invocation).

You can read the formal specification of the language in the [EBNF grammar file](./docs/grammar.ebnf), and track our completed milestones in the [Scope document](./docs/Scope.md).

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

### 2. Parser & AST: *Complete*
*   **Input:** The lazy stream of tokens from the Lexer.
*   **Output:** An untyped Abstract Syntax Tree (AST).
*   **Role:** Analyzes structural syntax using recursive descent parsing to construct a nested, hierarchical tree representation of the program (e.g., assignment nodes, conditional nodes, function definition nodes).

### 3. Type Inference Engine: *Next Phase*
*   **Input:** An untyped AST.
*   **Output:** A fully typed AST (or type errors).
*   **Role:** Collects type equations/constraints by traversing the AST, and then unifies those equations (similar to Hindley-Milner unification) to determine the exact type of every expression without requiring the programmer to write explicit types!

---

## Library Public API

The compiler is designed library-first. Internal compiler phases (`lexer`, `parser`) are encapsulated and kept private, while the AST is exposed via a safe, read-only inspection API.

### Basic Usage

To compile a source string and inspect the generated AST structure:

```rust
use infer::Infer;
use infer::ast::statement::Statement;

fn main() {
    let compiler = Infer::new();
    let source_code = "
        fn calculate_sum(first: i32, second: i32): i32 {
            var total = first + second;
        }
    ";

    // Parse into untyped-AST program or get detailed syntax errors
    let program = compiler.infer(source_code).unwrap();

    // Safely traverse the program statements
    for statement in program.statements() {
        if let Statement::FunctionDefinition(function) = statement {
            println!("Parsed function name: {}", function.name());
            println!("Return type annotation: {:?}", function.return_type());
            println!("Body statements: {}", function.body().len());
        }
    }
}
```

---

## Testing & Code Coverage

The compiler pipeline is covered by a comprehensive suite of unit and integration tests.

### Running Tests

To run the entire test suite (including unit and modular integration tests):

```bash
cargo test
```

### Measuring Code Coverage

We use `cargo-tarpaulin` to trace test line coverage in our local environments and Git pipelines. 

To run coverage locally:

```bash
cargo tarpaulin --out html
```
