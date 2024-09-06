# The configuration of the local network

## Chain config
- The `ChainId` is `0`.
- The initial `privileged_address` which can perform the network upgrade:
    ```shell
    {"address":"9f0e19d6c2a6283a3222426ab2630d35516b1799b503f37b02105bebe1b8a3e9","secret":"d80a243ef91956f626d1dad2f23bdfeb73fd0b363282b1eb2227ac5964144afb","type":"block_production"}
    ```
- The public address of the authority node that produces blocks: 
    ```shell
    {"address":"e0a9fcde1b73f545252e01b30b50819eb9547d07531fa3df0385c5695736634d","secret":"4dd0cdca64ef56a01fc81891f9beb6d898f19a22b2e287bce91d807fdf46589a","type":"block_production"}
    ```
- The block gas limit is `30000000`.

### Gas costs

The gas costs was created from the [benchmarks_fuel_core_0_30_0.json](benchmarks_fuel_core_0_30_0.json) benchmark results.
The `new_storage_per_byte` is manually set to be `63`.
The `gtf` is manually set to be `13`.
The "jmpb", "jmpf", "jneb", "jnef", "jnzb", "jnzf" is manually set to be the same price as "jnei".

## State config
- The `coinbase` address hard coded in the genesis contract: 
    ```shell
    {"address":"7b4b30b2437b0073e5ba5a9324cf55831d180a89f66332b541827e12e647b751","secret":"9e24cfa071f6c1c4984a17ecf18061a8d0c9c304e7dd7703788bd122bd578650","type":"block_production"}
    ```
- Contains many wallets with fake ETHs:
  - Wallet 1:
    ```shell
    {"address":"6b63804cfbf9856e68e5b6e7aef238dc8311ec55bec04df774003a2c96e0418e","secret":"de97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c","type":"block_production"}
    ```
- The base asset contract source code is taken from [here](https://github.com/FuelLabs/fuel-bridge/tree/b0ebf0b01a903f1866156b7c370ff03d6fb4ec49/packages/base-asset).
  - The `ContractId` of the contract is `0x7e2becd64cd598da59b4d1064b711661898656c6b1f4918a787156b8965dc83c`.
  - The derived(`SubId` is `0000000000000000000000000000000000000000000000000000000000000000`) base `AssetId` from this contract is `0xf8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07`.
