# The configuration of the Ignition network

## Chain config
- The `ChainId` is `9889`.
- The initial `privileged_address` which can perform the network upgrade: `eb42c979046d091f4f524e1efd8d4ffad92629b9a506b268a511861b0dbf2366`
- The genesis public address of the authority node that produces blocks: `da6b021bf570f126869591f2009a95efb826f69272987db699fdea1de813697e`
- The block gas limit is `30000000`.
- Genesis state transition version is set to `10` - corresponding release is `fuel-core 0.35.0`.

### Gas costs

The gas costs was created from the [benchmarks_fuel_core_0_35_0.json](benchmarks_fuel_core_0_35_0.json) benchmark results.
The `gas_per_byte` is manually set to be `233`.
The `new_storage_per_byte` is manually set to be `233`.

### State transition
The state transition bytecode from [`0.35.0` release](https://github.com/FuelLabs/fuel-core/releases/download/v0.35.0/fuel-core-0.35.0-aarch64-apple-darwin.tar.gz).
This state transition function is used for any blocks produced with the `state_transition_bytecode_version` equal to `10`.

## State config
- The `coinbase` address hard coded in the genesis contract: 15df2400bbf43bfa8f01cc97c69ecb541797d6d72a4fcea199c0f3b8d7303f15
- The base asset contract source code is taken from [here](https://github.com/FuelLabs/sway-standard-implementations/pull/22). It is a proxy contract with empty implementation.
  - The owner of the base asset proxy is `fc96a3a99ae1873e9e571a8be7d14111a2b4b7bd3abacb367c6e0f79c9c149d9`.
  - The `ContractId` of the contract is `7e2becd64cd598da59b4d1064b711661898656c6b1f4918a787156b8965dc83c`.
  - The derived(`SubId` is `0000000000000000000000000000000000000000000000000000000000000000`) base `AssetId` from this contract is `0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07`.
