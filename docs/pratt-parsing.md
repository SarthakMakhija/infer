# Pratt Parsing in `infer`

## What is Pratt Parsing?

Pratt parsing (also called top-down operator precedence parsing) is a technique for parsing expressions where 
each token carries its own precedence and associativity rules. Instead of encoding precedence into nested 
grammar rules, it is encoded directly into numeric levels. The parser uses a single recursive function that 
keeps consuming tokens as long as the next operator binds tighter than the current floor.

The key insight is:

> **Keep consuming the next operator only if its precedence is strictly greater than the current floor.**

This single rule gives you left-associativity, correct precedence nesting, and unary/prefix handling, 
all without separate grammar rules per precedence level.

---

## The Core Loop

In `src/parser/expr/mod.rs`, `parse_with_precedence` is the heart of the parser:

```rust
pub(crate) fn parse_with_precedence(
    &mut self,
    precedence: Precedence,
) -> Result<Expression, ParseError> {
    let token = self.stream.expect_token()?;
    let mut left = self.parse_prefix(&token)?;       // (1)

    while precedence < self.precedence()? {           // (2)
        let token = self.stream.expect_token()?;
        left = self.parse_infix(left, &token)?;       // (3)
    }

    Ok(left)
}
```

1. **Prefix step**: consume the first token and parse it as a prefix expression (a literal, identifier, unary operator, or opening parenthesis).
2. **Infix loop**: peek at the next token's precedence. If it is strictly greater than the current `precedence` floor, consume the token and fold it into `left` as an infix expression.
3. **Infix step**: the infix parser receives the already-parsed `left` and the operator token, then recursively parses the right-hand side.

The `precedence` argument acts as a floor: it says *"only absorb operators that bind tighter than this"*.

---

## Prefix and Infix Parsers

Every token type falls into one of two roles:

| Role   | When it appears               | Examples                                                 | Handled by                                                                                                       |
|--------|-------------------------------|----------------------------------------------------------|------------------------------------------------------------------------------------------------------------------|
| Prefix | At the start of an expression | `42`, `"hello"`, `true`, `-x`, `!flag`, `(expr)`, `name` | `WholeNumberParser`, `StringParser`, `BooleanParser`, `UnaryExpressionParser`, `GroupParser`, `IdentifierParser` |
| Infix  | Between two expressions       | `+`, `-`, `*`, `/`, `>`, `==`, `and`, `or`, `(` (call)   | `BinaryExpressionParser`, `FunctionCallParser`                                                                   |

Note that `(` plays both roles: as a prefix it starts a grouped expression `(a + b)`; as an infix it starts a function call `f(a, b)`.

---

## Precedence Levels

Defined in `src/parser/expr/precedence.rs`:

```rust
pub(crate) enum Precedence {
    None       = 0,
    Or         = 10,
    And        = 20,
    Equality   = 30,   // == !=
    Comparison = 40,   // > >= < <=
    Plus       = 50,   // + -
    Star       = 60,   // * /
    Unary      = 70,   // - !
    Call       = 80,   // f()
}
```

Higher number = binds more tightly.

### Why these levels?

Each level is chosen so that expressions compose the way mathematics and logic expect:

**`Call = 80` (tightest)**
A function call `f()` must bind tighter than anything else so that `f() + 1` is `(f()) + 1`, never `f(() + 1)`.

**`Unary = 70`**
Unary operators `-` and `!` bind tighter than binary operators so that `-a * b` is `(-a) * b`, not `-(a * b)`.

**`Star = 60` and `Plus = 50`**
Multiplication and division before addition and subtraction — standard arithmetic precedence. `1 + 2 * 3` is `1 + (2 * 3)`.

**`Comparison = 40` > `Equality = 30`**
This is the most important design choice. Consider:

```
a + b > c == true
```

We want this to parse as `(a + b > c) == true`, meaning:
- First compute `a + b` (arithmetic, highest among binary ops)
- Then compare `> c` (comparison)
- Then check `== true` (equality)

If `Equality` were higher than `Comparison`, `a + b > c == true` would parse as `(a + b) > (c == true)`, which is wrong — you'd be comparing a number against a boolean result.

Concretely: when the parser is sitting at `>` with floor `Equality(30)`, it checks `30 < 40` (Comparison) — true, so it absorbs `>`. When it is sitting at `==` with floor `Comparison(40)`, it checks `40 < 30` — false, so it stops. The comparison groups first.

**`And = 20` and `Or = 10`**
Logical operators bind looser than comparisons so that `a > 0 and b < 10` parses as `(a > 0) and (b < 10)` rather than `a > (0 and b) < 10`. `And` is higher than `Or` so that `a and b or c` parses as `(a and b) or c`, matching standard boolean logic.

**`None = 0` (floor)**
The initial call uses `Precedence::None` as the floor, meaning the loop will absorb any operator it encounters. It is also returned when peeking at a non-operator token (`;`, `}`, `)`, EOF), which stops the loop naturally.

---

## Left-Associativity

Left-associativity (the default for all operators here) falls out of passing the **same** precedence level as the floor when parsing the right-hand side.

For `a + b + c`:

1. Parse `a`. Floor = `None`.
2. Peek `+` (Plus=50 > None=0) → enter infix.
3. `BinaryExpressionParser` parses RHS with floor = `Plus(50)`.
4. Parse `b`. Peek `+` (Plus=50). Check `50 < 50` → **false**. Stop. RHS = `b`.
5. `left` = `Binary(a, +, b)`.
6. Back in outer loop: peek `+` (50 > 0) → enter infix again.
7. RHS = `c`. `left` = `Binary(Binary(a,+,b), +, c)`.

Result: `(a + b) + c` — left-associative. ✓

For right-associativity you would pass `precedence - 1` as the floor so the RHS can absorb equal-precedence operators. No operators in this language require right-associativity.

---

## Chained Comparison Guard

The language explicitly forbids chained comparisons like `a < b < c`. After parsing a binary comparison, if the new operator is also a comparison and the left-hand side (after unwrapping any `Grouped` layers) is already a comparison `Binary`, the parser rejects it:

```rust
if operator.is_comparison() {
    if let Expression::Binary(_, ref left_operator, _) = left.unwrap_grouped() {
        if left_operator.is_comparison() {
            return Err(ParseError::ChainedComparison(token.line));
        }
    }
}
```

`unwrap_grouped` recursively strips `Grouped` wrappers so that `(a > b) > c` is caught just as `a > b > c` is.

---

## Example: `base_price + tax_rate > budget`

Walking through the full parse with floor `None(0)`:

```
Tokens: base_price + tax_rate > budget
```

1. Prefix: consume `base_price` → `Identifier("base_price")`. Floor = 0.
2. Peek `+` (Plus=50). `0 < 50` → enter infix.
3. Consume `+`. `BinaryExpressionParser` with floor `Plus(50)`.
   - Prefix: consume `tax_rate` → `Identifier("tax_rate")`.
   - Peek `>` (Comparison=40). `50 < 40` → **false**. Stop.
   - RHS = `Identifier("tax_rate")`.
4. `left` = `Binary(base_price, +, tax_rate)`.
5. Back in outer loop: peek `>` (Comparison=40). `0 < 40` → enter infix.
6. Consume `>`. `BinaryExpressionParser` with floor `Comparison(40)`.
   - Prefix: consume `budget` → `Identifier("budget")`.
   - Peek: EOF → `None(0)`. `40 < 0` → false. Stop.
   - RHS = `Identifier("budget")`.
7. `left` = `Binary(Binary(base_price, +, tax_rate), >, budget)`.

Final AST: `(base_price + tax_rate) > budget` ✓

Arithmetic bound more tightly than comparison because `Plus(50) > Comparison(40)`.
