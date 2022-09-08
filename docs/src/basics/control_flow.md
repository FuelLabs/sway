# Control Flow

## `if` expressions

Sway supports _if_, _else_, and _else if_ expressions that allow you to branch your code depending on conditions.

For example:

```sway
fn main() {
    let number = 6;

    if number % 4 == 0 {
        // do something
    } else if number % 3 == 0 {
        // do something else
    } else {
        // do something else
    }
}
```

### Using `if` in a `let` statement

Like Rust, `if`s are expressions in Sway. What this means is you can use `if` expressions on the right side of a `let` statement to assign the outcome to a variable.

```sway
let my_data = if some_bool < 10 { foo() } else { bar() };
```

Note that all branches of the `if` expression must return a value of the same type.

### `match` expressions

Sway supports advanced pattern matching through exhaustive `match` expressions.

```sway
{{#include ../../../examples/match_statements/src/main.sw}}
```

## Loops

### `while`

Loops in Sway are currently limited to `while` loops. This is what they look like:

```sway
while counter < 10 {
    counter = counter + 1;
}
```

You need the `while` keyword, some condition (`value < 10` in this case) which will be evaluated each iteration, and a block of code inside the curly braces (`{...}`) to execute each iteration.

### `break` and `continue`

`break` and `continue` keywords are available to use inside the body of a `while` loop. The purpose of the `break` statement is to break out of a loop early:

```sway
{{#include ../../../examples/break_and_continue/src/main.sw:break_example}}
```

The purpose of the `continue` statement is to skip a portion of a loop in an iteration and jump directly into the next iteration:

```sway
{{#include ../../../examples/break_and_continue/src/main.sw:continue_example}}
```

### Nested loops

You can also use nested `while` loops if needed:

```sway
while condition_1 == true {
    // do stuff...
    while condition_2 == true {
        // do more stuff...
    }
}
```
