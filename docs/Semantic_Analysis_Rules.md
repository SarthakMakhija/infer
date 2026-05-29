# Semantic Analysis & Type Inference Rules

This document outlines the semantic analysis and type checking/inference rules that the compiler validates. 
These rules are divided into two main passes performed after parsing the Untyped AST.

---

## Semantic Analysis Passes

```mermaid
graph LR
    AST[Untyped AST] --> Pass1[Pass 1: Scoping & Structural Analysis]
    Pass1 --> Pass2[Pass 2: Type Analysis & Inference]
    Pass2 --> TypedAST[Typed & Validated AST]
```

### 1. Pass 1: Scoping & Structural Analysis
This pass walks the AST to establish symbol tables, resolve identifiers to their declarations, and validate that the program conforms to lexical scoping and structural layout rules.

To support forward-references (calling a function defined later in the file) and recursion, two standard 
implementation strategies can be used in this phase:

* **Strategy A: Two-Pass AST Traversal**
  * **Pass 1A (Signature Collection)**: Traverses only the top-level AST to extract function signatures, 
  populating the global function symbol table.
  * **Pass 1B (Resolution & Validation)**: Traverses the AST again, entering function bodies to resolve local 
  variables and validate function calls against the fully-populated symbol table.
  
* **Strategy B: Single-Pass Traversal with Deferred Resolution Log**
  * **Single Pass**: Walks the AST once, registering function definitions in the global symbol table and 
  immediately parsing function bodies. When a function call is seen, it is recorded in a **Deferred Call Sites** 
  log (e.g., `call to target with arguments inside source function`).
  * **Post-Validation**: Once the single traversal is complete, the compiler loops through the `call sites` log 
  and validates them against the gathered global definitions, emitting any arity or undefined function errors.

### 2. Pass 2: Type Analysis & Inference
This pass reconstructs the types of all expressions (resolving implicit or untyped parameters and variables using 
constraint generation and unification) and checks that operators and assignments are applied to compatible types.

---

## Rule Reference Table

|   #    | Rule Description                                                                                                                |            Validation Pass            | Failure Case Example                                                   |
|:------:|:--------------------------------------------------------------------------------------------------------------------------------|:-------------------------------------:|:-----------------------------------------------------------------------|
| **1**  | A variable must not be redeclared with the same name in the same scope.                                                         | **Pass 1: Scoping & Symbol Analysis** | `var x = 10; var x = 20;`                                              |
| **2**  | A variable must be declared (in either the current or any enclosing parent scope) before it can be assigned a value.            | **Pass 1: Scoping & Symbol Analysis** | `x = 42;` *(without a prior `var x;`)*                                 |
| **3**  | A `break` statement must have a lexically enclosing `loop` statement.                                                           | **Pass 1: Scoping & Symbol Analysis** | `fn main() { break; }`                                                 |
| **4**  | Duplicate function names are prohibited in the same global scope.                                                               | **Pass 1: Scoping & Symbol Analysis** | `fn foo() {} fn foo() {}`                                              |
| **5**  | Duplicate parameter names in a function's signature are prohibited.                                                             | **Pass 1: Scoping & Symbol Analysis** | `fn compute(a: i32, a: bool) {}`                                       |
| **6**  | Any statements placed directly after a terminal control-flow statement (`return` or `break`) in the same block are unreachable. | **Pass 1: Scoping & Symbol Analysis** | `return 5; var x = 10;`                                                |
| **7**  | A function call must reference a function that is defined (or will be defined later in the same file).                          | **Pass 1: Scoping & Symbol Analysis** | `undeclared_func();`                                                   |
| **8**  | A function call must pass exactly the same number of parameters as declared in its signature.                                   | **Pass 1: Scoping & Symbol Analysis** | `fn f(x, y) {}`<br>`f(1);` *(mismatch)*                                |
| **9**  | Operators must be applied to compatible operand types (e.g. `+` requires numeric operands).                                     | **Pass 2: Type Analysis & Inference** | `fn f(): bool { return true; }`<br>`f() + b;` *(bool cannot be added)* |
| **10** | If a function definition specifies an explicit return type, the function must return a value of that type.                      | **Pass 2: Type Analysis & Inference** | `fn f(): i32 { }` *(missing return value)*                             |
| **11** | A void/none returning function call cannot have its result bound to a variable.                                                 | **Pass 2: Type Analysis & Inference** | `fn f() {}`<br>`var x = f();` *(attempting to bind void)*              |
| **12** | Explicitly declared variables must have initializers that unify with their declared types.                                      | **Pass 2: Type Analysis & Inference** | `var x: i32 = "hello";`                                                |
| **13** | Assigning a value to an already-declared variable must match/unify with that variable's inferred/declared type.                 | **Pass 2: Type Analysis & Inference** | `var x = 10; x = "hello";`                                             |
| **14** | The conditional expression of an `if` statement must evaluate to or unify with `bool`.                                          | **Pass 2: Type Analysis & Inference** | `if 10 { }`                                                            |
