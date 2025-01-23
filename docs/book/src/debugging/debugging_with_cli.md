# Debugging with CLI

The `forc debug` CLI enables debugging a live transaction on a running Fuel Client node.

## An example project

First, we need a project to debug, so create a new project using

```bash
forc new --script dbg_example && cd dbg_example
```

And then add some content to `src/main.sw`, for example:

```sway
script;

use std::logging::log;

fn factorial(n: u64) -> u64 {
    let mut result = 1;
    let mut counter = 0;
    while counter < n {
        counter = counter + 1;
        result = result * counter;
    }
    return result;
}

fn main() {
    log::<u64>(factorial(5)); // 120
}
```

## Building and bytecode output

Now we are ready to build the project.

```bash
forc build
```

After this the resulting binary should be located at `out/debug/dbg_example.bin`. Because we are interested in the resulting bytecode, we can read that with:

```bash
forc parse-bytecode out/debug/dbg_example.bin
```

We can recognize the main loop by observing the control flow. Looking around halfword 58-60, we can see:

```text
  half-word   byte    op                                                 raw
          58   232    MOVI { dst: 0x11, val: 5 }                         72 44 00 05                                 
          59   236    LT { dst: 0x10, lhs: 0x10, rhs: 0x11 }             16 41 04 40                                 
          60   240    JNZF { cond_nz: 0x10, dynamic: 0x0, fixed: 81 }    76 40 00 51
```

Here we can see our `factorial(5)` being set up with `MOVI` setting the value 5, followed by the `LT` comparison and conditional jump `JNZF`. The multiplication for our factorial happens at halfword 147 with `MUL { dst: 0x10, lhs: 0x10, rhs: 0x11 }`. Finally, we can spot our log statement at halfword 139 with the `LOGD` instruction.

## Setting up the debugging

We can start up the debug infrastructure. On a new terminal session run `fuel-core run --db-type in-memory --debug`; we need to have that running because it actually executes the program. Now we can fire up the debugger itself: `forc-debug`. Now if everything is set up correctly, you should see the debugger prompt (`>>`). You can use `help` command to list available commands.

The debugger supports tab completion to help you discover files in your current working directory (and its subdirectories):

- Type `tx` and press tab to recursively search for valid transaction JSON files
- After selecting a transaction file, press tab again to search for ABI files
- You can keep pressing tab to cycle through the found files
- Of course, you can also manually type the full path to any transaction or ABI file, they don't have to be in your current directory

Now we would like to inspect the program while it's running. To do this, we first need to send the script to the executor, i.e. `fuel-core`. To do so, we need a *transaction specification*, `tx.json`. It looks something like this:

```json
{
  "Script": {
    "body": {
      "script_gas_limit": 1000000,
      "script": [
        26, 240, 48, 0, 116, 0, 0, 2, 0, 0, 0, 0, 0, 0, 3, 96, 93, 255, 192, 1, 16, 255, 255, 0, 26, 236, 80, 0, 145, 0, 0, 184, 80, 67, 176, 80, 32, 248, 51, 0, 88, 251, 224, 2, 80, 251, 224, 4, 116, 0, 0, 37, 80, 71, 176, 40, 26, 233, 16, 0, 32, 248, 51, 0, 88, 251, 224, 2, 80, 251, 224, 4, 116, 0, 0, 136, 26, 71, 208, 0, 114, 72, 0, 24, 40, 237, 20, 128, 80, 79, 176, 120, 114, 68, 0, 24, 40, 79, 180, 64, 80, 71, 176, 160, 114, 72, 0, 24, 40, 69, 52, 128, 80, 71, 176, 96, 114, 72, 0, 24, 40, 69, 52, 128, 80, 75, 176, 64, 26, 233, 16, 0, 26, 229, 32, 0, 32, 248, 51, 0, 88, 251, 224, 2, 80, 251, 224, 4, 116, 0, 0, 144, 26, 71, 208, 0, 80, 75, 176, 24, 114, 76, 0, 16, 40, 73, 20, 192, 80, 71, 176, 144, 114, 76, 0, 16, 40, 69, 36, 192, 114, 72, 0, 16, 40, 65, 20, 128, 93, 69, 0, 1, 93, 65, 0, 0, 37, 65, 16, 0, 149, 0, 0, 63, 150, 8, 0, 0, 26, 236, 80, 0, 145, 0, 1, 88, 26, 87, 224, 0, 95, 236, 16, 42, 95, 236, 0, 41, 93, 67, 176, 41, 114, 68, 0, 5, 22, 65, 4, 64, 118, 64, 0, 81, 93, 67, 176, 42, 80, 71, 176, 200, 26, 233, 16, 0, 32, 248, 51, 0, 88, 251, 224, 2, 80, 251, 224, 4, 116, 0, 0, 87, 26, 71, 208, 0, 114, 72, 0, 24, 40, 237, 20, 128, 80, 71, 176, 160, 114, 72, 0, 24, 40, 71, 180, 128, 80, 75, 176, 24, 114, 76, 0, 24, 40, 73, 20, 192, 80, 71, 176, 88, 114, 76, 0, 24, 40, 69, 36, 192, 93, 83, 176, 11, 93, 79, 176, 12, 93, 71, 176, 13, 114, 72, 0, 8, 16, 73, 20, 128, 21, 73, 36, 192, 118, 72, 0, 1, 116, 0, 0, 7, 114, 72, 0, 2, 27, 73, 52, 128, 114, 76, 0, 8, 16, 77, 36, 192, 38, 76, 0, 0, 40, 29, 68, 64, 26, 80, 112, 0, 16, 73, 68, 64, 95, 73, 0, 0, 114, 64, 0, 8, 16, 65, 20, 0, 80, 71, 176, 112, 95, 237, 64, 14, 95, 237, 48, 15, 95, 237, 0, 16, 80, 67, 176, 48, 114, 72, 0, 24, 40, 65, 20, 128, 80, 71, 176, 136, 114, 72, 0, 24, 40, 69, 4, 128, 80, 67, 177, 8, 114, 72, 0, 24, 40, 65, 20, 128, 80, 71, 177, 48, 114, 72, 0, 24, 40, 69, 4, 128, 80, 67, 177, 48, 80, 71, 176, 240, 114, 72, 0, 24, 40, 69, 4, 128, 80, 67, 176, 224, 26, 233, 16, 0, 26, 229, 0, 0, 32, 248, 51, 0, 88, 251, 224, 2, 80, 251, 224, 4, 116, 0, 0, 56, 26, 67, 208, 0, 80, 71, 176, 72, 114, 72, 0, 16, 40, 69, 4, 128, 80, 67, 177, 32, 114, 72, 0, 16, 40, 65, 20, 128, 80, 71, 176, 184, 114, 72, 0, 16, 40, 69, 4, 128, 93, 67, 240, 0, 93, 71, 176, 23, 93, 75, 176, 24, 52, 1, 4, 82, 26, 244, 0, 0, 116, 0, 0, 8, 93, 67, 176, 41, 16, 65, 0, 64, 95, 237, 0, 41, 93, 67, 176, 42, 93, 71, 176, 41, 27, 65, 4, 64, 95, 237, 0, 42, 117, 0, 0, 91, 146, 0, 1, 88, 26, 249, 80, 0, 152, 8, 0, 0, 151, 0, 0, 63, 74, 248, 0, 0, 149, 0, 0, 15, 150, 8, 0, 0, 26, 236, 80, 0, 145, 0, 0, 72, 26, 67, 160, 0, 26, 71, 224, 0, 114, 72, 4, 0, 38, 72, 0, 0, 26, 72, 112, 0, 80, 79, 176, 24, 95, 237, 32, 3, 114, 72, 4, 0, 95, 237, 32, 4, 95, 236, 0, 5, 114, 72, 0, 24, 40, 237, 52, 128, 80, 75, 176, 48, 114, 76, 0, 24, 40, 75, 180, 192, 114, 76, 0, 24, 40, 65, 36, 192, 26, 245, 0, 0, 146, 0, 0, 72, 26, 249, 16, 0, 152, 8, 0, 0, 151, 0, 0, 15, 74, 248, 0, 0, 149, 0, 0, 63, 150, 8, 0, 0, 26, 236, 80, 0, 145, 0, 0, 104, 26, 67, 160, 0, 26, 71, 144, 0, 26, 75, 224, 0, 80, 79, 176, 80, 114, 80, 0, 24, 40, 77, 5, 0, 114, 64, 0, 24, 40, 237, 52, 0, 80, 67, 176, 40, 114, 76, 0, 24, 40, 67, 180, 192, 93, 79, 176, 5, 80, 65, 0, 16, 80, 83, 176, 64, 95, 237, 48, 8, 80, 77, 64, 8, 114, 84, 0, 8, 40, 77, 5, 64, 80, 67, 176, 24, 114, 76, 0, 16, 40, 65, 68, 192, 114, 76, 0, 16, 40, 69, 4, 192, 26, 245, 16, 0, 146, 0, 0, 104, 26, 249, 32, 0, 152, 8, 0, 0, 151, 0, 0, 63, 74, 248, 0, 0, 71, 0, 0, 0, 21, 6, 230, 244, 76, 29, 98, 145
      ],
      "script_data": [],
      "receipts_root": "0000000000000000000000000000000000000000000000000000000000000000"
    },
    "policies": {
      "bits": "MaxFee",
      "values": [0, 0, 0, 0]
    },
    "inputs": [
      {
        "CoinSigned": {
          "utxo_id": {
            "tx_id": "c49d65de61cf04588a764b557d25cc6c6b4bc0d7429227e2a21e61c213b3a3e2",
            "output_index": 18
          },
          "owner": "f1e92c42b90934aa6372e30bc568a326f6e66a1a0288595e6e3fbd392a4f3e6e",
          "amount": 10599410012256088000,
          "asset_id": "2cafad611543e0265d89f1c2b60d9ebf5d56ad7e23d9827d6b522fd4d6e44bc3",
          "tx_pointer": {
            "block_height": 0,
            "tx_index": 0
          },
          "witness_index": 0,
          "maturity": 0,
          "predicate_gas_used": null,
          "predicate": null,
          "predicate_data": null
        }
      }
    ],
    "outputs": [],
    "witnesses": [
      {
        "data": [156, 254, 34, 102, 65, 96, 133, 170, 254, 105, 147, 35, 196, 199, 179, 133, 132, 240, 208, 149, 11, 46, 30, 96, 44, 91, 121, 195, 145, 184, 159, 235, 117, 82, 135, 41, 84, 154, 102, 61, 61, 16, 99, 123, 58, 173, 75, 226, 219, 139, 62, 33, 41, 176, 16, 18, 132, 178, 8, 125, 130, 169, 32, 108]
      }
    ]
  }
}
```

However, the key `script` should contain the actual bytecode to execute, i.e. the contents of `out/debug/dbg_example.bin` as a JSON array. The following command can be used to generate it:

```bash
python3 -c 'print(list(open("out/debug/dbg_example.bin", "rb").read()))'
```

So now we replace the script array with the result, and save it as `tx.json`.

## Using the debugger

Now we can actually execute the script with an ABI to decode the log values:

```text
>> start_tx tx.json out/debug/dbg_example-abi.json

Receipt: LogData { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 0, rb: 1515152261580153489, ptr: 67107840, len: 8, digest: d2b80ebb9ce633ad49a9ccfcc58ac7ad33a9ab4741529ae4247a3b07e8fa1c74, pc: 10924, is: 10368, data: Some(0000000000000078) }
Decoded log value: 120, from contract: 0000000000000000000000000000000000000000000000000000000000000000
Receipt: ReturnData { id: 0000000000000000000000000000000000000000000000000000000000000000, ptr: 67106816, len: 0, digest: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855, pc: 10564, is: 10368, data: Some() }
Receipt: ScriptResult { result: Success, gas_used: 1273 }
Terminated
```

Looking at the output, we can see our `factorial(5)` result as the decoded log value of 120. The ABI has helped us decode the raw bytes `(0000000000000078)` into a meaningful value. It also tells us that the execution terminated without hitting any breakpoints. That's unsurprising, because we haven't set up any. We can do so with `breakpoint` command:

```text
>> breakpoint 0

>> start_tx tx.json out/debug/dbg_example-abi.json

Receipt: ScriptResult { result: Success, gas_used: 0 }
Stopped on breakpoint at address 0 of contract 0x0000000000000000000000000000000000000000000000000000000000000000
```

Now we have stopped execution at the breakpoint on entry (address `0`). We can now inspect the initial state of the VM.

```text
>> register ggas

reg[0x9] = 1000000  # ggas

>> memory 0x10 0x8

 000010: db f3 63 c9 1c 7f ec 95
```

However, that's not too interesting either, so let's just execute until the end, and then reset the VM to remove the breakpoints.

```text
>> continue

Receipt: LogData { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 0, rb: 1515152261580153489, ptr: 67107840, len: 8, digest: d2b80ebb9ce633ad49a9ccfcc58ac7ad33a9ab4741529ae4247a3b07e8fa1c74, pc: 10924, is: 10368, data: Some(0000000000000078) }
Decoded log value: 120, from contract: 0000000000000000000000000000000000000000000000000000000000000000
Receipt: ReturnData { id: 0000000000000000000000000000000000000000000000000000000000000000, ptr: 67106816, len: 0, digest: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855, pc: 10564, is: 10368, data: Some() }
Terminated

>> reset
```

Next, we will setup a breakpoint to check the state on each iteration of the `while` loop. For instance, if we'd like to see what numbers get multiplied together, we could set up a breakpoint before the operation. Looking at our bytecode we can see the main multiplication for our factorial happens at:

```text
  half-word   byte   op                                        raw
        147   588    MUL { dst: 0x10, lhs: 0x10, rhs: 0x11 }   1b 41 04 40
```

We can set a breakpoint on its address, at halfword-offset `147`.

```text
>>> breakpoint 147

>> start_tx tx.json out/debug/dbg_example-abi.json

Receipt: ScriptResult { result: Success, gas_used: 82 }
Stopped on breakpoint at address 588 of contract 0x0000000000000000000000000000000000000000000000000000000000000000
```

Now we can inspect the inputs to multiply. Looking at [the specification](https://github.com/FuelLabs/fuel-specs/blob/master/src/fuel-vm/instruction-set.md#mul-multiply) tells us that the instruction `MUL { dst: 0x10, lhs: 0x10, rhs: 0x11 }` means `reg[0x10] = reg[0x10] * reg[0x11]`. So inspecting the inputs:

```text
>> r 0x10 0x11
reg[0x10] = 1        # reg16
reg[0x11] = 1        # reg17
```

So on the first round the numbers are 1 and 1, so we can continue to the next iteration with the `c` command:

```text
>> c
Stopped on breakpoint at address 588 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 0x10 0x11
reg[0x10] = 1        # reg16
reg[0x11] = 2        # reg17
```

And the next one:

```text
>> c
Stopped on breakpoint at address 588 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 0x10 0x11
reg[0x10] = 2        # reg16
reg[0x11] = 3        # reg17
```

And fourth one:

```text
>> c
Stopped on breakpoint at address 588 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 0x10 0x11
reg[0x10] = 6        # reg16
reg[0x11] = 4        # reg17
```

And round 5:

```text
>> c
Stopped on breakpoint at address 588 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 0x10 0x11
reg[0x10] = 24       # reg16
reg[0x11] = 5        # reg17
```

At this point we can look at the values

0x10 | 0x11
-----|------
1    | 1
1    | 2
2    | 3
6    | 4
24   | 5

From this we can clearly see that the left side, register `0x10` is the `result` variable which accumulates the factorial calculation (1, 1, 2, 6, 24), and register `0x11` is the `counter` which increments from 1 to 5. Now the counter equals the given factorial function argument `5`, and the loop terminates. So when we continue, the program finishes without encountering any more breakpoints:

```text
>> c

Receipt: LogData { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 0, rb: 1515152261580153489, ptr: 67107840, len: 8, digest: d2b80ebb9ce633ad49a9ccfcc58ac7ad33a9ab4741529ae4247a3b07e8fa1c74, pc: 10924, is: 10368, data: Some(0000000000000078) }
Decoded log value: 120, from contract: 0000000000000000000000000000000000000000000000000000000000000000
Receipt: ReturnData { id: 0000000000000000000000000000000000000000000000000000000000000000, ptr: 67106816, len: 0, digest: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855, pc: 10564, is: 10368, data: Some() }
Terminated
```
