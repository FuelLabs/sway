# Deploy and call a Sway contract with the fuel-core CLI

## 6. Spin up a fuel node

In order to deploy the contract, let's spin up a local fuel node:

```console
fuel-core --db-type in-memory
```

## 7. Deploy `wallet_contract` on your local fuel node

To deploy `wallet_contract` on your local fuel node, run

```console
forc deploy
```

from the root of the `wallet_contract` directory.

This should produce some output in stdout that looks like this:

```console
  Compiled library "lib-std" with 7 warnings.
  Compiled library "wallet_lib".
  Compiled script "wallet_contract".
  Bytecode size is 212 bytes.
Contract id: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38
Logs:
HexString256(HexFormatted(0xd9240bc439834bc6afc3f334abf285b3b733560b63d7ce1eb53afa8981984af7))
```

Note the contract id -- you'll need it in the next step.

## 8. Write a Sway script to call a Sway smart contract

Now that we have deployed our wallet contract, we need to actually _call_ our contract. We can do this by calling the contract from a script.

Let's navigate to the `wallet_script` repo created in step 2.

First, you need to link the `wallet_lib` library. Open up the `Forc.toml` in the root of the repo. It should look something like this:

```toml
[project]
authors = ["Yiren Lu"]
entry = "main.sw"
license = "Apache-2.0"
name = "wallet_script"

[dependencies]
core = { git = "http://github.com/FuelLabs/sway-lib-core" }
std = { git = "http://github.com/FuelLabs/sway-lib-std" }
```

Link the `wallet_lib` repo by adding the following line to the bottom of the file:

````toml
wallet_lib = {path = "../wallet_lib"}

Next, open up the `main.sw` file in `src`. Copy and paste the following code:

```sway
script;

use wallet_abi::Wallet;
use wallet_abi::SendFundsRequest;
use std::constants::ETH_ID;

fn main() {
    let caller = abi(Wallet, contract_address);
    let req = SendFundsRequest {
        amount_to_send: 200,
        recipient_address: 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b,
    };
    caller.send_funds(10000, 0, ETH_ID, req);
}
````

Replace the `$contract_address` with the contract id you noted in step 8.

The main new concept is the _abi cast_: `abi(AbiName, contract_address)`. This returns a `ContractCaller` type which can be used to call contracts. The methods of the ABI become the methods available on this contract caller: `send_funds` and `receive_funds`. We then construct the request format, `SendFundsRequest`, and directly call the contract ABI method as if it was just a regular method.

## 9. Build `wallet_script`

To build `wallet_script`, run

```console
forc build
```

from the root of the `wallet_script` directory.

## 10. Run the `wallet_script` against the local fuel node

To run the script now against the local fuel node, run

```console
forc run --contract <contract-id>
```

from the root of the `wallet_script` directory.

Note that we are passing in the `wallet_script` contract-id as a flag. You will need to pass in the contract id of every contract that this script will be interacting with.

If the script is successfully run, it will output something that looks like:

```console
  Compiled library "lib-std" with 7 warnings.
  Compiled library "wallet_lib".
  Compiled script "wallet_script".
  Bytecode size is 272 bytes.
[Call { id: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38, to: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38, amount: 0, color: 0x0000000000000000000000000000000000000000000000000000000000000000, gas: 10000, a: 1506869579, b: 968, pc: 1656, is: 1656 }, Return { id: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38, val: 0, pc: 1764, is: 1656 }, Return { id: 0x0000000000000000000000000000000000000000000000000000000000000000, val: 0, pc: 584, is: 472 }, ScriptResult { result: InstructionResult { reason: RESERV00, instruction: Instruction { op: 0, ra: 0, rb: 0, rc: 0, rd: 0, imm06: 0, imm12: 0, imm18: 0, imm24: 0 } }, gas_used: 998 }]
```

It returns a `Call` object and a `ScriptResult` object.
