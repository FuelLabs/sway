# Functions
Functions in Sway are declared with the `fn` keyword. Let's take a look:
```
fn equals(first_param: u64, second_param: u64) -> bool {
  first_param == second_param
}
```

We have just declared a function named `equals` which takes two parameters: `first_param` and `second_param`. This function also returns a `bool` value, i.e. either `true` or `false`. This function returns `true` if the two given parameters are equal, and `false` if they are not. If we want to use this function, we can do so like this:
```
fn main() {
  equals(5, 5); // evaluates to `true`
  equals(5, 6); // evaluates to `false`
}
```

