# Standard Library Prelude

The [prelude](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/prelude.sw) is a list of commonly used features from the [standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std) which is automatically imported into every Sway program.

The prelude contains the following:

- [`Address`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/address.sw): A struct containing a `b256` value which represents the wallet address
- [`ContractId`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/contract_id.sw) A struct containing a `b256` value which represents the ID of a contract
- [`Identity`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/identity.sw): An enum containing `Address` & `ContractID` structs
- [`Vec`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/vec.sw): A growable, heap-allocated vector
- [`StorageMap`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/storage.sw): A key-value mapping in contract storage
- [`Option`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/option.sw): An enum containing either some generic value `<T>` or an absence of that value, we also expose the variants directly:
  - `Some`
  - `None`
- [`Result`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/result.sw): An enum used to represent either a success or failure of an operation, we also expose the variants directly:
  - `Ok`
  - `Err`
- [`assert`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/assert.sw): A module containing
  - `assert`: A function that reverts the VM if the condition provided to it is false
  - `assert_eq`: A function that reverts the VM and logs its two inputs v1 and v2 if the condition v1 == v2 is false
- [`revert`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/revert.sw): A module containing
  - `require`: A function that reverts and logs a given value if the condition is `false`
  - `revert`: A function that reverts
- [`log`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/logging.sw): A function that logs arbitrary stack types
- [`msg_sender`](https://github.com/FuelLabs/sway/blob/master/sway-lib-std/src/auth.sw): A function that gets the Identity from which a call was made
