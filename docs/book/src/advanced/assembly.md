# Inline Assembly in Sway

While many users will never have to touch assembly language while writing sway code, it is a powerful tool that enables many advanced use-cases (ie: optimizations, building libraries, etc).

## ASM Block

In Sway, the way we use assembly inline is to declare an asm block like this:

```sway
asm() {...}
```

Declaring an `asm` block is similar to declaring a function.
We can specify register names to operate on as arguments, we can perform operations within the block, and we can return a value.
Here's an example showing what this might look like:

```sway
pub fn add_1(num: u32) -> u32 {
    asm(r1: num, r2) {
        add r2 r1 one;
        r2: u32
    }
}
```

An `asm` block can only return a single register. If you really need to return more than one value, you can modify a tuple. Here's an example showing how can implement this `(u64, u64)`:

```sway
{{#include ../../../../examples/asm_return_tuple_pointer/src/main.sw}}
```

Note that this is contrived example meant to demonstrate the syntax; there's absolutely no need to use assembly to add integers!

Note that in the above example:

- we initialized the register `r1` with the value of `num`.
- we declared a second register `r2` (you may choose any register names you want).
- we use the `add` opcode to add `one` to the value of `r1` and store it in `r2`.
- `one` is an example of a "reserved register", of which there are 16 in total. Further reading on this is linked below under "Semantics".
- we return `r2` & specify the return type as being u32 (the return type is u64 by default).

An important note is that the `ji` and `jnei` opcodes are not available within an `asm` block. For those looking to introduce control flow to `asm` blocks, it is recommended to surround smaller chunks of `asm` with control flow (`if`, `else`, and `while`).

## Helpful Links

For examples of assembly in action, check out the [Sway standard library](https://github.com/FuelLabs/sway/tree/master/sway-lib-std).

For a complete list of all instructions supported in the FuelVM: [Instructions](https://fuellabs.github.io/fuel-specs/master/vm/instruction_set).

And to learn more about the FuelVM semantics: [Semantics](https://fuellabs.github.io/fuel-specs/master/vm#semantics).
