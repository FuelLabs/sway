# Sway Standards

Just like many other smart contract languages, usage standards have been developed to enable cross compatibility between smart contracts.

For more information on using a Sway Standard, please refer to the [Sway-Standards Repository](https://github.com/FuelLabs/sway-standards).

## Standards

### Native Asset Standards

- [SRC-20; Native Asset Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-20.md) defines the implementation of a standard API for [Native Assets](../blockchain-development/native_assets.md) using the Sway Language.
- [SRC-3; Mint and Burn](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-3.md) is used to enable mint and burn functionality for Native Assets.
- [SRC-7; Arbitrary Asset Metadata Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-7.md) is used to store metadata for [Native Assets](../blockchain-development/native_assets.md), usually as NFTs.
- [SRC-9; Metadata Keys Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-9.md) is used to store standardized metadata keys for [Native Assets](../blockchain-development/native_assets.md) in combination with the SRC-7 standard.
- [SRC-6; Vault Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-6.md) defines the implementation of a standard API for asset vaults developed in Sway.

### Predicate Standards

- [SRC-13; Soulbound Address Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-13.md) defines a specific `Address` as a Soulbound Address for Soulbound Assets to become non-transferable.

### Access Control Standards

- [SRC-5; Ownership Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-5.md) is used to restrict function calls to admin users in contracts.

### Contract Standards

- [SRC-12; Contract Factory](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-12.md) defines the implementation of a standard API for contract factories.

### Bridge Standards

- [SRC-8; Bridged Asset](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-8.md) defines the metadata required for an asset bridged to the Fuel Network.
- [SRC-10; Native Bridge Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-10.md) defines the standard API for the Native Bridge between the Fuel Chain and the canonical base chain.

### Documentation Standards

- [SRC-2; Inline Documentation](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-2.md) defines how to document your Sway files.

## Standards Support

Libraries have also been developed to support Sway Standards. These can be in [Sway-Libs](../reference/sway_libs.md).
