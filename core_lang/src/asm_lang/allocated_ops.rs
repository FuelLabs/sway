//! This module contains abstracted versions of bytecode primitives that the compiler uses to
//! ensure correctness and safety.
//!
//! These ops are different from [VirtualOp]s in that they contain allocated registers, i.e. at
//! most 48 free registers plus reserved registers. These ops can be safely directly converted to
//! bytecode.
//!
//!
//! It is unfortunate that there are copies of our opcodes in multiple places, but this ensures the
//! best type safety. It can be macro'd someday.

use super::virtual_ops::*;
use std::fmt;

/// Represents virtual registers that have yet to be allocated.
/// Note that only the Virtual variant will be allocated, and the Constant variant refers to
/// reserved registers.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum AllocatedRegister {
    Allocated(u8),
    Constant(super::virtual_ops::ConstantRegister),
}

impl fmt::Display for AllocatedRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocatedRegister::Allocated(name) => write!(f, "$r{}", name),
            AllocatedRegister::Constant(name) => {
                write!(f, "{}", name)
            }
        }
    }
}

/// This enum is unfortunately a redundancy of the [fuel_asm::Opcode] and [crate::VirtualOp] enums. This variant, however,
/// allows me to use the compiler's internal [AllocatedRegister] types and maintain type safety
/// between virtual ops and those which have gone through register allocation.
/// A bit of copy/paste seemed worth it for that safety,
/// so here it is.
#[derive(Clone)]
pub(crate) enum AllocatedOp {
    /// Adds two registers.
    ///
    /// | Operation   | ```$rA = $rB + $rC;``` |
    /// | Syntax      | `add $rA, $rB, $rC`    |
    /// | Encoding    | `0x00 rA rB rC -`      |
    ///
    /// #### Panics
    /// - `$rA` is a reserved register.
    ///
    /// #### Execution
    /// `$of` is assigned the overflow of the operation.
    /// `$err` is cleared.
    ADD(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Adds a register and an immediate value.
    ///
    /// | Operation   | ```$rA = $rB + imm;```                  |
    /// | Syntax      | `addi $rA, $rB, immediate`              |
    /// | Encoding    | `0x00 rA rB i i`                        |
    ///
    /// #### Panics
    /// - `$rA` is a reserved register.
    ///
    /// #### Execution
    /// `$of` is assigned the overflow of the operation.
    /// `$err` is cleared.
    ADDI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Bitwise ANDs two registers.
    ///
    /// | Operation   | ```$rA = $rB & $rC;```      |
    /// | Syntax      | `and $rA, $rB, $rC`         |
    /// | Encoding    | `0x00 rA rB rC -`           |
    ///
    /// #### Panics
    /// - `$rA` is a reserved register.
    ///
    /// #### Execution
    /// `$of` and `$err` are cleared.
    AND(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Bitwise ANDs a register and an immediate value.
    ///
    /// | Operation   | ```$rA = $rB & imm;```                          |
    /// | Syntax      | `andi $rA, $rB, imm`                            |
    /// | Encoding    | `0x00 rA rB i i`                                |
    ///
    /// #### Panics
    /// - `$rA` is a reserved register.
    ///
    /// #### Execution
    /// `imm` is extended to 64 bits, with the high 52 bits set to `0`.
    /// `$of` and `$err` are cleared.
    ANDI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Divides two registers.
    ///
    /// | Operation   | ```$rA = $rB // $rC;``` |
    /// | Syntax      | `div $rA, $rB, $rC`     |
    /// | Encoding    | `0x00 rA rB rC -`       |
    ///
    /// #### Panics
    /// - `$rA` is a reserved register.
    ///
    /// #### Execution
    /// If `$rC == 0`, `$rA` is cleared and `$err` is set to `true`.
    /// Otherwise, `$err` is cleared.
    /// `$of` is cleared.
    DIV(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Divides a register and an immediate value.
    ///
    /// | Operation   | ```$rA = $rB // imm;```                    |
    /// | Syntax      | `divi $rA, $rB, imm`                       |
    /// | Encoding    | `0x00 rA rB i i`                           |
    ///
    /// #### Panics
    /// - `$rA` is a reserved register.
    ///
    /// #### Execution
    /// If `imm == 0`, `$rA` is cleared and `$err` is set to `true`.
    /// Otherwise, `$err` is cleared.
    /// `$of` is cleared.
    DIVI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Compares two registers for equality.
    ///
    /// | Operation   | ```$rA = $rB == $rC;```              |
    /// | Syntax      | `eq $rA, $rB, $rC`                   |
    /// | Encoding    | `0x00 rA rB rC -`                    |
    ///
    /// #### Panics
    /// - `$rA` is a reserved register,
    ///
    /// #### Execution
    /// `$of` and `$err` are cleared.
    EQ(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Raises one register to the power of another.
    ///
    /// | Operation   | ```$rA = $rB ** $rC;```                      |
    /// | Syntax      | `exp $rA, $rB, $rC`                          |
    /// | Encoding    | `0x00 rA rB rC -`                            |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// If the result cannot fit in 8 bytes, `$of` is set to `1`, otherwise
    /// `$of` is cleared.
    /// `$err` is cleared.
    EXP(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Raises one register to the power of an immediate value.
    ///
    /// | Operation   | ```$rA = $rB ** imm;```             |
    /// | Syntax      | `expi $rA, $rB, imm`                |
    /// | Encoding    | `0x00 rA rB i i`                    |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// If the result cannot fit in 8 bytes, `$of` is set to `1`, otherwise
    /// `$of` is cleared.
    /// `$err` is cleared.
    EXPI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Compares two registers for greater-than.
    ///
    /// | Operation   | ```$rA = $rB > $rC;```                   |
    /// | Syntax      | `gt $rA, $rB, $rC`                       |
    /// | Encoding    | `0x00 rA rB rC -`                        |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` and `$err` are cleared.
    GT(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// The (integer) logarithm base `$rC` of `$rB`.
    ///
    /// | Operation   | ```$rA = math.floor(math.log($rB, $rC));```  |
    /// | Syntax      | `mlog $rA, $rB, $rC`                         |
    /// | Encoding    | `0x00 rA rB rC -`                            |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// If `$rB == 0`, both `$rA` and `$of` are cleared and `$err` is set to
    /// `true`.
    ///
    /// If `$rC <= 1`, both `$rA` and `$of` are cleared and `$err` is set to
    /// `true`.
    ///
    /// Otherwise, `$of` and `$err` are cleared.
    MLOG(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// The (integer) `$rC`th root of `$rB`.
    ///
    /// | Operation   | ```$rA = math.floor(math.root($rB, $rC));``` |
    /// | Syntax      | `mroo $rA, $rB, $rC`                         |
    /// | Encoding    | `0x00 rA rB rC -`                            |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// If `$rC == 0`, both `$rA` and `$of` are cleared and `$err` is set to
    /// `true`.
    ///
    /// Otherwise, `$of` and `$err` are cleared.
    MROO(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Modulo remainder of two registers.
    ///
    /// | Operation   | ```$rA = $rB % $rC;```             |
    /// | Syntax      | `mod $rA, $rB, $rC`                |
    /// | Encoding    | `0x00 rA rB rC -`                  |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// If `$rC == 0`, both `$rA` and `$of` are cleared and `$err` is set to
    /// `true`.
    ///
    /// Otherwise, `$of` and `$err` are cleared.
    MOD(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Modulo remainder of a register and an immediate value.
    ///
    /// | Operation   | ```$rA = $rB % imm;```                                 |
    /// | Syntax      | `modi $rA, $rB, imm`                                   |
    /// | Encoding    | `0x00 rA rB i i`                                       |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// If `imm == 0`, both `$rA` and `$of` are cleared and `$err` is set to
    /// `true`.
    ///
    /// Otherwise, `$of` and `$err` are cleared.
    MODI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Copy from one register to another.
    ///
    /// | Operation   | ```$rA = $rB;```                   |
    /// | Syntax      | `move $rA, $rB`                    |
    /// | Encoding    | `0x00 rA rB - -`                   |
    /// | Notes       |                                    |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` and `$err` are cleared.
    MOVE(AllocatedRegister, AllocatedRegister),

    /// Multiplies two registers.
    ///
    /// | Operation   | ```$rA = $rB * $rC;```    |
    /// | Syntax      | `mul $rA, $rB, $rC`       |
    /// | Encoding    | `0x00 rA rB rC -`         |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` is assigned the overflow of the operation.
    ///
    /// `$err` is cleared.
    MUL(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Multiplies a register and an immediate value.
    ///
    /// | Operation   | ```$rA = $rB * imm;```                        |
    /// | Syntax      | `mul $rA, $rB, imm`                           |
    /// | Encoding    | `0x00 rA rB i i`                              |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` is assigned the overflow of the operation.
    ///
    /// `$err` is cleared.
    MULI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Bitwise NOT a register.
    ///
    /// | Operation   | ```$rA = ~$rB;```       |
    /// | Syntax      | `not $rA, $rB`          |
    /// | Encoding    | `0x00 rA rB - -`        |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` and `$err` are cleared.
    NOT(AllocatedRegister, AllocatedRegister),

    /// Bitwise ORs two registers.
    ///
    /// | Operation   | ```$rA = $rB \| $rC;```    |
    /// | Syntax      | `or $rA, $rB, $rC`         |
    /// | Encoding    | `0x00 rA rB rC -`          |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` and `$err` are cleared.
    OR(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Bitwise ORs a register and an immediate value.
    ///
    /// | Operation   | ```$rA = $rB \| imm;```                        |
    /// | Syntax      | `ori $rA, $rB, imm`                            |
    /// | Encoding    | `0x00 rA rB i i`                               |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `imm` is extended to 64 bits, with the high 52 bits set to `0`.
    ///
    /// `$of` and `$err` are cleared.
    ORI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Left shifts a register by a register.
    ///
    /// | Operation   | ```$rA = $rB << $rC;```               |
    /// | Syntax      | `sll $rA, $rB, $rC`                   |
    /// | Encoding    | `0x00 rA rB rC -`                     |
    /// | Notes       | Zeroes are shifted in.                |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` is assigned the overflow of the operation.
    ///
    /// `$err` is cleared.
    SLL(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Left shifts a register by an immediate value.
    ///
    /// | Operation   | ```$rA = $rB << imm;```                       |
    /// | Syntax      | `slli $rA, $rB, imm`                          |
    /// | Encoding    | `0x00 rA rB i i`                              |
    /// | Notes       | Zeroes are shifted in.                        |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` is assigned the overflow of the operation.
    ///
    /// `$err` is cleared.
    SLLI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Right shifts a register by a register.
    ///
    /// | Operation   | ```$rA = $rB >> $rC;```                |
    /// | Syntax      | `srl $rA, $rB, $rC`                    |
    /// | Encoding    | `0x00 rA rB rC -`                      |
    /// | Notes       | Zeroes are shifted in.                 |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` is assigned the underflow of the operation, as though `$of` is the
    /// high byte of a 128-bit register.
    ///
    /// `$err` is cleared.
    SRL(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Right shifts a register by an immediate value.
    ///
    /// | Operation   | ```$rA = $rB >> imm;```                        |
    /// | Syntax      | `srli $rA, $rB, imm`                           |
    /// | Encoding    | `0x00 rA rB i i`                               |
    /// | Notes       | Zeroes are shifted in.                         |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` is assigned the underflow of the operation, as though `$of` is the
    /// high byte of a 128-bit register.
    ///
    /// `$err` is cleared.
    SRLI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Subtracts two registers.
    ///
    /// | Operation   | ```$rA = $rB - $rC;```                           |
    /// | Syntax      | `sub $rA, $rB, $rC`                              |
    /// | Encoding    | `0x00 rA rB rC -`                                |
    /// | Notes       | `$of` is assigned the overflow of the operation. |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` is assigned the underflow of the operation, as though `$of` is the
    /// high byte of a 128-bit register.
    ///
    /// `$err` is cleared.
    SUB(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Subtracts a register and an immediate value.
    ///
    /// | Operation   | ```$rA = $rB - imm;```                           |
    /// | Syntax      | `subi $rA, $rB, imm`                             |
    /// | Encoding    | `0x00 rA rB i i`                                 |
    /// | Notes       | `$of` is assigned the overflow of the operation. |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` is assigned the underflow of the operation, as though `$of` is the
    /// high byte of a 128-bit register.
    ///
    /// `$err` is cleared.
    SUBI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Bitwise XORs two registers.
    ///
    /// | Operation   | ```$rA = $rB ^ $rC;```      |
    /// | Syntax      | `xor $rA, $rB, $rC`         |
    /// | Encoding    | `0x00 rA rB rC -`           |
    /// | Notes       |                             |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` and `$err` are cleared.
    XOR(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Bitwise XORs a register and an immediate value.
    ///
    /// | Operation   | ```$rA = $rB ^ imm;```                          |
    /// | Syntax      | `xori $rA, $rB, imm`                            |
    /// | Encoding    | `0x00 rA rB i i`                                |
    /// | Notes       |                                                 |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    ///
    /// #### Execution
    /// `$of` and `$err` are cleared.
    XORI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Set `$rA` to `true` if the `$rC <= tx.input[$rB].maturity`.
    ///
    /// | Operation   | ```$rA = checkinputmaturityverify($rB, $rC);``` |
    /// | Syntax      | `cimv $rA $rB $rC`                              |
    /// | Encoding    | `0x00 rA rB rC -`                               |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    /// - `$rC > tx.input[$rB].maturity`
    /// - the input `$rB` is not of type
    ///   [`InputType.Coin`](../protocol/tx_format.md)
    /// - `$rB > tx.inputsCount`
    ///
    /// #### Execution
    /// Otherwise, advance the program counter `$pc` by `4`.
    ///
    /// See also: [BIP-112](https://github.com/bitcoin/bips/blob/master/bip-0112.mediawiki) and [CLTV](#cltv-check-lock-time-verify).
    CIMV(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Set `$rA` to `true` if `$rB <= tx.maturity`.
    ///
    /// | Operation   | ```$rA = checktransactionmaturityverify($rB);``` |
    /// | Syntax      | `ctmv $rA $rB`                                   |
    /// | Encoding    | `0x00 rA rB - -`                                 |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    /// - `$rB > tx.maturity`
    ///
    /// #### Execution
    /// Otherwise, advance the program counter `$pc` by `4`.
    ///
    /// See also: [BIP-65](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki) and [Bitcoin's Time Locks](https://prestwi.ch/bitcoin-time-locks).
    CTMV(AllocatedRegister, AllocatedRegister),

    /// Jumps to the code instruction offset by `imm`.
    ///
    /// | Operation   | ```$pc = $is + imm * 4;```                     |
    /// | Syntax      | `ji imm`                                       |
    /// | Encoding    | `0x00 i i i i`                                 |
    ///
    /// #### Panics
    /// - `$is + imm * 4 > VM_MAX_RAM - 1`
    JI(VirtualImmediate24),

    /// Jump to the code instruction offset by `imm` if `$rA` is not equal to
    /// `$rB`.
    ///
    /// | Operation   | ```if $rA != $rB:```<br>```$pc = $is + imm *
    /// 4;```<br>```else:```<br>```$pc += 4;``` | Syntax      | `jnei $rA
    /// $rB imm` | Encoding    | `0x00 rA rB i i`
    ///
    /// #### Panics
    /// - `$is + imm * 4 > VM_MAX_RAM - 1`
    JNEI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Returns from [context](./main.md#contexts) with value `$rA`.
    ///
    /// | Operation   | ```return($rA);```
    /// | Syntax      | `ret $rA`
    /// | Encoding    | `0x00 rA - - -`
    ///
    /// If current context is external, cease VM execution and return `$rA`.
    ///
    /// Returns from contract call, popping the call frame. Before popping:
    ///
    /// 1. Return the unused forwarded gas to the caller:
    ///     - `$cgas = $cgas + $fp->$cgas` (add remaining context gas from
    ///       previous context to current remaining context gas)
    ///
    /// Then pop the call frame and restoring registers _except_ `$ggas` and
    /// `$cgas`. Afterwards, set the following registers:
    ///
    /// 1. `$pc = $pc + 4` (advance program counter from where we called)
    RET(AllocatedRegister),

    /// Extend the current call frame's stack by an immediate value.
    ///
    /// | Operation   | ```$sp = $sp + imm```
    /// | Syntax      | `cfei imm`
    /// | Encoding    | `0x00 i i i i`
    /// | Notes       | Does not initialize memory.
    ///
    /// #### Panics
    /// - `$sp + imm` overflows
    /// - `$sp + imm > $hp`
    CFEI(VirtualImmediate24),

    /// Shrink the current call frame's stack by an immediate value.
    ///
    /// | Operation   | ```$sp = $sp - imm```
    /// | Syntax      | `cfsi imm`
    /// | Encoding    | `0x00 i i i i`
    /// | Notes       | Does not clear memory.
    ///
    /// #### Panics
    /// - `$sp - imm` underflows
    /// - `$sp - imm < $ssp`
    CFSI(VirtualImmediate24),

    /// A byte is loaded from the specified address offset by `imm`.
    ///
    /// | Operation   | ```$rA = MEM[$rB + imm, 1];```
    /// | Syntax      | `lb $rA, $rB, imm`
    /// | Encoding    | `0x00 rA rB i i`
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    /// - `$rB + imm + 1` overflows
    /// - `$rB + imm + 1 > VM_MAX_RAM`
    LB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// A word is loaded from the specified address offset by `imm`.
    /// | Operation   | ```$rA = MEM[$rB + imm, 8];```
    /// | Syntax      | `lw $rA, $rB, imm`
    /// | Encoding    | `0x00 rA rB i i`
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    /// - `$rB + imm + 8` overflows
    /// - `$rB + imm + 8 > VM_MAX_RAM`
    LW(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Allocate a number of bytes from the heap.
    ///
    /// | Operation   | ```$hp = $hp - $rA;```                    |
    /// | Syntax      | `aloc $rA`                                |
    /// | Encoding    | `0x00 rA - - -`                           |
    /// | Notes       | Does not initialize memory.               |
    ///
    /// #### Panics
    /// - `$hp - $rA` underflows
    /// - `$hp - $rA < $sp`
    ALOC(AllocatedRegister),

    /// Clear bytes in memory.
    ///
    /// | Operation   | ```MEM[$rA, $rB] = 0;``` |
    /// | Syntax      | `mcl $rA, $rB`           |
    /// | Encoding    | `0x00 rA rB - -`         |
    ///
    /// #### Panics
    /// - `$rA + $rB` overflows
    /// - `$rA + $rB > VM_MAX_RAM`
    /// - `$rB > MEM_MAX_ACCESS_SIZE`
    /// - The memory range `MEM[$rA, $rB]`  does not pass [ownership
    ///   check](./main.md#ownership)
    MCL(AllocatedRegister, AllocatedRegister),

    /// Clear bytes in memory.
    ///
    /// | Operation   | ```MEM[$rA, imm] = 0;``` |
    /// | Syntax      | `mcli $rA, imm`          |
    /// | Encoding    | `0x00 rA i i i`          |
    ///
    /// #### Panics
    /// - `$rA + imm` overflows
    /// - `$rA + imm > VM_MAX_RAM`
    /// - `imm > MEM_MAX_ACCESS_SIZE`
    /// - The memory range `MEM[$rA, imm]`  does not pass [ownership
    ///   check](./main.md#ownership)
    MCLI(AllocatedRegister, VirtualImmediate18),

    /// Copy bytes in memory.
    ///
    /// | Operation   | ```MEM[$rA, $rC] = MEM[$rB, $rC];``` |
    /// | Syntax      | `mcp $rA, $rB, $rC`                  |
    /// | Encoding    | `0x00 rA rB rC -`                    |
    ///
    /// #### Panics
    /// - `$rA + $rC` overflows
    /// - `$rB + $rC` overflows
    /// - `$rA + $rC > VM_MAX_RAM`
    /// - `$rB + $rC > VM_MAX_RAM`
    /// - `$rC > MEM_MAX_ACCESS_SIZE`
    /// - The memory ranges `MEM[$rA, $rC]` and `MEM[$rB, $rC]` overlap
    /// - The memory range `MEM[$rA, $rC]`  does not pass [ownership
    ///   check](./main.md#ownership)
    MCP(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Compare bytes in memory.
    ///
    /// | Operation   | ```$rA = MEM[$rB, $rD] == MEM[$rC, $rD];``` |
    /// | Syntax      | `meq $rA, $rB, $rC, $rD`                    |
    /// | Encoding    | `0x00 rA rB rC rD`                          |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    /// - `$rB + $rD` overflows
    /// - `$rC + $rD` overflows
    /// - `$rB + $rD > VM_MAX_RAM`
    /// - `$rC + $rD > VM_MAX_RAM`
    /// - `$rD > MEM_MAX_ACCESS_SIZE`
    MEQ(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),

    /// The least significant byte of `$rB` is stored at the address `$rA`
    /// offset by `imm`.
    ///
    /// | Operation   | ```MEM[$rA + imm, 1] = $rB[7, 1];```    |
    /// | Syntax      | `sb $rA, $rB, imm`                      |
    /// | Encoding    | `0x00 rA rB i i`                        |
    ///
    /// #### Panics
    /// - `$rA + imm + 1` overflows
    /// - `$rA + imm + 1 > VM_MAX_RAM`
    /// - The memory range `MEM[$rA + imm, 1]`  does not pass [ownership
    ///   check](./main.md#ownership)
    SB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// The value of `$rB` is stored at the address `$rA` offset by `imm`.
    ///
    /// | Operation   | ```MEM[$rA + imm, 8] = $rB;```
    /// | Syntax      | `sw $rA, $rB, imm`
    /// | Encoding    | `0x00 rA rB i i`
    ///
    /// #### Panics
    /// - `$rA + imm + 8` overflows
    /// - `$rA + imm + 8 > VM_MAX_RAM`
    /// - The memory range `MEM[$rA + imm, 8]`  does not pass [ownership
    ///   check](./main.md#ownership)
    SW(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /// Get block header hash.
    ///
    /// | Operation   | ```MEM[$rA, 32] = blockhash($rB);``` |
    /// | Syntax      | `bhsh $rA $rB`                       |
    /// | Encoding    | `0x00 rA rB - -`                     |
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - The memory range `MEM[$rA, 32]`  does not pass [ownership
    ///   check](./main.md#ownership)
    ///
    /// Block header hashes for blocks with height greater than or equal to
    /// current block height are zero (`0x00**32`).
    BHSH(AllocatedRegister, AllocatedRegister),

    /// Get Fuel block height.
    ///
    /// | Operation   | ```$rA = blockheight();``` |
    /// | Syntax      | `bhei $rA`                 |
    /// | Encoding    | `0x00 rA - - -`            |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    BHEI(AllocatedRegister),

    /// Burn `$rA` coins of the current contract's color.
    ///
    /// | Operation   | ```burn($rA);```                                  |
    /// | Syntax      | `burn $rA`                                        |
    /// | Encoding    | `0x00 rA - - -`                                   |
    ///
    /// #### Panic
    /// - Balance of color `MEM[$fp, 32]` of output with contract ID `MEM[$fp,
    ///   32]` minus `$rA` underflows
    /// - `$fp == 0` (in the script context)
    ///
    /// For output with contract ID `MEM[$fp, 32]`, decrease balance of color
    /// `MEM[$fp, 32]` by `$rA`.
    ///
    /// This modifies the `balanceRoot` field of the appropriate output.
    BURN(AllocatedRegister),

    /// Call contract.
    ///
    /// | Syntax      | `call $rA $rB $rC $rD` |
    /// | Encoding    | `0x00 rA rB rC rD`     |
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rC + 32` overflows
    /// - Contract with ID `MEM[$rA, 32]` is not in `tx.inputs`
    /// - Reading past `MEM[VM_MAX_RAM - 1]`
    /// - Any output range does not pass [ownership check](./main.md#ownership)
    /// - In an external context, if `$rB > MEM[balanceOfStart(MEM[$rC, 32]),
    ///   8]`
    /// - In an internal context, if `$rB` is greater than the balance of color
    ///   `MEM[$rC, 32]` of output with contract ID `MEM[$rA, 32]`
    ///
    /// Register `$rA` is a memory address from which the following fields are
    /// set (word-aligned).
    ///
    /// `$rD` is the amount of gas to forward. If it is set to an amount greater
    /// than the available gas, all available gas is forwarded.
    ///
    /// For output with contract ID `MEM[$rA, 32]`, increase balance of color
    /// `MEM[$rC, 32]` by `$rB`. In an external context, decrease
    /// `MEM[balanceOfStart(MEM[$rC, 32]), 8]` by `$rB`. In an internal context,
    /// decrease color `MEM[$rC, 32]` balance of output with contract ID
    /// `MEM[$fp, 32]` by `$rB`.
    ///
    /// A [call frame](./main.md#call-frames) is pushed at `$sp`. In addition to
    /// filling in the values of the call frame, the following registers are
    /// set:
    ///
    /// 1. `$fp = $sp` (on top of the previous call frame is the beginning of
    /// this call frame) 1. Set `$ssp` and `$sp` to the start of the
    /// writable stack area of the call frame. 1. Set `$pc` and `$is` to the
    /// starting address of the code. 1. `$bal = $rD` (forward coins)
    /// 1. `$cgas = $rD` or all available gas (forward gas)
    ///
    /// This modifies the `balanceRoot` field of the appropriate output(s).
    CALL(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),

    /// Copy `$rD` bytes of code starting at `$rC` for contract with ID equal to
    /// the 32 bytes in memory starting at `$rB` into memory starting at `$rA`.
    ///
    /// | Operation   | ```MEM[$rA, $rD] = code($rB, $rC, $rD);```
    /// | Syntax      | `ccp $rA, $rB, $rC, $rD`
    /// | Encoding    | `0x00 rA rB rC rD`
    /// | Notes       | If `$rD` is greater than the code size, zero bytes are
    /// filled in.
    ///
    /// #### Panics
    /// - `$rA + $rD` overflows
    /// - `$rB + 32` overflows
    /// - `$rA + $rD > VM_MAX_RAM`
    /// - `$rB + 32 > VM_MAX_RAM`
    /// - The memory range `MEM[$rA, $rD]`  does not pass [ownership
    ///   check](./main.md#ownership)
    /// - `$rD > MEM_MAX_ACCESS_SIZE`
    /// - Contract with ID `MEM[$rB, 32]` is not in `tx.inputs`
    CCP(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),

    /// Set the 32 bytes in memory starting at `$rA` to the code root for
    /// contract with ID equal to the 32 bytes in memory starting at `$rB`.
    ///
    /// | Operation   | ```MEM[$rA, 32] = coderoot(MEM[$rB, 32]);```
    /// | Syntax      | `croo $rA, $rB`
    /// | Encoding    | `0x00 rA rB - -`
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rB + 32` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$rB + 32 > VM_MAX_RAM`
    /// - The memory range `MEM[$rA, 32]`  does not pass [ownership
    ///   check](./main.md#ownership)
    /// - Contract with ID `MEM[$rB, 32]` is not in `tx.inputs`
    ///
    /// Code root compuration is defined
    /// [here](../protocol/identifiers.md#contract-id).
    CROO(AllocatedRegister, AllocatedRegister),

    /// Set `$rA` to the size of the code for contract with ID equal to the 32
    /// bytes in memory starting at `$rB`.
    ///
    /// | Operation   | ```$rA = codesize(MEM[$rB, 32]);```
    /// | Syntax      | `csiz $rA, $rB`
    /// | Encoding    | `0x00 rA rB - -`
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    /// - `$rB + 32` overflows
    /// - `$rB + 32 > VM_MAX_RAM`
    /// - Contract with ID `MEM[$rB, 32]` is not in `tx.inputs`
    CSIZ(AllocatedRegister, AllocatedRegister),

    /// Get block proposer address.
    ///
    /// | Operation   | ```MEM[$rA, 32] = coinbase();``` |
    /// | Syntax      | `cb $rA`                         |
    /// | Encoding    | `0x00 rA - - -`                  |
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - The memory range `MEM[$rA, 32]`  does not pass [ownership
    ///   check](./main.md#ownership)
    CB(AllocatedRegister),

    /// Copy `$rC` bytes of code starting at `$rB` for contract with ID equal to
    /// the 32 bytes in memory starting at `$rA` into memory starting at `$ssp`.
    ///
    /// | Operation   | ```MEM[$ssp, $rC] = code($rA, $rB, $rC);```
    /// | Syntax      | `ldc $rA, $rB, $rC`
    /// | Encoding    | `0x00 rA rB rC -`
    /// | Notes       | If `$rC` is greater than the code size, zero bytes are
    /// filled in.
    ///
    /// #### Panics
    /// - `$ssp + $rC` overflows
    /// - `$rA + 32` overflows
    /// - `$ssp + $rC > VM_MAX_RAM`
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$ssp != $sp`
    /// - `$ssp + $rC > $hp`
    /// - `$rC > CONTRACT_MAX_SIZE`
    /// - `$rC > MEM_MAX_ACCESS_SIZE`
    /// - Contract with ID `MEM[$rA, 32]` is not in `tx.inputs`
    ///
    /// Increment `$hp->codesize`, `$ssp`, and `$sp` by `$rC` padded to word
    /// alignment.
    ///
    /// This opcode can be used to concatenate the code of multiple contracts
    /// together. It can only be used when the stack area of the call frame is
    /// unused (i.e. prior to being used).
    LDC(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Log an event. This is a no-op.
    ///
    /// | Operation   | ```log($rA, $rB, $rC, $rD);``` |
    /// | Syntax      | `log $rA, $rB, $rC, $rD`       |
    /// | Encoding    | `0x00 rA rB rC rD`             |
    LOG(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),

    /// Mint `$rA` coins of the current contract's color.
    ///
    /// | Operation   | ```mint($rA);```                                  |
    /// | Syntax      | `mint $rA`                                        |
    /// | Encoding    | `0x00 rA - - -`                                   |
    ///
    /// #### Panics
    /// - Balance of color `MEM[$fp, 32]` of output with contract ID `MEM[$fp,
    ///   32]` plus `$rA` overflows
    /// - `$fp == 0` (in the script context)
    ///
    /// For output with contract ID `MEM[$fp, 32]`, increase balance of color
    /// `MEM[$fp, 32]` by `$rA`.
    ///
    /// This modifies the `balanceRoot` field of the appropriate output.
    MINT(AllocatedRegister),

    /// Halt execution, reverting state changes and returning value in `$rA`.
    ///
    /// | Operation   | ```revert($rA);```
    /// | Syntax      | `rvrt $rA`
    /// | Encoding    | `0x00 rA - - -`
    ///
    /// After a revert:
    ///
    /// 1. All [OutputContract](../protocol/tx_format.md#outputcontract) outputs
    /// will have the same `amount` and `stateRoot` as on initialization. 1.
    /// All [OutputVariable](../protocol/tx_format.md outputs#outputvariable)
    /// outputs will have `to` and `amount` of zero.
    /// 1. All [OutputContractConditional](../protocol/tx_format.md#
    /// outputcontractconditional) outputs will have `contractID`, `amount`, and
    /// `stateRoot` of zero.
    RVRT(AllocatedRegister),

    /// Copy `$rC` bytes of code starting at `$rB` for contract with static
    /// index `$rA` into memory starting at `$ssp`.
    ///
    /// | Operation   | ```MEM[$ssp, $rC] = scode($rA, $rB, $rC);```
    /// | Syntax      | `sloadcode $rA, $rB, $rC`
    /// | Encoding    | `0x00 rA rB rC -`
    /// | Notes       | If `$rC` is greater than the code size, zero bytes
    /// are filled in.                                               |
    ///
    /// #### Panics
    /// - `$ssp + $rC` overflows
    /// - `$ssp + $rC > VM_MAX_RAM`
    /// - `$rA >= MAX_STATIC_CONTRACTS`
    /// - `$rA` is greater than or equal to `staticContractsCount` for the
    ///   contract with ID `MEM[$fp, 32]`
    /// - `$ssp != $sp`
    /// - `$ssp + $rC > $hp`
    /// - `$rC > CONTRACT_MAX_SIZE`
    /// - `$rC > MEM_MAX_ACCESS_SIZE`
    /// - `$fp == 0` (in the script context)
    ///
    /// Increment `$hp->codesize`, `$ssp`, and `$sp` by `$rC` padded to word
    /// alignment.
    ///
    /// This opcode can be used to concatenate the code of multiple contracts
    /// together. It can only be used when the stack area of the call frame is
    /// unused (i.e. prior to being used).
    SLDC(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// A word is read from the current contract's state.
    ///
    /// | Operation   | ```$rA = STATE[MEM[$rB, 32]][0, 8];```            |
    /// | Syntax      | `srw $rA, $rB`                                    |
    /// | Encoding    | `0x00 rA rB - -`                                  |
    /// | Notes       | Returns zero if the state element does not exist. |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    /// - `$rB + 32` overflows
    /// - `$rB + 32 > VM_MAX_RAM`
    /// - `$fp == 0` (in the script context)
    SRW(AllocatedRegister, AllocatedRegister),

    /// 32 bytes is read from the current contract's state.
    ///
    /// | Operation   | ```MEM[$rA, 32] = STATE[MEM[$rB, 32]];```           |
    /// | Syntax      | `srwx $rA, $rB`                                     |
    /// | Encoding    | `0x00 rA rB - -`                                    |
    /// | Notes       | Returns zero if the state element does not exist.   |
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rB + 32` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$rB + 32 > VM_MAX_RAM`
    /// - The memory range `MEM[$rA, 32]`  does not pass [ownership
    ///   check](./main.md#ownership)
    /// - `$fp == 0` (in the script context)
    SRWQ(AllocatedRegister, AllocatedRegister),

    /// A word is written to the current contract's state.
    ///
    /// | Operation   | ```STATE[MEM[$rA, 32]][0, 8] = $rB;```             |
    /// | Syntax      | `sww $rA $rB`                                      |
    /// | Encoding    | `0x00 rA rB - -`                                   |
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$fp == 0` (in the script context)
    SWW(AllocatedRegister, AllocatedRegister),

    /// 32 bytes is written to the current contract's state.
    ///
    /// | Operation   | ```STATE[MEM[$rA, 32]] = MEM[$rB, 32];```            |
    /// | Syntax      | `swwx $rA, $rB`                                      |
    /// | Encoding    | `0x00 rA rB - -`                                     |
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rB + 32` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$rB + 32 > VM_MAX_RAM`
    /// - `$fp == 0` (in the script context)
    SWWQ(AllocatedRegister, AllocatedRegister),

    /// Transfer `$rB` coins with color at `$rC` to contract with ID at `$rA`.
    ///
    /// | Operation   | ```transfer(MEM[$rA, 32], $rB, MEM[$rC, 32]);```
    /// | Syntax      | `tr $rA,  $rB, $rC`
    /// | Encoding    | `0x00 rA rB rC -`
    ///
    /// Given helper `balanceOfStart(color: byte[32]) -> uint32` which returns
    /// the memory address of `color` balance, or `0` if `color` has no balance.
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rC + 32` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$rC + 32 > VM_MAX_RAM`
    /// - Contract with ID `MEM[$rA, 32]` is not in `tx.inputs`
    /// - In an external context, if `$rB > MEM[balanceOf(MEM[$rC, 32]), 8]`
    /// - In an internal context, if `$rB` is greater than the balance of color
    ///   `MEM[$rC, 32]` of output with contract ID `MEM[$fp, 32]`
    /// - `$rB == 0`
    ///
    /// For output with contract ID `MEM[$rA, 32]`, increase balance of color
    /// `MEM[$rC, 32]` by `$rB`. In an external context, decrease
    /// `MEM[balanceOfStart(MEM[$rC, 32]), 8]` by `$rB`. In an internal context,
    /// decrease color `MEM[$rC, 32]` balance of output with contract ID
    /// `MEM[$fp, 32]` by `$rB`.
    ///
    /// This modifies the `balanceRoot` field of the appropriate output(s).
    TR(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Transfer `$rC` coins with color at `$rD` to address at `$rA`, with
    /// output `$rB`. | Operation   | ```transferout(MEM[$rA, 32], $rB, $rC,
    /// MEM[$rD, 32]);``` | Syntax      | `tro $rA, $rB, $rC, $rD`
    /// | Encoding    | `0x00 rA rB rC rD`
    ///
    /// Given helper `balanceOfStart(color: byte[32]) -> uint32` which returns
    /// the memory address of `color` balance, or `0` if `color` has no balance.
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rD + 32` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$rD + 32 > VM_MAX_RAM`
    /// - `$rB > tx.outputsCount`
    /// - In an external context, if `$rC > MEM[balanceOf(MEM[$rD, 32]), 8]`
    /// - In an internal context, if `$rC` is greater than the balance of color
    ///   `MEM[$rD, 32]` of output with contract ID `MEM[$fp, 32]`
    /// - `$rC == 0`
    /// - `tx.outputs[$rB].type != OutputType.Variable`
    /// - `tx.outputs[$rB].amount != 0`
    ///
    /// In an external context, decrease `MEM[balanceOfStart(MEM[$rD, 32]), 8]`
    /// by `$rC`. In an internal context, decrease color `MEM[$rD, 32]` balance
    /// of output with contract ID `MEM[$fp, 32]` by `$rC`. Then set:
    ///
    /// - `tx.outputs[$rB].to = MEM[$rA, 32]`
    /// - `tx.outputs[$rB].amount = $rC`
    /// - `tx.outputs[$rB].color = MEM[$rD, 32]`
    ///
    /// This modifies the `balanceRoot` field of the appropriate output(s).
    TRO(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),

    /// The 64-byte public key (x, y) recovered from 64-byte
    /// signature starting at `$rB` on 32-byte message hash starting at `$rC`. |
    ///
    /// | Operation   | ```MEM[$rA, 64] = ecrecover(MEM[$rB, 64], MEM[$rC,
    /// 32]);``` | Syntax      | `ecr $rA, $rB, $rC`
    /// | Encoding    | `0x00 rA rB rC -`
    ///
    /// #### Panics
    /// - `$rA + 64` overflows
    /// - `$rB + 64` overflows
    /// - `$rC + 32` overflows
    /// - `$rA + 64 > VM_MAX_RAM`
    /// - `$rB + 64 > VM_MAX_RAM`
    /// - `$rC + 32 > VM_MAX_RAM`
    /// - The memory range `MEM[$rA, 64]`  does not pass [ownership
    ///   check](./main.md#ownership)
    ///
    /// To get the address, hash the public key with
    /// [SHA-2-256](#sha256-sha-2-256).
    ECR(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// The keccak-256 hash of `$rC` bytes starting at `$rB`.
    ///
    /// | Operation   | ```MEM[$rA, 32] = keccak256(MEM[$rB, $rC]);```
    /// | Syntax      | `k256 $rA, $rB, $rC`
    /// | Encoding    | `0x00 rA rB rC -`
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rB + $rC` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$rB + $rC > VM_MAX_RAM`
    /// - The memory range `MEM[$rA, 32]`  does not pass [ownership
    ///   check](./main.md#ownership)
    /// - `$rC > MEM_MAX_ACCESS_SIZE`
    K256(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// The SHA-2-256 hash of `$rC` bytes starting at `$rB`.
    ///
    /// | Operation   | ```MEM[$rA, 32] = sha256(MEM[$rB, $rC]);```          |
    /// | Syntax      | `s256 $rA, $rB, $rC`                                 |
    /// | Encoding    | `0x00 rA rB rC -`                                    |
    ///
    /// #### Panics
    /// - `$rA + 32` overflows
    /// - `$rB + $rC` overflows
    /// - `$rA + 32 > VM_MAX_RAM`
    /// - `$rB + $rC > VM_MAX_RAM`
    /// - The memory range `MEM[$rA, 32]`  does not pass [ownership
    ///   check](./main.md#ownership)
    /// - `$rC > MEM_MAX_ACCESS_SIZE`
    S256(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /// Performs no operation.
    ///
    /// | Operation   |                        |
    /// | Syntax      | `noop`                 |
    /// | Encoding    | `0x00 - - - -`         |
    ///
    /// `$of` and `$err` are cleared.
    NOOP,

    /// Set `$flag` to `$rA`.
    ///
    /// | Operation   | ```$flag = $rA;```    |
    /// | Syntax      | `flag $rA`            |
    /// | Encoding    | `0x00 rA - - -`       |
    FLAG(AllocatedRegister),

    /// Undefined opcode, potentially from inconsistent serialization
    Undefined,
}
