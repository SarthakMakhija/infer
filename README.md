# Type Inference Compiler (infer)

`infer` is a compiler for a statically typed toy programming language implemented in Rust. The primary objective of this project is to serve as an educational showcase for designing and implementing **Constraint-Based Type Inference (Hindley-Milner style)** from scratch.

The target language is a tiny programming language supporting:
*   **Variable Declarations** (with optional explicit type annotations).
*   **Expressions** (arithmetic operations, variable lookups, etc.).
*   **Conditionals** (`if`-`else` branches).
*   **For Loops** (iterative loops).
*   **Functions** (function declarations, parameters, and invocation).

You can read the formal specification of the language in the [EBNF grammar file](./docs/grammar.ebnf).

---

## The Compiler Pipeline

This compiler is built as a step-by-step linear pipeline. Each phase of the compilation process is cleanly decoupled:

```mermaid
graph LR
    Source[Source Code] --> Lexer[1. Lexer]
    Lexer -->|Token Stream| Parser[2. Parser]
    Parser -->|AST| AST[3. AST Representation]
    AST -->|Untyped AST| Inference[4. Type Inference Engine]
    Inference -->|Fully Typed AST| Output[Execution / Valid Program]
```

### 1. Lexer (Scanner) — *Complete*
*   **Input:** A raw string of source code characters (`&str`).
*   **Output:** An on-demand stream of lexical tokens (`Iterator<Item = LexResult>`).
*   **Role:** Performs character chunking, handles whitespace, extracts keywords, integers, strings, and symbols, and keeps track of source location (line numbers) for robust error reporting.

### 2. Parser — *Next Phase*
*   **Input:** The lazy stream of tokens from the Lexer.
*   **Output:** An Abstract Syntax Tree (AST).
*   **Role:** Analyzes structural syntax to verify grammatical validity using recursive descent parsing.

### 3. Abstract Syntax Tree (AST) — *Next Phase*
*   **Role:** Represents the nested, hierarchical tree structure of the program statements (e.g., assignment nodes, conditional nodes, function definition nodes).

### 4. Type Inference Engine — *Core Objective*
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
