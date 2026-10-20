# Appendix: Operators & Symbols

This appendix is a quick reference for the operators and symbolic tokens
available in Sway. It complements the [Keywords](./keywords.md) page and the
[Basics](../basics/index.md) sections.

## Arithmetic Operators

| Operator | Meaning | Example |
| -------- | ------- | ------- |
| `+` | Addition | `a + b` |
| `-` | Subtraction | `a - b` |
| `*` | Multiplication | `a * b` |
| `/` | Division | `a / b` |
| `%` | Remainder | `a % b` |

## Comparison Operators

| Operator | Meaning | Example |
| -------- | ------- | ------- |
| `==` | Equal to | `a == b` |
| `!=` | Not equal to | `a != b` |
| `>`  | Greater than | `a > b` |
| `<`  | Less than | `a < b` |
| `>=` | Greater than or equal | `a >= b` |
| `<=` | Less than or equal | `a <= b` |

## Logical Operators

| Operator | Meaning | Example |
| -------- | ------- | ------- |
| `&&` | Logical AND | `a && b` |
| `||` | Logical OR | `a || b` |
| `!`  | Logical NOT | `!a` |

## Bitwise Operators

| Operator | Meaning | Example |
| -------- | ------- | ------- |
| `&`  | Bitwise AND | `a & b` |
| `|`  | Bitwise OR | `a | b` |
| `^`  | Bitwise XOR | `a ^ b` |
| `<<` | Left shift | `a << b` |
| `>>` | Right shift | `a >> b` |

## Assignment Operators

| Operator | Meaning | Example |
| -------- | ------- | ------- |
| `=`   | Assign | `let a = b;` |
| `+=`  | Add and assign | `a += b` |
| `-=`  | Subtract and assign | `a -= b` |
| `*=`  | Multiply and assign | `a *= b` |
| `/=`  | Divide and assign | `a /= b` |
| `%=`  | Remainder and assign | `a %= b` |

## Symbolic Tokens

| Symbol | Meaning |
| ------ | ------- |
| `::` | Path separator, e.g. `std::vec::Vec` |
| `->` | Return type, e.g. `fn f() -> u64` |
| `=>` | Match arm, e.g. `0 => "zero"` |
| `@`  | Read storage value, e.g. `@storage_field` |
| `..` | Range, e.g. `0..10` |
| `..=` | Inclusive range, e.g. `0..=10` |
| `_`  | Wildcard / unused binding |
| `?`  | Propagate error from `Result` |

## Compound Type Constructors

| Symbol | Meaning | Example |
| ------ | ------- | ------- |
| `[]` | Array / vector literal | `let v = [1, 2, 3];` |
| `()` | Tuple literal | `let t = (1, true);` |
| `{}` | Block / struct literal | `{ let x = 1; x }` |
