# Calling Contracts

End users and smart contracts typically need to communicate with each other.

The FuelVM achieves end user and inter contract communication primarily with the [`call`](https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/opcodes.md#call-call-contract) opcode.

Sway provides a nice way to manage calls with its `abi` system.

Calls can be made both between contracts and from scripts to contracts.

## Example

Here is an example of a *contract calling another contract* in Sway:

```sway
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

```sway
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

All calls forward a gas stipend, and may additionally forward one native asset with the call.

Here is an example of how to specify the amount of gas (`gas`), the asset ID of the native asset (`asset_id`), and the amount of the native asset (`amount`) to forward:

```sway
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

A common attack vector for smart contracts is [re-entrancy](https://docs.soliditylang.org/en/v0.8.4/security-considerations.html#re-entrancy). Similar to the Ethereum Virtual Machine, the FuelVM allows for re-entrancy.

A _stateless_ re-entrancy guard in the Sway standard library.

Note, this gaurd method will panic if re-entrancy is detected.

```sway
contract;

use std::reentrancy::reentrancy_guard;

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

While Fuel does share some similar conceptual call paradigms to Ethereum (i.e. gas forwarding and data), it differs in *two* key ways:

1) [**Native Assets**](./native_assets.md): FuelVM calls can forward any native asset not just Ether.

2) [**No Data Serialization**](https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/main.md#vm-initialization): Fuel calls **do not** need to serialize data into ABI format, instead they simply pass pointers.

This is because Fuel has a shared global memory context which all call frames can read from and so calling contracts only requires pointers to be passed, and no re-serialization of data is necessarily.
