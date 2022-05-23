# Hashing and Cryptography

The Sway standard library provides easy access to a selection of cryptographic hash functions (`sha256` and Ethereum-compatible `keccak256`), and Ethereum-compatible `secp256k1`-based signature recovery operations.

## Hashing

```sway
{{#include ../../../examples/hashing/src/main.sw}}
```

## Signature Recovery

```sway
{{#include ../../../examples/signatures/src/main.sw}}
```

Note: Recovery of consensus layer addresses (e.g. 20-byte Ethereum addresses) is also supported via modules in `std::vm`
