# Temporary Workarounds

## Standard Library

The standard library is currently not distributed with `forc` if [installed via `cargo`](./installation.md#installing-from-cargo). It must be downloaded manually or specified as a dependency in the Forc manifest file. A variation of the following must be included in your project's `Forc.toml` file:

```toml
[dependencies]
core = { git = "http://github.com/FuelLabs/sway-lib-core", version = "v0.0.1" }
std = { git = "http://github.com/FuelLabs/sway-lib-std", version = "v0.0.1" }
```

Note that the default `Forc.toml` generated with `forc init` already includes these lines, so no further action is necessary.

## Explicit Parameters

For now, the first tree parameters of [an ABI method](../sway-on-chain/smart_contracts.md#the-abi-declaration) must be the amount of gas forwarded with the call, the amount of coins, and the asset ID of the coin (i.e. token type). A single fourth parameter is available (which could be a struct) for actual arguments. This restriction will be removed in the near future, such that only the actual arguments need to be declared.

## Storage Variables and Mappings

Storage variables (or more specifically, automatic assignment of storage slots) are not yet implemented. Storage slots will have to be assigned manually.

```sway
contract;

use std::hash::*;
use std::storage::*;

struct ParamsStore {
    x: 64,
    y: b256,
}

abi Store {
    fn store(gas: u64, coins: u64, asset_id: b256, args: ParamsStore);
}

// Storage slot domain separator for a primitive
const STORAGE_SLOT_PRIMITIVE: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
// Storage slot domain separator for a mapping
const STORAGE_SLOT_MAPPING: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;

impl Store for Contract {
    fn store(gas: u64, coins: u64, asset_id: b256, args: ParamsStore) {
        // Compute storage slot for primitive and store `x`
        let storage_slot_primitive = hash_value(STORAGE_SLOT_PRIMITIVE, HashMethod::Sha256);
        store(storage_slot_primitive, args.x);

        // Compute mapping slot for `y` and store `x`
        let storage_slot_mapping = hash_pair(STORAGE_SLOT_MAPPING, args.y, HashMethod::Sha256);
        store(storage_slot_mapping, args.x);
    }
}
```

## Serialization and Deserialization

Serialization/encoding of structures (Solidity's `abi.encode()` and `abi.encodePacked()`) is not yet implemented. Therefore, hashing an encoded struct is not possible without some manual work.

Serializing arbitrary structures can be accomplished manually by composition of recursive `hash_pair()` invocations. See the above example for hashing a pair of values.

## Optimizer

The optimizing pass of the compiler is not yet implemented, therefore bytecode will be more expensive and larger than it would be in production. Note that eventually the optimizer will support zero-cost abstractions, avoiding the need for developers to go down to inline assembly to produce optimal code.

## Formatter

Currently, we need to parse the Sway code before formatting it. Hence, **the formatter cannot work on Sway code that does not parse correctly**. This requirement may be changed in the future.
