# Storage

When developing a [smart contract](../sway-on-chain/smart_contracts.md), you will typically need some sort of persistent storage. In this case, persistent storage, often just called _storage_ in this context, is a place where you can store values that are persisted inside the contract itself. This is in contrast to a regular value in _memory_, which disappears after the contract exits.

Put in conventional programming terms, contract storage is like saving data to a hard drive. That data is saved even after the program which saved it exits. That data is persistent. Using memory is like declaring a variable in a program: it exists for the duration of the program and is non-persistent.

Some basic use cases of storage include declaring an owner address for a contract and saving balances in a wallet.

## Storage By-Hand

Outside of the newer experimental `storage` syntax which is being stabalized, you can leverage FuelVM storage operations using the `store` and `get` methods provided in the standard (`std`) library.

With this approach you will have to design the internal key and value design manually.

An example is as follows:

```sway
contract;

use std::{
    storage::{get, store}
};

abi StorageExample {
    fn store_something(amount: u64);
    fn get_something() -> u64;
}

const STORAGE_KEY: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl StorageExample for Contract {
    fn store_something(amount: u64) {
        store(STORAGE_KEY, amount);
    }

    fn get_something() -> u64 {
        let value = get::<u64>(STORAGE_KEY);
        value
    }
}
```

If your looking to store b256 values by-hand, you will likely need the methods below as a temporary work around. 

As the new `storage` syntax is stabalized, these can be replace by a more standardized approach.

```sway
fn store_b256(key: b256, value: b256) {
    asm(r1: key, r2: value) {
        swwq r1 r2;
    };
}

fn get_b256(key: b256) -> b256 {
    asm(r1: key, r2) {
        srwq r2 r1;
        r2: T
    }
}
```


<!--
## Syntax

### Declaration

The syntax of declaring storage space in Sway looks like this:

```sway
storage {
    owner: b256,
}
```

It is very similar to a struct declaration, except with storage, you also have the option to specify an initial value:

```sway
storage {
    owner: 0xeeb578f9e1ebfb5b78f8ff74352370c120bc8cacead1f5e4f9c74aafe0ca6bfd,
}
```

This value is passed as a part of the transaction, which initializes storage upon contract deployment.

### Access

Storage access should be minimized, as it incurs a larger performance and gas cost than regular memory access. There are two types of storage access: _reading_ and _writing_.

#### Reading from Storage

Reading from storage is less expensive than writing. To read a value from storage, use the `.read()` method:

```sway
storage {
    owner: b256
}

impure fn get_owner() -> ref b256 {
    storage.owner.read()
}
```

This returns an immutable reference to a `b256` which is held in storage. The `read()` method itself copies the value from storage and returns a pointer to it to save on actual storage read opcodes, which are expensive. **This means that writing to a storage value will not update other variables that are holding references to that value acquired via `read()`**. If you'd like an actual `StorageRef` to the value itself, which does _not_ copy the value and instead incurs a storage read cost on every access, use `.direct_read()`.

#### Writing to Storage

Writing to storage is accomplished with the `.write()` method. The `.write()` method returns a special kind of mutable reference, called a `MutStorageRef`, which mutates storage directly upon every write. Writing to values of this type costs more gas than usual and should be minimized.

```sway
contract;

storage {
    owner: b256
}

impure fn main() {
    let mutable_owner_ptr = write_owner();
    deref mutable_owner_ptr = 0x27829e78404b18c037b15bfba5110c613a83ea22c718c8b51596e17c9cb1cd6f;
}

impure fn write_owner() -> MutStorageRef<b256> {
    storage.owner.write()
}
```

Note that to write to a mutable reference, you must dereference it first. See [the chapter on reference types](../basics/reference_types.md) for more information on reference types in general.
-->
