# What is a Smart Contract?

A smart contract is no different than a script or predicate in that it is a piece of bytecode that is deployed to the blockchain via a [transaction](https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/tx_format.md). The main features of a smart contract that differentiate it from scripts or predicates are that it is _callable_ and _stateful_. Put another way, a smart contract is analogous to a deployed API with some database state. The interface of a smart contract, also just called a contract, must be defined strictly with an [ABI declaration](#abi-declarations). See [this contract](../examples/subcurrency.md) for an example.

## Syntax of a Smart Contract

As with any Sway program, the program starts with a declaration of what [program type](./program_types.md) it is. A contract must also either define or import an [ABI declaration](#abi-declarations) and implement it. It is considered good practice to define your ABI in a separate library and import it into your contract. This allows callers of your contract to simply import the ABI directly and use it in their scripts to call your contract. Let's take a look at an ABI declaration in a library:

```sway
library wallet_abi;

abi Wallet {
    fn receive_funds(gas: u64, coins_to_forward: u64, asset_id: b256, unused: ());
    fn send_funds(gas: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest);
}

pub struct SendFundsRequest {
    amount_to_send: u64,
    recipient_address: b256,
}
```

There are two declarations going on here. One is a struct representing the data that `send_funds` needs and the other is the ABI declaration. Let's focus on the ABI declaration and inspect it line-by-line.

### The ABI Declaration

```sway
abi Wallet {
    fn receive_funds(gas: u64, coins_to_forward: u64, asset_id: b256, unused: ());
    fn send_funds(gas: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest);
}
```

---

In the first line, `abi Wallet {`, we declare the name of this _Application Binary Interface_, or ABI. We are naming this ABI `Wallet`. To import this ABI into either a script for calling or a contract for implementing, you would use `use wallet_abi::Wallet;`.

---

In the second line,

```sway
    fn receive_funds(gas: u64, coins_to_forward: u64, asset_id: b256, unused: ());
```

we are declaring an ABI interface surface method called `receive funds` which, when called, should receive funds into this wallet. Note that we are simply defining an interface here, so there is no _function body_ or implementation of the function. We only need to define the interface itself. In this way, ABI declarations are similar to [trait declarations](../advanced/traits.md). This ABI method takes four parameters: `gas`, `coins_to_forward`, `asset_id`, and `unused`, and doesn't return anything.

1. `gas` represents the gas being forwarded to the contract when it is called.
2. `coins_to_forward` represents how many coins are being forwarded with this call.
3. `asset_id` represents the ID of the _asset type_ of the coin being forwarded.
4. `unused` is the configurable user parameter, which this method does not need and is therefore unused.

**For now, all ABI methods must take these four parameters _in this order_. This will change shortly, and ABI methods will be able to accept any number of user-based parameters and not need to specify arguments for gas and coin forwarding.** You will see a compile error if you do not specify these parameters correctly in your ABI.

---

In the third line,

```sway
    fn send_funds(gas: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest);
```

we are declaring another ABI method, this time called `send_funds`. It takes the same parameters as the last ABI method, but with one difference: the fourth argument, the configurable one, is used. By specifying a struct here, you can pass in many values in this one parameter. In this case, `SendFundsRequest` simply has two values: the amount to send, and the address to send the funds to.

## Implementing an ABI for a Smart Contract

Now that we've discussed how to define the interface, let's discuss how to use it. We will start by implementing the above ABI for a specific contract.

Implementing an ABI for a contract is accomplished with _impl ABI_ syntax:

```sway
impl Wallet for Contract {
    fn receive_funds(gas_to_forward: u64, coins_to_forward: u64, asset_id: b256, unused: ()) {
        if asset_id == ETH_ID {
            let balance = storage.balance.write();
            deref balance = balance + coins_to_forward;
        };
    }

    fn send_funds(gas_to_forward: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest) {
        assert(sender() == OWNER_ADDRESS);
        assert(storage.balance.read() > req.amount_to_send);
        let balance = storage.balance.write();
        deref balance = balance - req.amount_to_send;
        transfer_coins(asset_id, req.recipient_address, req.amount_to_send);
    }
}
```

You may notice once again the similarities between [traits](../advanced/traits.md) and ABIs. And, indeed, as a bonus, you can specify methods in addition to the interface surface of an ABI, just like a trait. By implementing the methods in the interface surface, you get the extra method implementations For Free™.

Note that the above implementation of the ABI follows the [Checks, Effects, Interactions](https://docs.soliditylang.org/en/v0.6.11/security-considerations.html#re-entrancy) pattern.

## Calling a Smart Contract from a Script

Now that we have defined our interface and implemented it for our contract, we need to know how to actually _call_ our contract. Let's take a look at a contract call:

```sway
script;

use wallet_abi::Wallet;
use wallet_abi::SendFundsRequest;
use std::consts::ETH_ID;

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(Wallet, contract_address);
    let req = SendFundsRequest {
        amount_to_send: 200,
        recipient_address: 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b,
    };
    caller.send_funds(10000, 0, ETH_ID, req);
}
```

The main new concept is the _abi cast_: `abi(AbiName, contract_address)`. This returns a `ContractCaller` type which can be used to call contracts. The methods of the ABI become the methods available on this contract caller: `send_funds` and `receive_funds`. We then construct the request format, `SendFundsRequest`, and directly call the contract ABI method as if it was just a regular method.
