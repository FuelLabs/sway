# Standard Library Prelude

The [prelude](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/prelude.sw) is a list of commonly used features from the [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) which is automatically imported into every Sway program.

The prelude contains the following modules:

- [`Address`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/address.sw): A struct containing a `b256` value which represents the wallet address
- [`assert`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/assert.sw): A function that reverts if the condition provided is `false`
- [`ContractId`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/contract_id.sw) A struct containing a `b256` value which represents the ID of a contract
- [`Identity`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/identity.sw): An enum containing `Address` & `ContractID` structs
- [`Option`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/option.sw): An enum containing either some generic value `<T>` or an absence of that value
- [`Result`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/result.sw): An enum used to represent either a success or failure of an operation
- [`revert`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/revert.sw): A module containing
  - `require`: A function that reverts and logs a given value if the condition is `false`
  - `revert`: A function that reverts
- [`StorageMap`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/storage.sw): A key-value mapping in contract storage
- [`Vec`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/vec.sw): A growable, heap-allocated vector
