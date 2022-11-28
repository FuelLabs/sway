# Logging

Logging is a way to record data as the program runs.

The [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) provides a [`logging`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/logging.sw) module which contains a [generic](../language/generics/index.md) `log` function that is used to log a variable of any type.

Each call to `log` appends 1 of 2 types of a [`receipt`](https://fuellabs.github.io/fuel-specs/master/protocol/abi/receipts.html) to the list of receipts

- [`Log`](https://fuellabs.github.io/fuel-specs/master/protocol/abi/receipts.html#log-receipt)
  - Generated for _non-reference_ types: `bool`, `u8`, `u16`, `u32`, and `u64`
- [`LogData`](https://fuellabs.github.io/fuel-specs/master/protocol/abi/receipts.html#logdata-receipt)
  - Generated for _reference_ types

The [Rust](https://fuellabs.github.io/fuels-rs/latest/) & [Typescript](https://fuellabs.github.io/fuels-ts/) SDKs may be used to decode the data.

## Example

To use the `log` function we must import it from the standard library and pass in any [generic](../language/generics/index.md) type `T` that we want to log.

```sway
{{#include ../../code/operations/logging/src/lib.sw:logging}}
```

In the example above a `u64` is used however we can pass in any [generic](../language/generics/index.md) type such as a [struct](../language/built-ins/structs.md), [enum](../language/built-ins/enums.md), [string](../language/built-ins/string.md) etc.
