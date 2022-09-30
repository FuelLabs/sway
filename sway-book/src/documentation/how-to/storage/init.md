# Initialization

When developing a [smart contract](../sway-program-types/smart_contracts.md), you will typically need some sort of persistent storage. In this case, persistent storage, often just called _storage_ in this context, is a place where you can store values that are persisted inside the contract itself. This is in contrast to a regular value in _memory_, which disappears after the contract exits.

Put in conventional programming terms, contract storage is like saving data to a hard drive. That data is saved even after the program which saved it exits. That data is persistent. Using memory is like declaring a variable in a program: it exists for the duration of the program and is non-persistent.

Some basic use cases of storage include declaring an owner address for a contract and saving balances in a wallet.

## Storage Accesses Via the `storage` Keyword

Declaring variables in storage requires a `storage` declaration that contains a list of all your variables, their types, and their initial values as follows:

```sway
{{#include ../../../../examples/storage_variables/src/main.sw:storage_declaration}}
```

To write into a storage variable, you need to use the `storage` keyword as follows:

```sway
{{#include ../../../../examples/storage_variables/src/main.sw:storage_write}}
```

To read a storage variable, you also need to use the `storage` keyword as follows:

```sway
{{#include ../../../../examples/storage_variables/src/main.sw:storage_read}}
```
