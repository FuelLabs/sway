# if expressions

Sway supports _if_, _else_, and _else if_ expressions which provide control over which instructions should be executed depending on the conditions.

## Conditional Branching

In the following example we have a function which takes some number and depending on the value it will call one of three different functions.

```sway
{{#include ../../../code/language/control_flow/src/lib.sw:conditional}}
```

First, we check if the number is divisibly by three through the use of a modulo operation (`%`) and compare the result to the value of zero (`if number % 3 == 0`). 

If the number is divisibe by three then the condition will be evaluated to `true` and we will call function `1` after which we will exit out of our `magic_function()`.

If the first condition evaluates to `false` then we check the next condition (`else if number % 4 == 0`) and the same logic applies.

> You can have as many _else if_ conditions as you like, in the example we only use one.

If neither of those two conditions are met then we fall back into a "catch all" case `else` where we call function `3`.
 
> Each conditional expression must evaluate to a [boolean](../built-ins/boolean.md). This means that you cannot do something like `if 7 { ... }` because `7` does not evaluate to a bool.

## Using if & let together

In [Conditional Branching](#conditional-branching) we have opted to perform some logic (call some function) depending on which condition is met however that is not the only thing we can do. Since `if`'s are expressions in Sway we can use them with `let` statements to assign the result of an expression to a variable.

```sway
{{#include ../../../code/language/control_flow/src/lib.sw:compute}}
```

The function `compute()` takes a number (`deposit`) and checks to see if it is greater than the value of `10`. If the value is greater than `10` then `minimum_deposit_met` will take the value of `true` otherwise it will be set to `false`.

> All branches of the `if` expression must return a value of the same type.
