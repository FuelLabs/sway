# What is a Smart Contract?

<!-- This section should explain what is a smart contract -->
<!-- contract:example:start -->
A smart contract is no different than a script or predicate in that it is a piece of bytecode that is deployed to the blockchain via a [transaction](https://fuellabs.github.io/fuel-specs/master/protocol/tx_format). The main features of a smart contract that differentiate it from scripts or predicates are that it is _callable_ and _stateful_. Put another way, a smart contract is analogous to a deployed API with some database state.
<!-- contract:example:end -->

The interface of a smart contract, also just called a contract, can be explicitly defined with an [ABI declaration](#the-abi-declaration) or implicitly with an [impl Self](#impl-self-contracts) item for the special type "Contract". See [this contract](../examples/wallet_smart_contract.md) for an example on using ABIs.

## Syntax of a Smart Contract

As with any Sway program, the program starts with a declaration of what [program type](./index.md) it is. When using ABIs, a contract must either define or import an [ABI declaration](#the-abi-declaration) and implement it.

<!-- This section should explain best practices for ABIs -->
<!-- ABI:example:start -->
It is considered good practice to define your ABI in a separate library and import it into your contract. This allows callers of your contract to simply import the ABI directly and use it in their scripts to call your contract.
<!-- ABI:example:end -->

Let's take a look at an ABI declaration in a library:

```sway
{{#include ../../../../examples/wallet_abi/src/main.sw:abi_library}}
```

Let's focus on the ABI declaration and inspect it line-by-line.

### The ABI Declaration

```sway
{{#include ../../../../examples/wallet_abi/src/main.sw:abi}}
```

---

In the first line, `abi Wallet {`, we declare the name of this _Application Binary Interface_, or ABI. We are naming this ABI `Wallet`. To import this ABI into either a script for calling or a contract for implementing, you would use

```sway
{{#include ../../../../examples/wallet_smart_contract/src/main.sw:abi_import}}
```

---

In the second line,

```sway
{{#include ../../../../examples/wallet_abi/src/main.sw:receive_funds}}
```

we are declaring an ABI method called `receive_funds` which, when called, should receive funds into this wallet. Note that we are simply defining an interface here, so there is no _function body_ or implementation of the function. We only need to define the interface itself. In this way, ABI declarations are similar to [trait declarations](../advanced/traits.md). This particular ABI method does not take any parameters.

---

In the third line,

```sway
{{#include ../../../../examples/wallet_abi/src/main.sw:send_funds}}
```

we are declaring another ABI method, this time called `send_funds`. It takes two parameters: the amount to send, and the address to send the funds to.

>**Note**: The ABI methods `receive_funds` and `send_funds` also require the annotation `#[storage(read, write)]` because their implementations require reading and writing a storage variable that keeps track of the wallet balance, as we will see shortly. Refer to [Purity](
../blockchain-development/purity.md#Purity) for more information on storage annotations.

## Implementing an ABI for a Smart Contract

Now that we've discussed how to define the interface, let's discuss how to use it. We will start by implementing the above ABI for a specific contract.

Implementing an ABI for a contract is accomplished with `impl <ABI name> for Contract` syntax. The `for Contract` syntax can only be used to implement an ABI for a contract; implementing methods for a struct should use `impl Foo` syntax.

```sway
{{#include ../../../../examples/wallet_smart_contract/src/main.sw:abi_impl}}
```

You may notice once again the similarities between [traits](../advanced/traits.md) and ABIs. And, indeed, as a bonus, you can define methods in addition to the interface surface of an ABI, just like a trait. These pre-implemented ABI methods automatically become available as part of the contract interface that implements the corresponding ABI.

Note that the above implementation of the ABI follows the [Checks, Effects, Interactions](https://docs.soliditylang.org/en/v0.6.11/security-considerations.html#re-entrancy) pattern.

## The `ContractId` type

Contracts have an associated `ContractId` type in Sway. The `ContractId` type allows for Sway programs to refer to contracts in the Sway language. Please refer to the [ContractId](../basics/blockchain_types.md#contractid-type) section of the book for more information on `ContractId`s.

## Calling a Smart Contract from a Script

>**Note**: In most cases, calling a contract should be done from the [Rust SDK](../testing/testing-with-rust.md) or the [TypeScript SDK](https://docs.fuel.network/docs/fuels-ts) which provide a more ergonomic UI for interacting with a contract. However, there are situations where manually writing a script to call a contract is required.

Now that we have defined our interface and implemented it for our contract, we need to know how to actually _call_ our contract. Let's take a look at a contract call:

```sway
{{#include ../../../../examples/wallet_contract_caller_script/src/main.sw}}
```

The main new concept is the `abi cast`: `abi(AbiName, contract_address)`. This returns a `ContractCaller` type which can be used to call contracts. The methods of the ABI become the methods available on this contract caller: `send_funds` and `receive_funds`. We then directly call the contract ABI method as if it was just a regular method. You also have the option of specifying the following special parameters inside curly braces right before the main list of parameters:

1. `gas`: a `u64` that represents the gas being forwarded to the contract when it is called.
2. `coins`: a `u64` that represents how many coins are being forwarded with this call.
3. `asset_id`: a `b256` that represents the ID of the _asset type_ of the coins being forwarded.

Each special parameter is optional and assumes a default value when skipped:

1. The default value for `gas` is the context gas (i.e. the content of the special register `$cgas`). Refer to the [FuelVM specifications](https://fuellabs.github.io/fuel-specs/master/vm) for more information about context gas.
2. The default value for `coins` is 0.
3. The default value for `asset_id` is `b256::zero()`.

## Impl Self Contracts

In some cases, it may be more convenient to avoid declaring an ABI and implement the contract directly, as shown in the example below. 

Notice how there is no ABI specified in the `impl` item. In this case, the compiler will automatically create an ABI named as the package containing this `impl Contract` item, and will include each function in the ABI.

```sway
{{#include ../../../../examples/wallet_smart_contract_self_impl/src/main.sw:abi_impl}}
```

Without an ABI, there is no way for scripts and other contracts to use `abi(...)` and call this contract, but it can still be tested and called using any of available SDKs, as any other contract. 

The ABI name will be the "upper camel case" version of the package name containing the "impl Contract" item. `CONTRACT_ID` is a compiler special constant that references the contract being implemented in this file.

```sway
{{#include ../../../../examples/wallet_smart_contract_self_impl/src/main.sw:tests}}
```
