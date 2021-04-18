//! This module contains things that I need from the VM to build that we will eventually import
//! from the VM when it is ready.
//! Basically this is copy-pasted until things are public and it can be properly imported.
//!
//! Only things needed for opcode serialization and generation are included here.

#![allow(dead_code)]

#[macro_export]
macro_rules! opcodes {
    (
        $(
            $op:ident ( $($inits:ident),* ) = $val:expr
        ),+
    ) => {
        #[derive(Copy, Clone, PartialEq, Debug)]
        pub enum Opcode {
            $(
                #[warn(unused_must_use)]
                $op( $($inits),* ),
            )+
        }

        $(
            #[allow(non_upper_case_globals)]
            const $op:u32 = $val;
        )+

    }
}

impl Opcode {
    /// If this name matches an opcode and there are the correct number and
    /// type of arguments, parse the given inputs into an opcode.
    pub(crate) fn parse(name: &str, args: &[&str]) -> Result<Opcode, ()> {
        use Opcode::*;
        let op = match name {
            "add" => {
                if args.len() == 3 {
                    Add(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "addi" => {
                if args.len() == 3 {
                    Addi(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "and" => {
                if args.len() == 3 {
                    And(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "andi" => {
                if args.len() == 3 {
                    Andi(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "div" => {
                if args.len() == 3 {
                    Div(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "divi" => {
                if args.len() == 3 {
                    Divi(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "mod" => {
                if args.len() == 3 {
                    Mod(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "modi" => {
                if args.len() == 3 {
                    Modi(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "eq" => {
                if args.len() == 3 {
                    Eq(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "gt" => {
                if args.len() == 3 {
                    Gt(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "mult" => {
                if args.len() == 3 {
                    Mult(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "multi" => {
                if args.len() == 3 {
                    Multi(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "noop" => {
                if args.len() == 0 {
                    Noop()
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "not" => {
                if args.len() == 2 {
                    Not(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "or" => {
                if args.len() == 3 {
                    Or(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ori" => {
                if args.len() == 3 {
                    Ori(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sll" => {
                if args.len() == 3 {
                    Sll(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sllv" => {
                if args.len() == 3 {
                    Sllv(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sltiu" => {
                if args.len() == 3 {
                    Sltiu(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sltu" => {
                if args.len() == 3 {
                    Sltu(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sra" => {
                if args.len() == 3 {
                    Sra(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srl" => {
                if args.len() == 3 {
                    Srl(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srlv" => {
                if args.len() == 3 {
                    Srlv(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srav" => {
                if args.len() == 3 {
                    Srav(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sub" => {
                if args.len() == 3 {
                    Sub(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "subi" => {
                if args.len() == 3 {
                    Subi(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "xor" => {
                if args.len() == 3 {
                    Xor(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "xori" => {
                if args.len() == 3 {
                    Xori(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "exp" => {
                if args.len() == 3 {
                    Exp(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "expi" => {
                if args.len() == 3 {
                    Expi(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "cimv" => {
                if args.len() == 3 {
                    CIMV(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ctmv" => {
                if args.len() == 2 {
                    CTMV(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ji" => {
                if args.len() == 1 {
                    Ji(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "jnzi" => {
                if args.len() == 2 {
                    Jnzi(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ret" => {
                if args.len() == 1 {
                    Ret(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "cfe" => {
                if args.len() == 1 {
                    Cfe(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "cfs" => {
                if args.len() == 1 {
                    Cfs(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "lb" => {
                if args.len() == 3 {
                    Lb(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "lw" => {
                if args.len() == 3 {
                    Lw(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "malloc" => {
                if args.len() == 1 {
                    Malloc(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "memclear" => {
                if args.len() == 2 {
                    MemClear(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "memcp" => {
                if args.len() == 2 {
                    MemCp(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "memeq" => {
                if args.len() == 4 {
                    MemEq(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                        args[3].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sb" => {
                if args.len() == 3 {
                    Sb(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sw" => {
                if args.len() == 3 {
                    Sw(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "blockhash" => {
                if args.len() == 2 {
                    BlockHash(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "blockheight" => {
                if args.len() == 1 {
                    BlockHeight(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "call" => {
                if args.len() == 4 {
                    Call(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                        args[3].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "codecopy" => {
                if args.len() == 3 {
                    CodeCopy(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "coderoot" => {
                if args.len() == 2 {
                    CodeRoot(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "codesize" => {
                if args.len() == 2 {
                    Codesize(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "coinbase" => {
                if args.len() == 1 {
                    Coinbase(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "loadcode" => {
                if args.len() == 3 {
                    LoadCode(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sloadcode" => {
                if args.len() == 3 {
                    SLoadCode(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "log" => {
                if args.len() == 4 {
                    Log(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                        args[3].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "revert" => {
                if args.len() == 1 {
                    Revert(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srw" => {
                if args.len() == 2 {
                    Srw(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srwx" => {
                if args.len() == 2 {
                    Srwx(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sww" => {
                if args.len() == 2 {
                    Sww(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "swwx" => {
                if args.len() == 2 {
                    Swwx(args[0].parse().unwrap(), args[1].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "transfer" => {
                if args.len() == 3 {
                    Transfer(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "transferout" => {
                if args.len() == 4 {
                    TransferOut(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                        args[3].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ecrecover" => {
                if args.len() == 3 {
                    Ecrecover(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "keccak256" => {
                if args.len() == 3 {
                    Keccak256(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sha256" => {
                if args.len() == 3 {
                    Sha256(
                        args[0].parse().unwrap(),
                        args[1].parse().unwrap(),
                        args[2].parse().unwrap(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "flag" => {
                if args.len() == 1 {
                    Flag(args[0].parse().unwrap())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            _ => todo!("unknown op error"),
        };
        Ok(op)
    }
}

// internal representation for register ids
// simpler to represent as usize since it avoids casts
pub type RegisterId = u8;

// Immediate Value.
pub type ImmediateValue = u32;

opcodes! {
    // Arithmetic and Logic.
    Add(RegisterId, RegisterId, RegisterId) = 0,
    Addi(RegisterId, RegisterId, ImmediateValue) = 1,
    And(RegisterId, RegisterId, RegisterId) = 2,
    Andi(RegisterId, RegisterId, ImmediateValue) = 3,
    Div(RegisterId, RegisterId, RegisterId) = 4,
    Divi(RegisterId, RegisterId, ImmediateValue) = 5,
    Mod(RegisterId, RegisterId, RegisterId) = 6,
    Modi(RegisterId, RegisterId, ImmediateValue) = 7,
    Eq(RegisterId, RegisterId, RegisterId) = 8,
    Gt(RegisterId, RegisterId, RegisterId) = 9,
    Mult(RegisterId, RegisterId, RegisterId) = 10,
    Multi(RegisterId, RegisterId, ImmediateValue) = 11,
    Noop() = 12,
    Not(RegisterId, RegisterId) = 13,
    Or(RegisterId, RegisterId, RegisterId) = 14,
    Ori(RegisterId, RegisterId, ImmediateValue) = 15,
    Sll(RegisterId, RegisterId, ImmediateValue) = 16,
    Sllv(RegisterId, RegisterId, RegisterId) = 17,
    Sltiu(RegisterId, RegisterId, ImmediateValue) = 18,
    Sltu(RegisterId, RegisterId, RegisterId) = 19,
    Sra(RegisterId, RegisterId, ImmediateValue) = 20,
    Srl(RegisterId, RegisterId, ImmediateValue) = 21,
    Srlv(RegisterId, RegisterId, RegisterId) = 22,
    Srav(RegisterId, RegisterId, RegisterId) = 23,
    Sub(RegisterId, RegisterId, RegisterId) = 24,
    Subi(RegisterId, RegisterId, ImmediateValue) = 25,
    Xor(RegisterId, RegisterId, RegisterId) = 26,
    Xori(RegisterId, RegisterId, ImmediateValue) = 27,
    Exp(RegisterId, RegisterId, RegisterId) = 28,
    Expi(RegisterId, RegisterId, ImmediateValue) = 29,

    // Control Flow Opcodes.
    CIMV(RegisterId, RegisterId, ImmediateValue) = 50,
    CTMV(RegisterId, RegisterId) = 51,
    Ji(ImmediateValue) = 52,
    Jnzi(RegisterId, ImmediateValue) = 53,
    Ret(RegisterId) = 54,

    // Memory opcodes.
    Cfe(RegisterId) = 60,
    Cfs(RegisterId) = 61,
    Lb(RegisterId, RegisterId, ImmediateValue) = 62,
    Lw(RegisterId, RegisterId, ImmediateValue) = 63,
    Malloc(RegisterId) = 64,
    MemClear(RegisterId, RegisterId) = 65,
    MemCp(RegisterId, RegisterId, RegisterId) = 66,
    MemEq(RegisterId, RegisterId, RegisterId, RegisterId) = 67,
    Sb(RegisterId, RegisterId, ImmediateValue) = 68,
    Sw(RegisterId, RegisterId, ImmediateValue) = 69,

    // Contract Opcodes.
    BlockHash(RegisterId, RegisterId) = 80,
    BlockHeight(RegisterId) = 81,
    Call(RegisterId, RegisterId, RegisterId, RegisterId) = 82,
    CodeCopy(RegisterId, RegisterId, ImmediateValue) = 83,
    CodeRoot(RegisterId, RegisterId) = 84,
    Codesize(RegisterId, RegisterId) = 85,
    Coinbase(RegisterId) = 86,
    LoadCode(RegisterId, RegisterId, RegisterId) = 87,
    SLoadCode(RegisterId, RegisterId, RegisterId) = 88,
    Log(RegisterId, RegisterId, RegisterId, RegisterId) = 89,
    Revert(RegisterId) = 90,
    Srw(RegisterId, RegisterId) = 91,
    Srwx(RegisterId, RegisterId) = 92,
    Sww(RegisterId, RegisterId) = 93,
    Swwx(RegisterId, RegisterId) = 94,
    Transfer(RegisterId, RegisterId, RegisterId) = 95,
    TransferOut(RegisterId, RegisterId, RegisterId, RegisterId) = 96,

    // Cryptographic Opcodes.
    Ecrecover(RegisterId, RegisterId, RegisterId) = 110,
    Keccak256(RegisterId, RegisterId, RegisterId) = 111,
    Sha256(RegisterId, RegisterId, RegisterId) = 112,

    // Additional Opcodes.
    Flag(RegisterId) = 130
}
