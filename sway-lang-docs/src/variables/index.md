# Variables

Variables in Sway are _immutable by default_. This means that, by default, once a variable is declared, its value cannot change. This is one of the ways how Sway encourages safe programming, and many modern languages have this same default. Let's take a look at variables in detail.

## Declaring a Variable

Let's look at a variable declaration:

```sway
let foo = 5;
```

Great! We have just declared a variable, `foo`. What do we know about `foo`?

1. It is immutable.
1. Its value is `5`.
1. Its type is `u64`, a 64-bit unsigned integer.

`u64` is the default numeric type, and represents a 64-bit unsigned integer. See the section [Built-in Types](./built_in_types.md) for more details.

We can also make a mutable variable. Let's take a look:

```sway
let mut foo = 5;
foo = 6;
```

Now, `foo` is mutable, and the reassignment to the number `6` is valid. That is, we are allowed to _mutate_ the variable `foo` to change its value.
