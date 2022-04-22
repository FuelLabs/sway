# Calling Contracts

The Sway standard library provides easy access to hashing (namely `sha256` and Ethereum compatible `keccak256`) operations and Ethereum-compatible Secp256k1-based signature recovery operations.

## Hashing

```sway
{{#include ../../../examples/hashing/src/main.sw}}
```

## Signature Recovery

```sway
{{#include ../../../examples/cryptography/src/main.sw}}
```