# Debugging with IDE

The `forc debug` plugin also enables line-by-line debugging of Sway unit tests in VSCode.

## Installation

1. Install the Sway VSCode extension from the [marketplace](https://marketplace.visualstudio.com/items?itemName=FuelLabs.sway-vscode-plugin).
2. Ensure you have the forc-debug binary installed. `which forc-debug`.
It can be installed with `fuelup component add forc-debug`.
3. Create a `.vscode/launch.json` file with the following contents:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
        "type": "sway",
        "request": "launch",
        "name": "Debug Sway",
        "program": "${file}"
    }]
}
```

## An example project

Given this example contract:

```sway
contract;

abi CallerContract {
    fn test_false() -> bool;
}

impl CallerContract for Contract {
    fn test_false() -> bool {
        false
    }
}

abi CalleeContract {
    fn test_true() -> bool;
}

#[test]
fn test_multi_contract_calls() {
    let caller = abi(CallerContract, CONTRACT_ID);
    let callee = abi(CalleeContract, callee::CONTRACT_ID);

    let should_be_false = caller.test_false();
    let should_be_true = callee.test_true();
    assert(!should_be_false);
    assert(should_be_true);
}
```

Within the sway file open in VSCode, you can set breakpoints on lines within the test or functions that it calls, and click Run -> Start Debugging to begin debugging the unit test.

This will build the sway project and run it in debug mode. The debugger will stop the VM execution when a breakpoint is hit.

The debug panel will show VM registers under the Variables tab, as well as the current VM opcode where execution is suspended. You can continue execution, or use the Step Over function to step forward, instruction by instruction.
