# What is a Smart Contract?

A smart contract is no different than a script or predicate in that it is a piece of bytecode that is deployed to the blockchain via a [transaction](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/tx_format.md). The main features of a smart contract that differentiate it from scripts or predicates are that it is _callable_ and _stateful_. Put another way, a smart contract is analogous to a deployed API with some database state. The interface of a smart contract, also just called a contract, must be defined strictly with an [ABI declaration](#the-abi-declaration). See [this contract](../examples/subcurrency.md) for an example.

## Syntax of a Smart Contract

As with any Sway program, the program starts with a declaration of what [program type](./index.md) it is. A contract must also either define or import an [ABI declaration](#the-abi-declaration) and implement it. It is considered good practice to define your ABI in a separate library and import it into your contract. This allows callers of your contract to simply import the ABI directly and use it in their scripts to call your contract. Let's take a look at an ABI declaration in a library:

```sway
library wallet_abi;

abi Wallet {
    fn receive_funds();
    fn send_funds(amount_to_send: u64, recipient_address: b256);
}
```

Let's focus on the ABI declaration and inspect it line-by-line.

### The ABI Declaration

```sway
abi Wallet {
    fn receive_funds();
    fn send_funds(amount_to_send: u64, recipient_address: b256);
}
```

---

In the first line, `abi Wallet {`, we declare the name of this _Application Binary Interface_, or ABI. We are naming this ABI `Wallet`. To import this ABI into either a script for calling or a contract for implementing, you would use `use wallet_abi::Wallet;`.

---

In the second line,

```sway
    fn receive_funds();
```

we are declaring an ABI method called `receive_funds` which, when called, should receive funds into this wallet. Note that we are simply defining an interface here, so there is no _function body_ or implementation of the function. We only need to define the interface itself. In this way, ABI declarations are similar to [trait declarations](../advanced/traits.md). This particular ABI method does not take any parameters.

---

In the third line,

```sway
    fn send_funds(amount_to_send: u64, recipient_address: Address);
```

we are declaring another ABI method, this time called `send_funds`. It takes two parameters: the amount to send, and the address to send the funds to.

## Implementing an ABI for a Smart Contract

Now that we've discussed how to define the interface, let's discuss how to use it. We will start by implementing the above ABI for a specific contract.

Implementing an ABI for a contract is accomplished with `impl <ABI name> for Contract` syntax. The `for Contract` syntax can only be used to implement an ABI for a contract; implementing methods for a struct should use `impl Foo` syntax.

```sway
impl Wallet for Contract {
    fn receive_funds() {
        if asset_id == NATIVE_ASSET_ID {
            let balance = storage.balance.write();
            deref balance = balance + coins_to_forward;
        };
    }

    fn send_funds(amount_to_send: u64, recipient_address: b256) {
        assert(sender() == OWNER_ADDRESS);
        assert(storage.balance.read() > amount_to_send);
        let balance = storage.balance.write();
        deref balance = balance - amount_to_send;
        transfer_coins(asset_id, recipient_address, amount_to_send);
    }
}
```

You may notice once again the similarities between [traits](../advanced/traits.md) and ABIs. And, indeed, as a bonus, you can specify methods in addition to the interface surface of an ABI, just like a trait. By implementing the methods in the interface surface, you get the extra method implementations For Free™.

Note that the above implementation of the ABI follows the [Checks, Effects, Interactions](https://docs.soliditylang.org/en/v0.6.11/security-considerations.html#re-entrancy) pattern.

## Calling a Smart Contract from a Script

Now that we have defined our interface and implemented it for our contract, we need to know how to actually _call_ our contract. Let's take a look at a contract call:

```sway
script;

use std::consts::NATIVE_ASSET_ID;

use wallet_abi::Wallet;
use wallet_abi::SendFundsRequest;

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(Wallet, contract_address);
    let amount_to_send = 200;
    let recipient_address: 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    caller.send_funds{gas: 10000, coins: 0, asset_id: NATIVE_ASSET_ID}(amount_to_send, recipient_address);
}
```

The main new concept is the _abi cast_: `abi(AbiName, contract_address)`. This returns a `ContractCaller` type which can be used to call contracts. The methods of the ABI become the methods available on this contract caller: `send_funds` and `receive_funds`. We then directly call the contract ABI method as if it was just a regular method. You also have the option of specifying the following special parameters inside curly braces right before the main list of parameters:

1. `gas`: represents the gas being forwarded to the contract when it is called.
2. `coins`: represents how many coins are being forwarded with this call.
3. `asset_id`: represents the ID of the _asset type_ of the coins being forwarded.

Each special parameter is optional and assumes a default value when skipped.
