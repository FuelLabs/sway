# Hashing

The [`hash module`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/hash.sw) contains the following functions:

<!-- no toc -->
- [`sha256`](sha256.md)
- [`keccak256`](keccak256.md)

They take one [`generic`](../../language/generics/index.md) argument `T` and return a [`b256`](../../language/built-ins/b256.md) (hash of `T`). 

To hash multiple values the values must be wrapped into one type such as a [`tuple`](../../language/built-ins/tuples.md), [`array`](../../language/built-ins/arrays.md), [`struct`](../../language/built-ins/structs.md) & [`enum`](../../language/built-ins/enums.md).
