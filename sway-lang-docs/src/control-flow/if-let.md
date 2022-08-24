# Using if & let together

`if`s are expressions in Sway. This means that you can use `if` expressions on the right side of a `let` statement to assign the outcome to a variable.

```sway
fn compute(input: u64) {
    let data = if input < 10 { foo() } else { bar() };
    // code
}
```

All branches of the `if` expression must return a value of the same type.
