# Deploy and Call a Sway Contract With the CLI

## Spin Up a Fuel node

In order to deploy the contract, let's spin up a local Fuel node:

```console
Fuel-core --db-type in-memory
```

## Deploy `wallet_contract` To Your Local Fuel Node

To deploy `wallet_contract` on your local Fuel node, run

```console
forc deploy
```

from the root of the `wallet_contract` directory.

This should produce some output in `stdout` that looks like this:

```console
  Compiled library "lib-std" with 7 warnings.
  Compiled library "wallet_lib".
  Compiled script "wallet_contract".
  Bytecode size is 212 bytes.
Contract id: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38
Logs:
HexString256(HexFormatted(0xd9240bc439834bc6afc3f334abf285b3b733560b63d7ce1eb53afa8981984af7))
```

Note the contract ID—you'll need it in the next step.

## Run `wallet_script` Against Your Local Fuel Node

To run the script now against the local Fuel node, run

```console
forc run --contract <contract-id>
```

from the root of the `wallet_script` directory.

Note that we are passing in the `wallet_contract` contract ID as a command-line parameter. You will need to pass in the contract ID of every contract that this script will be interacting with.

If the script is successfully run, it will output something that looks like:

```console
  Compiled library "lib-std" with 7 warnings.
  Compiled library "wallet_lib".
  Compiled script "wallet_script".
  Bytecode size is 272 bytes.
[Call { id: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38, to: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38, amount: 0, color: 0x0000000000000000000000000000000000000000000000000000000000000000, gas: 10000, a: 1506869579, b: 968, pc: 1656, is: 1656 }, Return { id: 0xf4b63e0e09cb72762cec18a6123a9fb5bd501b87141fac5835d80f5162505c38, val: 0, pc: 1764, is: 1656 }, Return { id: 0x0000000000000000000000000000000000000000000000000000000000000000, val: 0, pc: 584, is: 472 }, ScriptResult { result: InstructionResult { reason: RESERV00, instruction: Instruction { op: 0, ra: 0, rb: 0, rc: 0, rd: 0, imm06: 0, imm12: 0, imm18: 0, imm24: 0 } }, gas_used: 998 }]
```

It returns a `Call` receipt and a `ScriptResult` receipt.
