# _If_ expressions

Sway supports _if_, _else_, and _else if_ expressions that allow you to branch your code depending on conditions.

For example:

```sway
fn main() {
    if something_is_true {
        do_this();
    } else {
        do_that();
    };   // <------------ note this semicolon
}
```

In Sway, note that a _statement_ is a _declaration **or** expression with a semicolon after it_. This means that you need to add a semicolon after an `if` to turn it into a statement, if it is being used for control flow:

This need for a semicolon after if expressions to turn them into statements will be removed eventually, but it hasn't been removed yet.

## Using _if_ in a _let_ statement

Like Rust, ifs are expressions in Sway. What this means is you can use _if_ expressions on the right side of a `let` statement to assign the outcome to a variable.

```sway
let my_data = if some_bool < 10 { foo() } else { bar() };
```




