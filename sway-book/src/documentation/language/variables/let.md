# let

The `let` keyword is used to assign a value to a variable during run-time. It can only be used inside of a [function](../functions/index.md) and its value can be changed when declared as [mutable](#mutable).

## Immutable

We can declare a variable that cannot have its value changed in the following way.

```sway
{{#include ../../../code/language/variables/src/lib.sw:immutable}}
```

By default `foo` is an immutable `u64` (more info [here](../built-ins/index.md#primitive-types)) with the value of `5`. This means that we can pass `foo` around and its value can be read however it cannot have its value changed from `5` to any other number.

## Mutable

We can declare a variable that can have its value changed through the use of the `mut` keyword.

```sway
{{#include ../../../code/language/variables/src/lib.sw:mutable}}
```
