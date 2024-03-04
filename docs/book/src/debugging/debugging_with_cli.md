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

Which should give us something like

```text

  half-word   byte   op                                    raw           notes
          0   0      JI { imm: 4 }                         90 00 00 04   jump to byte 16
          1   4      NOOP                                  47 00 00 00
          2   8      InvalidOpcode                         00 00 00 00   data section offset lo (0)
          3   12     InvalidOpcode                         00 00 00 44   data section offset hi (68)
          4   16     LW { ra: 63, rb: 12, imm: 1 }         5d fc c0 01
          5   20     ADD { ra: 63, rb: 63, rc: 12 }        10 ff f3 00
          6   24     MOVE { ra: 18, rb: 1 }                1a 48 10 00
          7   28     MOVE { ra: 17, rb: 0 }                1a 44 00 00
          8   32     LW { ra: 16, rb: 63, imm: 0 }         5d 43 f0 00
          9   36     LT { ra: 16, rb: 17, rc: 16 }         16 41 14 00
         10   40     JNZI { ra: 16, imm: 13 }              73 40 00 0d   conditionally jump to byte 52
         11   44     LOG { ra: 18, rb: 0, rc: 0, rd: 0 }   33 48 00 00
         12   48     RET { ra: 0 }                         24 00 00 00
         13   52     ADD { ra: 17, rb: 17, rc: 1 }         10 45 10 40
         14   56     MUL { ra: 18, rb: 18, rc: 17 }        1b 49 24 40
         15   60     JI { imm: 8 }                         90 00 00 08   jump to byte 32
         16   64     NOOP                                  47 00 00 00
         17   68     InvalidOpcode                         00 00 00 00
         18   72     InvalidOpcode                         00 00 00 05
```

We can recognize the `while` loop by the conditional jumps `JNZI`. The condition just before the first jump can be identified by `LT` instruction (for `<`). Some notable instructions that are generated only once in our code include `MUL` for multiplication and `LOG {.., 0, 0, 0}` from the `log` function.

## Setting up the debugging

We can start up the debug infrastructure. On a new terminal session run `fuel-core run --db-type in-memory --debug`; we need to have that running because it actually executes the program. Now we can fire up the debugger itself: `forc-debug`. Now
if everything is set up correctly, you should see the debugger prompt (`>>`). You can use `help` command to list available commands.

Now we would like to inspect the program while it's running. To do this, we first need to send the script to the executor, i.e. `fuel-core`. To do so, we need a *transaction specification*, `tx.json`. It looks something like this:

```json
{
    "Script": {
        "script_gas_limit": 1000000,
        "script": [],
        "script_data": [],
        "policies": {
            "bits": "GasPrice",
            "values": [0,0,0,0]
        },
        "inputs": [
            {
                "CoinSigned": {
                    "utxo_id": {
                        "tx_id": "c49d65de61cf04588a764b557d25cc6c6b4bc0d7429227e2a21e61c213b3a3e2",
                        "output_index": 18
                    },
                    "owner": "f1e92c42b90934aa6372e30bc568a326f6e66a1a0288595e6e3fbd392a4f3e6e",
                    "amount": 10599410012256088338,
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
                "data": [
                    156,254,34,102,65,96,133,170,254,105,147,35,196,199,179,133,132,240,208,149,11,46,30,96,44,91,121,195,145,184,159,235,117,82,135,41,84,154,102,61,61,16,99,123,58,173,75,226,219,139,62,33,41,176,16,18,132,178,8,125,130,169,32,108
                ]
            }
        ],
        "receipts_root": "0000000000000000000000000000000000000000000000000000000000000000"
    }
}
```

However, the key `script` should contain the actual bytecode to execute, i.e. the contents of `out/debug/dbg_example.bin` as a JSON array. The following command can be used to generate it:

```bash
python3 -c 'print(list(open("out/debug/dbg_example.bin", "rb").read()))'
```

So now we replace the script array with the result, and save it as `tx.json`.

## Using the debugger

Now we can actually execute the script:

```text
>> start_tx tx.json

Receipt: Log { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 120, rb: 0, rc: 0, rd: 0, pc: 10380, is: 10336 }
Receipt: Return { id: 0000000000000000000000000000000000000000000000000000000000000000, val: 0, pc: 10384, is: 10336 }
Receipt: ScriptResult { result: Success, gas_used: 60 }
Terminated
```

Looking at the first output line, we can see that it logged `ra: 120` which is the correct return value for `factorial(5)`. It also tells us that the execution terminated without hitting any breakpoints. That's unsurprising, because we haven't set up any. We can do so with `breakpoint` command:

```text
>> breakpoint 0

>> start_tx tx.json

Receipt: ScriptResult { result: Success, gas_used: 0 }
Stopped on breakpoint at address 0 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

```

Now we have stopped execution at the breakpoint on entry (address `0`). We can now inspect the initial state of the VM.

```text
>> register ggas

reg[0x9] = 1000000  # ggas

>> memory 0x10 0x8

 000010: e9 5c 58 86 c8 87 26 dd
```

However, that's not too interesting either, so let's just execute until the end, and then reset the VM to remove the breakpoints.

```text
>> continue

Receipt: Log { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 120, rb: 0, rc: 0, rd: 0, pc: 10380, is: 10336 }
Receipt: Return { id: 0000000000000000000000000000000000000000000000000000000000000000, val: 0, pc: 10384, is: 10336 }
Terminated

>> reset

```

Next, we will setup a breakpoint to check the state on each iteration of the `while` loop. For instance, if we'd like to see what numbers get multiplied together, we could set up a breakpoint before the operation. The bytecode has only a single `MUL` instruction:

```text
  half-word   byte   op                                    raw           notes
         14   56     MUL { ra: 18, rb: 18, rc: 17 }        1b 49 24 40
```

We can set a breakpoint on its address, at halfword-offset `14`.

```text
>>> breakpoint 14

>> start_tx tx.json

Receipt: ScriptResult { result: Success, gas_used: 9 }
Stopped on breakpoint at address 56 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

```

Now we can inspect the inputs to multiply. Looking at [the specification](https://github.com/FuelLabs/fuel-specs/blob/master/src/fuel-vm/instruction-set.md#mul-multiply) tells us that the instruction `MUL { ra: 18, rb: 18, rc: 17 }` means `reg[18] = reg[18] * reg[17]`. So inspecting the inputs tells us that

```text
>> r 18 17

reg[0x12] = 1        # reg18
reg[0x11] = 1        # reg17
```

So on the first round the numbers are `1` and `1`, so we can continue to the next iteration:

```text
>> c

Stopped on breakpoint at address 56 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 18 17

reg[0x12] = 1        # reg18
reg[0x11] = 2        # reg17

```

And the next one:

```text
>> c

Stopped on breakpoint at address 56 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 18 17

reg[0x12] = 2        # reg18
reg[0x11] = 3        # reg17
```

And fourth one:

```text
>> c

Stopped on breakpoint at address 56 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 18 17

reg[0x12] = 6        # reg18
reg[0x11] = 4        # reg17

```

And round 5:

```text
>> c

Stopped on breakpoint at address 56 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 18 17

reg[0x12] = 24       # reg18
reg[0x11] = 5        # reg17

```

At this point we can look at the values

17 | 18
---|----
1  | 1
2  | 1
3  | 2
4  | 6
5  | 24

From this we can clearly see that the left side, register `17` is the `counter` variable, and register `18` is `result`. Now the counter equals the given factorial function argument `5`, and the loop terminates. So when we continue, the program finishes without encountering any more breakpoints:

```text
>> c

Receipt: Log { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 120, rb: 0, rc: 0, rd: 0, pc: 10380, is: 10336 }
Receipt: Return { id: 0000000000000000000000000000000000000000000000000000000000000000, val: 0, pc: 10384, is: 10336 }
Terminated
```
