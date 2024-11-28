# The configuration of the Ignition network

## Chain config
- The `ChainId` is `0`.
- The initial `privileged_address` which can perform the network upgrade: b62c6f41c4c380d78ae67a8b432a4a41e2465383f8cc9869b67106835685c388
- The public address of the authority node that produces blocks: e8df6d432b2584a3ea9d0badf297c3b525bae71b577d22c1ccc12519adb64d92
- The block gas limit is `30000000`.

### Gas costs

The gas costs was created from the [benchmarks_fuel_core_0_26_0.json](benchmarks_fuel_core_0_26_0.json) benchmark results.
The `new_storage_per_byte` is manually set to be `63`.
The `vm_initialization.gas_per_unit` is manually set to be `0`.

### State transition
The state transition bytecode from [`0.26.0` release](https://github.com/FuelLabs/fuel-core/releases/download/v0.26.0/fuel-core-0.26.0-aarch64-apple-darwin.tar.gz).
This state transition function is used for any blocks produced with the `state_transition_bytecode_version` equal to `0`.

## State config
- The `coinbase` address hard coded in the genesis contract: 5a72f2b311ed44c5e9bbc70290c1f287abb56536bcb87eaaecdefcff628817a1
- The base asset contract source code is taken from [here](https://github.com/FuelLabs/fuel-bridge/tree/b0ebf0b01a903f1866156b7c370ff03d6fb4ec49/packages/base-asset).
  - The `ContractId` of the contract is `0x7e2becd64cd598da59b4d1064b711661898656c6b1f4918a787156b8965dc83c`.
  - The derived(`SubId` is `0000000000000000000000000000000000000000000000000000000000000000`) base `AssetId` from this contract is `0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07`.
