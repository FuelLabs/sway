# Temporary Workarounds

## Storage Variables and Mappings

Storage variables (or more specifically, automatic assignment of storage slots) are not yet implemented. Storage slots will have to be assigned manually.

```sway
contract;

use std::hash::*;
use std::storage::*;

abi Store {
    fn store(x: u64, y: b256);
}

// Storage slot domain separator for a primitive
const STORAGE_SLOT_PRIMITIVE: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
// Storage slot domain separator for a mapping
const STORAGE_SLOT_MAPPING: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;

impl Store for Contract {
    fn store(x: u64, y: b256) {
        // Compute storage slot for primitive and store `x`
        let storage_slot_primitive = hash_value(STORAGE_SLOT_PRIMITIVE, HashMethod::Sha256);
        store(storage_slot_primitive, x);

        // Compute mapping slot for `y` and store `x`
        let storage_slot_mapping = hash_pair(STORAGE_SLOT_MAPPING, y, HashMethod::Sha256);
        store(storage_slot_mapping, x);
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
