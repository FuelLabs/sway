# Temporary Workarounds

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

## Optimizer
