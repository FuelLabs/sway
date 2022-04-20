# Storage

When developing a [smart contract](../sway-program-types/smart_contracts.md), you will typically need some sort of persistent storage. In this case, persistent storage, often just called _storage_ in this context, is a place where you can store values that are persisted inside the contract itself. This is in contrast to a regular value in _memory_, which disappears after the contract exits.

Put in conventional programming terms, contract storage is like saving data to a hard drive. That data is saved even after the program which saved it exits. That data is persistent. Using memory is like declaring a variable in a program: it exists for the duration of the program and is non-persistent.

Some basic use cases of storage include declaring an owner address for a contract and saving balances in a wallet.

## Manual Storage Management

Outside of the newer experimental `storage` syntax which is being stabalized, you can leverage FuelVM storage operations using the `store` and `get` methods provided in the standard library (`std`). Which currently works with primitive types.

With this approach you will have to manually assign the internal key used for storage.

An example is as follows:

```sway
{{#include ../../../examples/storage_example/src/main.sw}}
```

Note, if you are looking to store non-primitive types (e.g. b256), please refer to [this issue](https://github.com/FuelLabs/sway/issues/1229).
