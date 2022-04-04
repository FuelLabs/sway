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
    }; // <------------ note this semicolon
}
```

In Sway, note that a _statement_ is a _declaration **or** expression with a semicolon after it_. This means that you need to add a semicolon after an `if` to turn it into a statement, if it is being used for control flow.

This need for a semicolon after if expressions to turn them into statements may be removed eventually.

### Using `if` in a `let` statement

Like Rust, `if`s are expressions in Sway. What this means is you can use `if` expressions on the right side of a `let` statement to assign the outcome to a variable.

```sway
let my_data = if some_bool < 10 { foo() } else { bar() };
```

Note that all branches of the `if` expression must return a value of the same type.

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

There are no `break` or `continue` keywords yet, but [they're coming](https://github.com/FuelLabs/sway/issues/587).

For now, the way to break out of a `while` loop early is to manually invalidate the condition. In this case, that just means setting `counter` to be >= 10.

Building on the previous example, here's what that might look like:

```sway
let mut counter = 0;
let mut break_early = false;
while counter < 10 {
    if break_early == true {
        // here we ensure the condition will evaluate to false, breaking the loop
        counter = 10
    } else {
        // calling some other function to set the bool value
        break_early = get_bool_value();
        counter = counter + 1;
    };
}
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
