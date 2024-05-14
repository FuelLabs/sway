# Wallet Smart Contract

The ABI declaration is a separate project from your ABI implementation. The project structure for the code should be organized as follows with the `wallet_abi` treated as an external library:

```sh
.
├── wallet_abi
│   ├── Forc.toml
│   └── src
│       └── main.sw
└── wallet_smart_contract
    ├── Forc.toml
    └── src
        └── main.sw
```

It's also important to specify the source of the dependency within the project's `Forc.toml` file when using external libraries. Inside the `wallet_smart_contract` project, it requires a declaration like this:

```sh
[dependencies]
wallet_abi = { path = "../wallet_abi/" }
```

## ABI Declaration

```sway
{{#include ../../../../examples/wallet_abi/src/main.sw:abi_library}}
```

## ABI Implementation

```sway
{{#include ../../../../examples/wallet_smart_contract/src/main.sw:full_wallet}}
```
