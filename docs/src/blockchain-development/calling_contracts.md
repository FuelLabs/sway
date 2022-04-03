# Calling Contracts

End users and smart contracts typically need to communicate with each other.

The FuelVM achieves end user and inter contract communication primarily with the [`call`](https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/opcodes.md#call-call-contract) opcode.

Sway provides a nice way to manage calls with its `abi` system.

## Script Calls

Here is an example of a *script calling a contract* in Sway:

```rs
script;

abi MyContract {
    fn foo(field_1: bool, field_2: u64);
}

const contract_id = 0x79fa8779bed2f36c3581d01c79df8da45eee09fac1fd76a5a656e16326317ef0;

fn main() {
    let x = abi(MyContract, contract_id);
    x.foo(true, 3);
}
```

## Inter Contract Calls

Here is an example of a *contract calling another contract* in Sway:

```rs
// ./contract_a.sw
contract;

abi ContractA {
    fn receive(field_1: bool, field_2: u64) -> u64;
}

impl ContractA for Contract {
    fn receive(field_1: bool, field_2: u64) -> u64 {
        assert(field_1 == true);
        assert(field_2 > 0);
        45
    }
}
```

```rs
// ./contract_b.sw
contract;

use contract_a::ContractA;

abi ContractB {
    fn make_call();
}

const contract_id = 0x79fa8779bed2f36c3581d01c79df8da45eee09fac1fd76a5a656e16326317ef0;

impl ContractB for Contract {
    fn make_call() {
      let x = abi(ContractA, contract_id);
      let return_value = x.receive(true, 3); // will be 45
    }
}
```

## Advanced Calls

Calls forward gas and may push Native Assets into contracts as well.

Here is an example of how to specify the `gas`, [Native `asset_id`](./native_assets.md) and `amount` to forward:

```rs
script;

abi MyContract {
    fn foo(field_1: bool, field_2: u64);
}

fn main() {
    let x = abi(MyContract, 0x79fa8779bed2f36c3581d01c79df8da45eee09fac1fd76a5a656e16326317ef0);
    let native_asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    x.foo {
        gas: 5000, asset_id: asset_id, amount: 5000
    }
    (true, 3);
}
```

## Handling Re-entrancy

A common attack vector for smart contracts is [re-entrancy](https://quantstamp.com/blog/what-is-a-re-entrancy-attack).

This is no different in Fuel and Sway contracts.

Luckily for Sway developers, we provide a stateless re-entracy gaurd in our standard library:

```rs
contract;

use std::reentrancy::{reentrancy_guard};

abi MyContract {
    fn some_method();
}

impl ContractB for Contract {
    fn some_method() {
        reentrancy_guard();
        // do something
    }
}
```

## Differences from Ethereum

While Fuel does share some similar conceptual call paradigms to Ethereum (i.e. gas forwarding and data). 

It differs in *two* key ways:

1) [**Native Assets**](./native_assets.md): FuelVM calls can forward any native asset not just Ether.

2) [**No Data Serialization**](https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/main.md#vm-initialization): Fuel calls **do not** need to serialize data into ABI format, instead they simply pass pointers.

This is because Fuel has a shared global memory context which all call frames can read from and so calling contracts only requires pointers to be passed, and no re-serialization of data is ncessarly.
