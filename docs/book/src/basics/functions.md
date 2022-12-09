# Functions

Functions in Sway are declared with the `fn` keyword. Let's take a look:

```sway
fn equals(first_param: u64, second_param: u64) -> bool {
    first_param == second_param
}
```

We have just declared a function named `equals` which takes two parameters: `first_param` and `second_param`. The parameters must both be 64-bit unsigned integers.

This function also returns a `bool` value, i.e. either `true` or `false`. This function returns `true` if the two given parameters are equal, and `false` if they are not. If we want to use this function, we can do so like this:

```sway
fn main() {
    equals(5, 5); // evaluates to `true`
    equals(5, 6); // evaluates to `false`
}
```

## Mutable Parameters

We can make a function parameter mutable by adding `ref mut` before the parameter name. This allows mutating the argument passed into the function when the function is called. For example:

```sway
{{#include ../../../../examples/ref_mut_params/src/main.sw:increment}}
```

This function is allowed to mutate its parameter `num` because of the `mut` keyword. In addition, the `ref` keyword instructs the function to modify the argument passed to it when the function is called, instead of modifying a local copy of it.

```sway
{{#include ../../../../examples/ref_mut_params/src/main.sw:call_increment}}
```

Note that the variable `num` itself has to be declared as mutable for the above to compile.

> **Note**
> It is not currently allowed to use `mut` without `ref` or vice versa for a function parameter.

Similarly, `ref mut` can be used with more complex data types such as:

```sway
{{#include ../../../../examples/ref_mut_params/src/main.sw:tuple_and_enum}}
```

We can then call these functions as shown below:

```sway
{{#include ../../../../examples/ref_mut_params/src/main.sw:call_tuple_and_enum}}
```

> **Note**
> The only place, in a Sway program, where the `ref` keyword is valid is before a mutable function parameter.
