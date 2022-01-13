# Identifiers

Addresses in Sway are similar to Ethereum addresses. The two major differences are:

1. Sway addresses are 32 bytes long (instead of 20), and
1. are computed with the SHA-256 hash of the public key instead of the keccak-256 hash.

Contracts, on the other hand, are uniquely identified with a contract ID rather than an address. A contract's ID is also 32 bytes long and is calculated [here](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/identifiers.md#contract-id).
