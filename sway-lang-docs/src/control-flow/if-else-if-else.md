# if expressions

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

Note that each conditional expression must evaluate to a boolean (true or false). This means that you cannot do something like `if 7 { ... }` or `if <some vector variable> { ... }`.
