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

Now we are ready to build the thing.

```bash
forc build
```

After this the resulting binary should be located at `out/debug/dbg_example.bin`. Because we are interested in the resulting bytecode, we can read that with:

```bash
forc parse-bytecode out/debug/dbg_example.bin
```

Which should give us something like

```text
  half-word   byte   op                   raw           notes
          0   0      JI(4)                90 00 00 04   jump to byte 16
          1   4      NOOP                 47 00 00 00
          2   8      Undefined            00 00 00 00   data section offset lo (0)
          3   12     Undefined            00 00 00 cc   data section offset hi (204)
          4   16     LW(63, 12, 1)        5d fc c0 01
          5   20     ADD(63, 63, 12)      10 ff f3 00
          6   24     MOVE(20, 5)          1a 50 50 00
          7   28     CFEI(24)             91 00 00 18
          8   32     ADDI(16, 20, 8)      50 41 40 08
          9   36     LW(16, 63, 0)        5d 43 f0 00
         10   40     SW(20, 16, 1)        5f 51 00 01
         11   44     ADDI(16, 20, 0)      50 41 40 00
         12   48     LW(16, 63, 1)        5d 43 f0 01
         13   52     SW(20, 16, 0)        5f 51 00 00
         14   56     ADDI(16, 20, 0)      50 41 40 00
         15   60     LW(17, 20, 0)        5d 45 40 00
         16   64     LW(16, 63, 2)        5d 43 f0 02
         17   68     LT(16, 17, 16)       16 41 14 00
         18   72     JNZI(16, 20)         73 40 00 14   conditionally jump to byte 80
         19   76     JI(34)               90 00 00 22   jump to byte 136
         20   80     ADDI(16, 20, 0)      50 41 40 00
         21   84     ADDI(16, 20, 0)      50 41 40 00
         22   88     LW(17, 20, 0)        5d 45 40 00
         23   92     LW(16, 63, 0)        5d 43 f0 00
         24   96     ADD(16, 16, 17)      10 41 04 40
         25   100    SW(20, 16, 0)        5f 51 00 00
         26   104    ADDI(16, 20, 8)      50 41 40 08
         27   108    ADDI(16, 20, 8)      50 41 40 08
         28   112    LW(17, 20, 1)        5d 45 40 01
         29   116    ADDI(16, 20, 0)      50 41 40 00
         30   120    LW(16, 20, 0)        5d 41 40 00
         31   124    MUL(16, 17, 16)      1b 41 14 00
         32   128    SW(20, 16, 1)        5f 51 00 01
         33   132    JI(14)               90 00 00 0e   jump to byte 56
         34   136    ADDI(16, 20, 8)      50 41 40 08
         35   140    LW(16, 20, 1)        5d 41 40 01
         36   144    JI(37)               90 00 00 25   jump to byte 148
         37   148    LW(17, 63, 3)        5d 47 f0 03
         38   152    EQ(17, 17, 0)        13 45 10 00
         39   156    JNZI(17, 41)         73 44 00 29   conditionally jump to byte 164
         40   160    JI(43)               90 00 00 2b   jump to byte 172
         41   164    LOG(16, 0, 0, 0)     33 40 00 00
         42   168    JI(49)               90 00 00 31   jump to byte 196
         43   172    LW(19, 63, 4)        5d 4f f0 04
         44   176    ADDI(17, 20, 16)     50 45 40 10
         45   180    SW(20, 19, 2)        5f 51 30 02
         46   184    ADDI(17, 20, 16)     50 45 40 10
         47   188    LW(17, 20, 2)        5d 45 40 02
         48   192    LOGD(0, 0, 16, 17)   34 00 04 11
         49   196    LW(16, 63, 1)        5d 43 f0 01
         50   200    RET(0)               24 00 00 00
         51   204    Undefined            00 00 00 00
         52   208    Undefined            00 00 00 01
         53   212    Undefined            00 00 00 00
         54   216    Undefined            00 00 00 00
         55   220    Undefined            00 00 00 00
         56   224    Undefined            00 00 00 05
         57   228    Undefined            00 00 00 00
         58   232    Undefined            00 00 00 00
         59   236    Undefined            00 00 00 00
         60   240    Undefined            00 00 00 08
```

We can recognize the `while` loop by the conditional jumps `JNZI(..)`. The condition just before the first jump can be identified by `LT(..)` instruction (for `<`). Some notable instructions that are generated only once in our code include `MUL(..)` for multiplication and `LOG(.., 0, 0, 0)` from the `log` function.

## Setting up the debugging

We can start up the debug infrastructure. On a new terminal session run `fuel-core --db-type in-memory`; we need to have that running because it actually executes the program. Now we can fire up the debugger itself: `fuel-debugger`. Now
if everything is set up correctly, you shoould see the debugger prompt (`>> `). You can use `help` command to list available commands.

Now we would like to inspect the program while it's running. To do this, we first need to send the script to the executor, i.e. `fuel-core`. To do so, we need a *transaction specification*, `tx.json`. It looks something like this:

```json
{
    "Script": {
        "byte_price": 0,
        "gas_price": 0,
        "gas_limit": 1000000,
        "maturity": 0,
        "script": [],
        "script_data": [],
        "inputs": [],
        "outputs": [],
        "witnesses": [],
        "receipts_root": "0000000000000000000000000000000000000000000000000000000000000000"
    }
}
```

However, the key `script` in should contain the actual bytecode to execute, i.e. the contents of `out/debug/dbg_example.bin` as a JSON array. The following command can be used to generate it:

```bash
python3 -c 'print(list(open("out/debug/dbg_example.bin", "rb").read()))'
```

So now we replace the script array with the result, and save it as `tx.json`. It looks something like this:

```json
{
    "Script": {
        "byte_price": 0,
        "gas_price": 0,
        "gas_limit": 1000000,
        "maturity": 0,
        "script": [144, 0, 0, 4, 71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 204, 93, 252, 192, 1, 16, 255, 243, 0, 26, 80, 80, 0, 145, 0, 0, 24, 80, 65, 64, 8, 93, 67, 240, 0, 95, 81, 0, 1, 80, 65, 64, 0, 93, 67, 240, 1, 95, 81, 0, 0, 80, 65, 64, 0, 93, 69, 64, 0, 93, 67, 240, 2, 22, 65, 20, 0, 115, 64, 0, 20, 144, 0, 0, 34, 80, 65, 64, 0, 80, 65, 64, 0, 93, 69, 64, 0, 93, 67, 240, 0, 16, 65, 4, 64, 95, 81, 0, 0, 80, 65, 64, 8, 80, 65, 64, 8, 93, 69, 64, 1, 80, 65, 64, 0, 93, 65, 64, 0, 27, 65, 20, 0, 95, 81, 0, 1, 144, 0, 0, 14, 80, 65, 64, 8, 93, 65, 64, 1, 144, 0, 0, 37, 93, 71, 240, 3, 19, 69, 16, 0, 115, 68, 0, 41, 144, 0, 0, 43, 51, 64, 0, 0, 144, 0, 0, 49, 93, 79, 240, 4, 80, 69, 64, 16, 95, 81, 48, 2, 80, 69, 64, 16, 93, 69, 64, 2, 52, 0, 4, 17, 93, 67, 240, 1, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8],
        "script_data": [],
        "inputs": [],
        "outputs": [],
        "witnesses": [],
        "receipts_root": "0000000000000000000000000000000000000000000000000000000000000000"
    }
}
```

## Using the debugger

Now we can actually execute the script:

```text
>> start_tx tx.json

Receipt: Log { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 120, rb: 0, rc: 0, rd: 0, pc: 10516, is: 10352 }
Receipt: Return { id: 0000000000000000000000000000000000000000000000000000000000000000, val: 0, pc: 10552, is: 10352 }
Receipt: ScriptResult { result: Success, gas_used: 1302 }
Terminated
```

Looking at the first output line, we can see that it logged `ra: 120` which is the correct return value for `factorial(5)`. It also tells us that the exection terminated without hitting any breakpoints. That's unsurprising, because we haven't set up any. We can do so with `breakpoint` command:

```text
>> breakpoint 0

>> start_tx tx.json

Receipt: ScriptResult { result: Success, gas_used: 0 }
Stopped on breakpoint at address 0 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

```

Now we have stopped execution at the breakpoint on entry (address `0`). We can now inspect the initial state of the VM.

```
>> register ggas

reg[0x9] = 1000000  # ggas

>> memory 0x10 0x8

 000010: dc dd a0 56 c2 20 1c 6b
```

However, that's not too interesting either, so let's just execute until the end, and then reset the vm to remove the breakpoints.

```
>> continue

Receipt: Log { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 120, rb: 0, rc: 0, rd: 0, pc: 10516, is: 10352 }
Receipt: Return { id: 0000000000000000000000000000000000000000000000000000000000000000, val: 0, pc: 10552, is: 10352 }
Terminated

>> reset

```

Next, we will setup a breakpoint to check the state on each iteration of the `while` loop. For instance, if we'd like to see what numbers get multiplied together, we could set up a breakpoint before the operation. The bytecode has only a single `MUL` instruction:

```
  half-word   byte   op                   raw           notes
         31   124    MUL(16, 17, 16)      1b 41 14 00
```

We can set a breakpoint on its address, at halfword-offset `31`.

```
>>> breakpoint 31

>> start_tx tx.json

Receipt: ScriptResult { result: Success, gas_used: 287 }
Stopped on breakpoint at address 124 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

```

Now we can inspect the inputs tu multiply. Looking at [the specification](https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/instruction_set.md#mul-multiply) tells us that the instruction `MUL(16, 17, 16)` means `reg[16] = reg[17] * reg[16]`. So inpecting the inputs tells us that

```
>> r 16 17

reg[0x10] = 1        # reg16
reg[0x11] = 1        # reg17
```

So on the first round the numbers are `1` and `1`, so we can continue to the next iteration:

```
>> c

Stopped on breakpoint at address 124 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 16 17

reg[0x10] = 2        # reg16
reg[0x11] = 1        # reg17

```

And the next one:

```
>> c

Stopped on breakpoint at address 124 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 16 17

reg[0x10] = 3        # reg16
reg[0x11] = 2        # reg17

```

And fourth one:

```
>> c

Stopped on breakpoint at address 124 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 16 17

reg[0x10] = 4        # reg16
reg[0x11] = 6        # reg17

```

And round 5:

```
>> c

Stopped on breakpoint at address 124 of contract 0x0000000000000000000000000000000000000000000000000000000000000000

>> r 16 17

reg[0x10] = 5        # reg16
reg[0x11] = 24       # reg17

```

At this point we can look at the values

16 | 17
---|----
1  | 1
2  | 1
3  | 2
4  | 6
5  | 24

From this we can clearly see that the left side, register `16` is the `counter` variable, and register `17` is `result`. Now the counter equals the given factorial function argument `5`, and the loop terminates. So when we continue, the program finishes without encountering any more breakpoints:

```
>> c

Receipt: Log { id: 0000000000000000000000000000000000000000000000000000000000000000, ra: 120, rb: 0, rc: 0, rd: 0, pc: 10516, is: 10352 }
Receipt: Return { id: 0000000000000000000000000000000000000000000000000000000000000000, val: 0, pc: 10552, is: 10352 }
Terminated
```
