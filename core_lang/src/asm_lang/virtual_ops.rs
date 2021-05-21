//! This module contains abstracted versions of bytecode primitives that the compiler uses to
//! ensure correctness and safety.
//!
//! The immediate types are used to safely construct numbers that are within their bounds, and the
//! ops are clones of the actual opcodes, but with the safe primitives as arguments.

use super::{
    allocated_ops::{AllocatedOp, AllocatedRegister},
    Op,
};
use crate::asm_generation::RegisterPool;
use crate::{error::*, Ident};
use pest::Span;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fmt;

/// Represents virtual registers that have yet to be allocated.
/// Note that only the Virtual variant will be allocated, and the Constant variant refers to
/// reserved registers.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum VirtualRegister {
    Virtual(String),
    Constant(ConstantRegister),
}

impl Into<VirtualRegister> for &VirtualRegister {
    fn into(self) -> VirtualRegister {
        self.clone()
    }
}

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirtualRegister::Virtual(name) => write!(f, "$r{}", name),
            VirtualRegister::Constant(name) => {
                write!(f, "{}", name)
            }
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
/// These are the special registers defined in the spec
pub enum ConstantRegister {
    Zero,
    One,
    Overflow,
    ProgramCounter,
    StackStartPointer,
    StackPointer,
    FramePointer,
    HeapPointer,
    Error,
    GlobalGas,
    ContextGas,
    Balance,
    InstructionStart,
    Flags,
}

impl fmt::Display for ConstantRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ConstantRegister::*;
        let text = match self {
            Zero => "$zero",
            One => "$one",
            Overflow => "$of",
            ProgramCounter => "$pc",
            StackStartPointer => "$ssp",
            StackPointer => "$sp",
            FramePointer => "$fp",
            HeapPointer => "$hp",
            Error => "$err",
            GlobalGas => "$ggas",
            ContextGas => "$cgas",
            Balance => "$bal",
            InstructionStart => "$is",
            Flags => "$flag",
        };
        write!(f, "{}", text)
    }
}

/// 6-bits immediate value type
#[derive(Clone)]
pub struct VirtualImmediate06 {
    value: u8,
}

impl VirtualImmediate06 {
    pub(crate) fn new<'sc>(raw: u64, err_msg_span: Span<'sc>) -> Result<Self, CompileError<'sc>> {
        if raw > 0b111_111 {
            return Err(CompileError::Immediate06TooLarge {
                val: raw,
                span: err_msg_span,
            });
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
}

/// 12-bits immediate value type
#[derive(Clone)]
pub struct VirtualImmediate12 {
    value: u16,
}

impl VirtualImmediate12 {
    pub(crate) fn new<'sc>(raw: u64, err_msg_span: Span<'sc>) -> Result<Self, CompileError<'sc>> {
        if raw > 0b111_111_111_111 {
            return Err(CompileError::Immediate12TooLarge {
                val: raw,
                span: err_msg_span,
            });
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().expect(&(msg.into())),
        }
    }
}

/// 18-bits immediate value type
#[derive(Clone)]
pub struct VirtualImmediate18 {
    value: u32,
}
impl VirtualImmediate18 {
    pub(crate) fn new<'sc>(raw: u64, err_msg_span: Span<'sc>) -> Result<Self, CompileError<'sc>> {
        if raw > 0b111_111_111_111_111_111 {
            return Err(CompileError::Immediate18TooLarge {
                val: raw,
                span: err_msg_span,
            });
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().expect(&(msg.into())),
        }
    }
}

/// 24-bits immediate value type
#[derive(Clone)]
pub struct VirtualImmediate24 {
    value: u32,
}
impl VirtualImmediate24 {
    pub(crate) fn new<'sc>(raw: u64, err_msg_span: Span<'sc>) -> Result<Self, CompileError<'sc>> {
        if raw > 0b111_111_111_111_111_111_111_111 {
            return Err(CompileError::Immediate24TooLarge {
                val: raw,
                span: err_msg_span,
            });
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().expect(&(msg.into())),
        }
    }
}

/// This enum is unfortunately a redundancy of the [fuel_asm::Opcode] enum. This variant, however,
/// allows me to use the compiler's internal [VirtualRegister] types and maintain type safety
/// between virtual ops and the real opcodes. A bit of copy/paste seemed worth it for that safety,
/// so here it is.
#[derive(Clone)]
pub(crate) enum VirtualOp {
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
    ADD(VirtualRegister, VirtualRegister, VirtualRegister),

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
    ADDI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    AND(VirtualRegister, VirtualRegister, VirtualRegister),

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
    ANDI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    DIV(VirtualRegister, VirtualRegister, VirtualRegister),

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
    DIVI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    EQ(VirtualRegister, VirtualRegister, VirtualRegister),

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
    EXP(VirtualRegister, VirtualRegister, VirtualRegister),

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
    EXPI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    GT(VirtualRegister, VirtualRegister, VirtualRegister),

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
    MLOG(VirtualRegister, VirtualRegister, VirtualRegister),

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
    MROO(VirtualRegister, VirtualRegister, VirtualRegister),

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
    MOD(VirtualRegister, VirtualRegister, VirtualRegister),

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
    MODI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    MOVE(VirtualRegister, VirtualRegister),

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
    MUL(VirtualRegister, VirtualRegister, VirtualRegister),

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
    MULI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    NOT(VirtualRegister, VirtualRegister),

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
    OR(VirtualRegister, VirtualRegister, VirtualRegister),

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
    ORI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    SLL(VirtualRegister, VirtualRegister, VirtualRegister),

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
    SLLI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    SRL(VirtualRegister, VirtualRegister, VirtualRegister),

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
    SRLI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    SUB(VirtualRegister, VirtualRegister, VirtualRegister),

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
    SUBI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    XOR(VirtualRegister, VirtualRegister, VirtualRegister),

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
    XORI(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    CIMV(VirtualRegister, VirtualRegister, VirtualRegister),

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
    CTMV(VirtualRegister, VirtualRegister),

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
    JNEI(VirtualRegister, VirtualRegister, Label),

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
    RET(VirtualRegister),

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
    LB(VirtualRegister, VirtualRegister, VirtualImmediate12),

    /// A word is loaded from the specified address offset by `imm`.
    /// | Operation   | ```$rA = MEM[$rB + imm, 8];```
    /// | Syntax      | `lw $rA, $rB, imm`
    /// | Encoding    | `0x00 rA rB i i`
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    /// - `$rB + imm + 8` overflows
    /// - `$rB + imm + 8 > VM_MAX_RAM`
    LW(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    ALOC(VirtualRegister),

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
    MCL(VirtualRegister, VirtualRegister),

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
    MCLI(VirtualRegister, VirtualImmediate18),

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
    MCP(VirtualRegister, VirtualRegister, VirtualRegister),

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
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
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
    SB(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    SW(VirtualRegister, VirtualRegister, VirtualImmediate12),

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
    BHSH(VirtualRegister, VirtualRegister),

    /// Get Fuel block height.
    ///
    /// | Operation   | ```$rA = blockheight();``` |
    /// | Syntax      | `bhei $rA`                 |
    /// | Encoding    | `0x00 rA - - -`            |
    ///
    /// #### Panics
    /// - `$rA` is a [reserved register](./main.md#semantics)
    BHEI(VirtualRegister),

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
    BURN(VirtualRegister),

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
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
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
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
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
    CROO(VirtualRegister, VirtualRegister),

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
    CSIZ(VirtualRegister, VirtualRegister),

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
    CB(VirtualRegister),

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
    LDC(VirtualRegister, VirtualRegister, VirtualRegister),

    /// Log an event. This is a no-op.
    ///
    /// | Operation   | ```log($rA, $rB, $rC, $rD);``` |
    /// | Syntax      | `log $rA, $rB, $rC, $rD`       |
    /// | Encoding    | `0x00 rA rB rC rD`             |
    LOG(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
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
    MINT(VirtualRegister),

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
    RVRT(VirtualRegister),

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
    SLDC(VirtualRegister, VirtualRegister, VirtualRegister),

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
    SRW(VirtualRegister, VirtualRegister),

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
    SRWQ(VirtualRegister, VirtualRegister),

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
    SWW(VirtualRegister, VirtualRegister),

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
    SWWQ(VirtualRegister, VirtualRegister),

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
    TR(VirtualRegister, VirtualRegister, VirtualRegister),

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
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
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
    ECR(VirtualRegister, VirtualRegister, VirtualRegister),

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
    K256(VirtualRegister, VirtualRegister, VirtualRegister),

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
    S256(VirtualRegister, VirtualRegister, VirtualRegister),

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
    FLAG(VirtualRegister),

    /// Undefined opcode, potentially from inconsistent serialization
    Undefined,
}

impl VirtualOp {
    pub(crate) fn registers(&self) -> HashSet<&VirtualRegister> {
        use VirtualOp::*;
        (match self {
            ADD(r1, r2, r3) => vec![r1, r2, r3],
            ADDI(r1, r2, _i) => vec![r1, r2],
            AND(r1, r2, r3) => vec![r1, r2, r3],
            ANDI(r1, r2, _i) => vec![r1, r2],
            DIV(r1, r2, r3) => vec![r1, r2, r3],
            DIVI(r1, r2, _i) => vec![r1, r2],
            EQ(r1, r2, r3) => vec![r1, r2, r3],
            EXP(r1, r2, r3) => vec![r1, r2, r3],
            EXPI(r1, r2, _i) => vec![r1, r2],
            GT(r1, r2, r3) => vec![r1, r2, r3],
            MLOG(r1, r2, r3) => vec![r1, r2, r3],
            MROO(r1, r2, r3) => vec![r1, r2, r3],
            MOD(r1, r2, r3) => vec![r1, r2, r3],
            MODI(r1, r2, _i) => vec![r1, r2],
            MOVE(r1, r2) => vec![r1, r2],
            MUL(r1, r2, r3) => vec![r1, r2, r3],
            MULI(r1, r2, _i) => vec![r1, r2],
            NOT(r1, r2) => vec![r1, r2],
            OR(r1, r2, r3) => vec![r1, r2, r3],
            ORI(r1, r2, _i) => vec![r1, r2],
            SLL(r1, r2, r3) => vec![r1, r2, r3],
            SLLI(r1, r2, _i) => vec![r1, r2],
            SRL(r1, r2, r3) => vec![r1, r2, r3],
            SRLI(r1, r2, _i) => vec![r1, r2],
            SUB(r1, r2, r3) => vec![r1, r2, r3],
            SUBI(r1, r2, _i) => vec![r1, r2],
            XOR(r1, r2, r3) => vec![r1, r2, r3],
            XORI(r1, r2, _i) => vec![r1, r2],
            CIMV(r1, r2, r3) => vec![r1, r2, r3],
            CTMV(r1, r2) => vec![r1, r2],
            JI(_im) => vec![],
            JNEI(r1, r2, _i) => vec![r1, r2],
            RET(r1) => vec![r1],
            CFEI(_imm) => vec![],
            CFSI(_imm) => vec![],
            LB(r1, r2, _i) => vec![r1, r2],
            LW(r1, r2, _i) => vec![r1, r2],
            ALOC(_imm) => vec![],
            MCL(r1, r2) => vec![r1, r2],
            MCLI(r1, _imm) => vec![r1],
            MCP(r1, r2, r3) => vec![r1, r2, r3],
            MEQ(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            SB(r1, r2, _i) => vec![r1, r2],
            SW(r1, r2, _i) => vec![r1, r2],
            BHSH(r1, r2) => vec![r1, r2],
            BHEI(r1) => vec![r1],
            BURN(r1) => vec![r1],
            CALL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CCP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CROO(r1, r2) => vec![r1, r2],
            CSIZ(r1, r2) => vec![r1, r2],
            CB(r1) => vec![r1],
            LDC(r1, r2, r3) => vec![r1, r2, r3],
            LOG(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            MINT(r1) => vec![r1],
            RVRT(r1) => vec![r1],
            SLDC(r1, r2, r3) => vec![r1, r2, r3],
            SRW(r1, r2) => vec![r1, r2],
            SRWQ(r1, r2) => vec![r1, r2],
            SWW(r1, r2) => vec![r1, r2],
            SWWQ(r1, r2) => vec![r1, r2],
            TR(r1, r2, r3) => vec![r1, r2, r3],
            TRO(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            ECR(r1, r2, r3) => vec![r1, r2, r3],
            K256(r1, r2, r3) => vec![r1, r2, r3],
            S256(r1, r2, r3) => vec![r1, r2, r3],
            NOOP => vec![],
            FLAG(r1) => vec![r1],
            Undefined => vec![],
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn allocate_registers(
        &self,
        pool: &mut RegisterPool,
        op_register_mapping: &[(Op, HashSet<VirtualRegister>)],
    ) -> AllocatedOp {
        let virtual_registers = self.registers();
        let register_allocation_result = virtual_registers
            .clone()
            .into_iter()
            .map(|x| (x, pool.get_register()))
            .map(|(x, res)| match res {
                Some(res) => Some((x, res)),
                None => None,
            })
            .collect::<Option<Vec<_>>>();

        // Maps virtual registers to their allocated equivalent
        let mut mapping = HashMap::default();
        match register_allocation_result {
            Some(o) => {
                for (key, val) in o {
                    mapping.insert(key, val);
                }
            }
            None => todo!("Out of registers error"),
        };

        for reg in virtual_registers {
            if virtual_register_is_never_accessed_again(reg, &op_register_mapping) {
                pool.return_register_to_pool(mapping.get(reg).unwrap().clone());
            }
        }

        use VirtualOp::*;
        match self {
            ADD(reg1, reg2, reg3) => AllocatedOp::ADD(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ADDI(reg1, reg2, imm) => AllocatedOp::ADDI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            AND(reg1, reg2, reg3) => AllocatedOp::AND(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ANDI(reg1, reg2, imm) => AllocatedOp::ANDI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            DIV(reg1, reg2, reg3) => AllocatedOp::DIV(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            DIVI(reg1, reg2, imm) => AllocatedOp::DIVI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            EQ(reg1, reg2, reg3) => AllocatedOp::EQ(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            EXP(reg1, reg2, reg3) => AllocatedOp::EXP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            EXPI(reg1, reg2, imm) => AllocatedOp::EXPI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            GT(reg1, reg2, reg3) => AllocatedOp::GT(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MLOG(reg1, reg2, reg3) => AllocatedOp::MLOG(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MROO(reg1, reg2, reg3) => AllocatedOp::MROO(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MOD(reg1, reg2, reg3) => AllocatedOp::MOD(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MODI(reg1, reg2, imm) => AllocatedOp::MODI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            MOVE(reg1, reg2) => AllocatedOp::MOVE(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            MUL(reg1, reg2, reg3) => AllocatedOp::MUL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MULI(reg1, reg2, imm) => AllocatedOp::MULI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            NOT(reg1, reg2) => AllocatedOp::NOT(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            OR(reg1, reg2, reg3) => AllocatedOp::OR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ORI(reg1, reg2, imm) => AllocatedOp::ORI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            SLL(reg1, reg2, reg3) => AllocatedOp::SLL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SLLI(reg1, reg2, imm) => AllocatedOp::SLLI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            SRL(reg1, reg2, reg3) => AllocatedOp::SRL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SRLI(reg1, reg2, imm) => AllocatedOp::SRLI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            SUB(reg1, reg2, reg3) => AllocatedOp::SUB(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SUBI(reg1, reg2, imm) => AllocatedOp::SUBI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            XOR(reg1, reg2, reg3) => AllocatedOp::XOR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            XORI(reg1, reg2, imm) => AllocatedOp::XORI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            CIMV(reg1, reg2, reg3) => AllocatedOp::CIMV(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            CTMV(reg1, reg2) => AllocatedOp::CTMV(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            JI(imm) => AllocatedOp::JI(imm.clone()),
            JNEI(reg1, reg2, imm) => AllocatedOp::JNEI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            RET(reg) => AllocatedOp::RET(map_reg(&mapping, reg)),
            CFEI(imm) => AllocatedOp::CFEI(imm.clone()),
            CFSI(imm) => AllocatedOp::CFSI(imm.clone()),
            LB(reg1, reg2, imm) => AllocatedOp::LB(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            LW(reg1, reg2, imm) => AllocatedOp::LW(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            ALOC(reg) => AllocatedOp::ALOC(map_reg(&mapping, reg)),
            MCL(reg1, reg2) => AllocatedOp::MCL(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            MCLI(reg1, imm) => AllocatedOp::MCLI(map_reg(&mapping, reg1), imm.clone()),
            MCP(reg1, reg2, reg3) => AllocatedOp::MCP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MEQ(reg1, reg2, reg3, reg4) => AllocatedOp::MEQ(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            SB(reg1, reg2, imm) => AllocatedOp::SB(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            SW(reg1, reg2, imm) => AllocatedOp::SW(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            BHSH(reg1, reg2) => AllocatedOp::BHSH(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            BHEI(reg1) => AllocatedOp::BHEI(map_reg(&mapping, reg1)),
            BURN(reg1) => AllocatedOp::BURN(map_reg(&mapping, reg1)),
            CALL(reg1, reg2, reg3, reg4) => AllocatedOp::CALL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            CCP(reg1, reg2, reg3, reg4) => AllocatedOp::CCP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            CROO(reg1, reg2) => AllocatedOp::CROO(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            CSIZ(reg1, reg2) => AllocatedOp::CSIZ(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            CB(reg1) => AllocatedOp::CB(map_reg(&mapping, reg1)),
            LDC(reg1, reg2, reg3) => AllocatedOp::LDC(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            LOG(reg1, reg2, reg3, reg4) => AllocatedOp::LOG(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            MINT(reg1) => AllocatedOp::MINT(map_reg(&mapping, reg1)),
            RVRT(reg1) => AllocatedOp::RVRT(map_reg(&mapping, reg1)),
            SLDC(reg1, reg2, reg3) => AllocatedOp::SLDC(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SRW(reg1, reg2) => AllocatedOp::SRW(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            SRWQ(reg1, reg2) => AllocatedOp::SRWQ(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            SWW(reg1, reg2) => AllocatedOp::SWW(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            SWWQ(reg1, reg2) => AllocatedOp::SWWQ(map_reg(&mapping, reg1), map_reg(&mapping, reg2)),
            TR(reg1, reg2, reg3) => AllocatedOp::TR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            TRO(reg1, reg2, reg3, reg4) => AllocatedOp::TRO(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            ECR(reg1, reg2, reg3) => AllocatedOp::ECR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            K256(reg1, reg2, reg3) => AllocatedOp::K256(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            S256(reg1, reg2, reg3) => AllocatedOp::S256(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            NOOP => AllocatedOp::NOOP,
            FLAG(reg) => AllocatedOp::FLAG(map_reg(&mapping, reg)),
            Undefined => AllocatedOp::Undefined,
        }
    }
}

/// An unchecked function which serves as a convenience for looking up register mappings
fn map_reg(
    mapping: &HashMap<&VirtualRegister, AllocatedRegister>,
    reg: &VirtualRegister,
) -> AllocatedRegister {
    mapping.get(reg).unwrap().clone()
}

#[derive(Clone, Eq, PartialEq)]
/// A label for a spot in the bytecode, to be later compiled to an offset.
pub(crate) struct Label(pub(crate) usize);
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}", self.0)
    }
}
fn virtual_register_is_never_accessed_again(
    reg: &VirtualRegister,
    ops: &[(Op, std::collections::HashSet<VirtualRegister>)],
) -> bool {
    !ops.iter().any(|(_, regs)| regs.contains(reg))
}
