# Missing Features

> TODO: also copied from the current book (some old version). Needs work

- [Issue: #1182](https://github.com/FuelLabs/sway/issues/1182)
  - Arrays in a storage block are not yet supported. See the [Manual Storage Management](https://fuellabs.github.io/sway/) section for details on how to use `store` and `get` from the standard library to manage storage slots directly. Note, however, that `StorageMap<K, V>` does support arbitrary types for `K` and `V` without any limitations.
- [Issue: #428](https://github.com/FuelLabs/sway/issues/428)
  - Arrays are currently immutable which means that changing elements of an array once initialized is not yet possible.
- [Issue: #2465](https://github.com/FuelLabs/sway/issues/2465) and [Issue: #1796](https://github.com/FuelLabs/sway/issues/1796)
  - It is not yet allowed to use `StorageMap<K, V>` as a component of a complex type such as a struct or an enum.
- [Issue: #2647](https://github.com/FuelLabs/sway/issues/2647)
  - Currently, it is only possible to define configuration-time constants that have primitive types and that are initialized using literals.

Ternary operator does not exist because `if` expressions cover that functionality
