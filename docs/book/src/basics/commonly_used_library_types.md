# Commonly Used Library Types

The Sway Standard Library is the foundation of portable Sway software, a set of minimal shared abstractions for the broader Sway ecosystem. It offers core types, library-defined operations on language primitives, native asset management, blockchain contextual operations, access control, storage management, and support for types from other VMs, among many other things. Reference the standard library docs [here](https://fuellabs.github.io/sway/master/std/index.html).

## `Result<T, E>`

Type `Result` is the type used for returning and propagating errors. It is an `enum` with two variants: `Ok(T)`, representing success and containing a value, and `Err(E)`, representing error and containing an error value. The `T` and `E` in this definition are type parameters, allowing `Result` to be generic and to be used with any types.

```sway
pub enum Result<T, E> {
    Ok: T,
    Err: E,
}
```

Functions return `Result` whenever errors are expected and recoverable. Take the following example:

In the `std` crate, `Result` is most prominently used for `Identity` interactions and cryptographic operations.

```sway
{{#include ../../../../examples/result/src/main.sw}}
```

## `Option<T>`

Type `Option` represents an optional value: every `Option` is either `Some` and contains a value, or `None`, and does not. `Option` types are very common in Sway code, as they have a number of uses:

- Initial values where `None` can be used as an initializer.
- Return value for otherwise reporting simple errors, where `None` is returned on error.

```sway
 pub fn unwrap(self) -> T {
    match self {
        Result::Ok(inner_value) => inner_value,
        _ => revert(0),
    }
}
```

The implementation of `Option` matches on the variant: if it's `Ok` it returns the inner value, if it's `None`, it [reverts](https://github.com/FuelLabs/fuel-specs/blob/master/src/vm/instruction_set.md#rvrt-revert).

`Option` is commonly paired with pattern matching to query the presence of a value and take action, allowing developers to choose how to handle the `None` case.

```sway
{{#include ../../../../examples/option/src/main.sw}}
```
