# Sway Standards

Just like many other smart contract languages, usage standards have been developed to enable cross compatibility between smart contracts.

For more information on using a Sway Standard, please refer to the [Sway-Standards Repository](https://github.com/FuelLabs/sway-standards).

## Standards

### Native Asset Standards

- [SRC-20; Native Asset Standard](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-20-native-asset.md) defines the implementation of a standard API for [Native Assets](../blockchain-development/native_assets.md) using the Sway Language.
- [SRC-3; Mint and Burn](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-3-minting-and-burning.md) is used to enable mint and burn functionality for Native Assets.
- [SRC-7; Arbitrary Asset Metadata Standard](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-7-asset-metadata.md) is used to store metadata for [Native Assets](../blockchain-development/native_assets.md), usually as NFTs.
- [SRC-9; Metadata Keys Standard](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-9-metadata-keys.md) is used to store standardized metadata keys for [Native Assets](../blockchain-development/native_assets.md) in combination with the SRC-7 standard.
- [SRC-6; Vault Standard](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-6-vault.md) defines the implementation of a standard API for asset vaults developed in Sway.

### Predicate Standards

- [SRC-13; Soulbound Address Standard](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-13-soulbound-address.md) defines a specific `Address` as a Soulbound Address for Soulbound Assets to become non-transferable.

### Access Control Standards

- [SRC-5; Ownership Standard](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-5-ownership.md) is used to restrict function calls to admin users in contracts.

### Contract Standards

- [SRC-12; Contract Factory](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-12-contract-factory.md) defines the implementation of a standard API for contract factories.

### Bridge Standards

- [SRC-8; Bridged Asset](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-8-bridged-asset.md) defines the metadata required for an asset bridged to the Fuel Network.
- [SRC-10; Native Bridge Standard](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-10-native-bridge.md) defines the standard API for the Native Bridge between the Fuel Chain and the canonical base chain.

### Documentation Standards

- [SRC-2; Inline Documentation](https://github.com/FuelLabs/sway-standards/blob/master/docs/src/src-2-inline-documentation.md) defines how to document your Sway files.

## Standards Support

Libraries have also been developed to support Sway Standards. These can be in [Sway-Libs](../reference/sway_libs.md).
