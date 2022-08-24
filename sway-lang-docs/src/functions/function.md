# Functions

In this section we will define a function that takes two numerical inputs and returns a boolean value indicating whether they are equal. We will also take a look at how to use the function.

### Declaration

The following function is called `equals` and it takes two parameters of type `u64` (64-bit unsigned integers). It performs a comparison and [implicitly](./return/implicit.md) returns the result of that comparison.

```sway
fn equals(first_parameter: u64, second_parameter: u64) -> bool {
    first_parameter == second_parameter
}
```

The `equals` function is currently private therefore if we want to have the ability to call the function in a different program we must add the `pub` keyword before the `fn` keyword.

```sway
pub fn equals(first_parameter: u64, second_parameter: u64) -> bool {
    first_parameter == second_parameter
}
```

> This is not enough to use the function externally. Refer to [libraries](../program-types/library.md) for more info.

### Usage

The following is a way to use the function defined above.

```sway
fn main() {
    let result_one = equals(5, 5);  // evaluates to `true`
    let result_two = equals(5, 6);  // evaluates to `false`
}
```
