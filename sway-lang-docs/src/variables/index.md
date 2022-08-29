# Variables

Sway has two types of variables:

- [immutable](#immutable)
  - Can be read but cannot be changed after it has been declared
- [mutable](#mutable)
  - Can be read and have its value changed but only if the type is the same

By default all variables in Sway are immutable unless declared as mutable. This is one of the ways how Sway encourages safe programming, and many modern languages have the same default.

## Declaring a Variable

### Immutable

Let's declare a variable that cannot be changed and it has the value of `5`.

```sway
let foo = 5;
```

By default `foo` is an immutable `u64` (more info [here](../built-ins/index.md#primitive-types)) with the value of `5`. This means that we can pass `foo` around and its value can be read however it cannot have its value changed from `5` to any other number.

### Mutable

This time we want to declare a variable that can have its value changed. Let's also give it the value of `5` and then change it to `6`.

```sway
let mut foo = 5;
foo = 6;
```

Using the `mut` keyword marks the variable `foo` as mutable which means we can change its value to another value of the same type, in this case from `5` to `6`.
