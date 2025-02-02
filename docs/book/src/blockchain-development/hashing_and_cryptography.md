# Hashing and Cryptography

The Sway standard library provides easy access to a selection of cryptographic hash functions (`sha256` and EVM-compatible `keccak256`), and EVM-compatible `secp256k1`-based signature recovery operations.

## Hashing

```sway
{{#include ../../../../examples/hashing/src/main.sw}}
```

## Cryptographic Signature Recovery and Verification

Fuel supports 3 asymmetric cryptographic signature schemes; `Secp256k1`, `Secp256r1`, and `Ed25519`.

### Public Key Recovery

Given a `Signature` and a sign `Message`, you can recover a `PublicKey`.

```sway
{{#include ../../../../examples/signatures/src/main.sw:public_key_recovery}}
```

### Signed Message Address Recovery

Given a `Signature` and signed `Message`, you can recover a Fuel `Address`.

```sway
{{#include ../../../../examples/signatures/src/main.sw:address_recovery}}
```

#### Signed Message EVM Address Recovery

Recovery of EVM addresses is also supported.

```sway
{{#include ../../../../examples/signatures/src/main.sw:evm_address_recovery}}
```

### Public Key Signature Verification

Given a `Signature`, `PublicKey`, and `Message`, you can verify that the message was signed using the public key.

```sway
{{#include ../../../../examples/signatures/src/main.sw:signature_verification}}
```

### Address Signature Verification

Given a `Signature`, `Address`, and `Message`, you can verify that the message was signed by the address.

```sway
{{#include ../../../../examples/signatures/src/main.sw:address_verification}}
```

#### EVM Address Signature Verification

Recovery of EVM addresses verification is also supported.

```sway
{{#include ../../../../examples/signatures/src/main.sw:evm_address_verification}}
```
