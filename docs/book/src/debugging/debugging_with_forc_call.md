# Debugging with Forc Call

The `forc call` command includes interactive debugging capabilities through the `--debug` flag, enabling developers to debug contract function calls step-by-step after transaction execution.

## Overview

When you add the `--debug` flag to any `forc call` command, it will:

1. Execute the contract function call as normal
2. Automatically launch an interactive debugging session
3. Allow you to step through the execution, inspect values, and set breakpoints
4. Provide full access to the `forc-debug` interface

This integration seamlessly combines contract interaction with debugging, making it easy to understand what happens during contract execution.

## Basic Usage

Simply add the `--debug` flag to any existing `forc call` command:

```bash
forc call <CONTRACT_ID> \
    --abi <ABI_PATH> \
    <FUNCTION_NAME> [ARGS...] \
    --debug
```

## Example: Debugging a Contract Call

Let's say you have a contract with a function that performs some calculations:

```sway
contract;

abi Calculator {
    fn factorial(n: u64) -> u64;
}

impl Calculator for Contract {
    fn factorial(n: u64) -> u64 {
        let mut result = 1;
        let mut counter = 0;
        while counter < n {
            counter = counter + 1;
            result = result * counter;
        }
        result
    }
}
```

### Debugging the Factorial Function

```bash
forc call 0x1234567890abcdef1234567890abcdef12345678 \
    --abi ./out/debug/calculator-abi.json \
    factorial 5 \
    --debug
```

This command will:

1. Execute `factorial(5)` on the deployed contract
2. Show the transaction result (return value: 120)
3. Launch the interactive debugger with the transaction data pre-loaded

### Interactive Debugging Session

Once the debugger launches, you'll see the pre-buffered debugger prompt to start the transaction:

```text
Welcome to the Sway Debugger! Type "help" for a list of commands.
>> start_tx /var/folders/xz/5djvk4596k5c1fj2prcd0d880000gn/T/.tmpexrnFE.json /var/folders/xz/5djvk4596k5c1fj2prcd0d880000gn/T/.tmpuaPRtF.json
```

You can now use all the [standard debugging commands](./debugging_with_cli.md#using-the-debugger).
