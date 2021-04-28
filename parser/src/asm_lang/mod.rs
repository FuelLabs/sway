//! This module contains things that I need from the VM to build that we will eventually import
//! from the VM when it is ready.
//! Basically this is copy-pasted until things are public and it can be properly imported.
//!
//! Only things needed for opcode serialization and generation are included here.
#![allow(dead_code)]

use crate::{error::*, parse_tree::AsmRegister, Ident};
use either::Either;
use pest::Span;
use std::collections::HashSet;

#[macro_export]
macro_rules! opcodes {
    (
        $(
            $op:ident ( $($inits:ident),* ) = $val:expr
        ),+
    ) => {
        #[derive( Clone, PartialEq, Debug)]
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

impl From<&AsmRegister> for RegisterId {
    fn from(o: &AsmRegister) -> Self {
        o.name.clone()
    }
}

pub(crate) struct Op<'sc> {
    pub(crate) opcode: Either<Opcode, OrganizationalOp>,
    /// A descriptive comment for debugging
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span<'sc>>,
}

impl<'sc> Op<'sc> {
    pub(crate) fn new(opcode: Opcode, owning_span: Span<'sc>) -> Self {
        Op {
            opcode: Either::Left(opcode),
            comment: String::new(),
            owning_span: Some(owning_span),
        }
    }
    pub(crate) fn new_with_comment(
        opcode: Opcode,
        owning_span: Span<'sc>,
        comment: impl Into<String>,
    ) -> Self {
        let comment = comment.into();
        Op {
            opcode: Either::Left(opcode),
            comment,
            owning_span: Some(owning_span),
        }
    }

    /// Given a label, creates the actual asm line to put in the ASM which represents a label
    pub(crate) fn jump_label(label: Label, owning_span: Span<'sc>) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Label(label)),
            comment: String::new(),
            owning_span: Some(owning_span),
        }
    }

    /// Given a label, creates the actual asm line to put in the ASM which represents a label
    pub(crate) fn unowned_jump_label(label: Label) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Label(label)),
            comment: String::new(),
            owning_span: None,
        }
    }

    /// Moves the register in the second argument into the register in the first argument
    pub(crate) fn register_move(r1: RegisterId, r2: RegisterId, owning_span: Span<'sc>) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::RMove(r1, r2)),
            comment: String::new(),
            owning_span: Some(owning_span),
        }
    }

    pub(crate) fn new_comment(comm: impl Into<String>) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Comment),
            comment: comm.into(),
            owning_span: None,
        }
    }

    /// Generates a new label with a UUID (in base 64) for convenience.
    pub(crate) fn new_label() -> Label {
        Label(uuid_b64::UuidB64::new())
    }

    pub(crate) fn jump_to_label(label: Label) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Jump(label)),
            comment: String::new(),
            owning_span: None,
        }
    }

    pub(crate) fn parse_opcode(
        name: &Ident<'sc>,
        args: &[&AsmRegister],
        immediate: Option<ImmediateValue>,
    ) -> CompileResult<'sc, Opcode> {
        Opcode::parse(name, args, immediate)
    }
}

impl Opcode {
    /// If this name matches an opcode and there are the correct number and
    /// type of arguments, parse the given inputs into an opcode.
    pub(crate) fn parse<'sc>(
        name: &Ident<'sc>,
        args: &[&AsmRegister],
        immediate: Option<ImmediateValue>,
    ) -> CompileResult<'sc, Opcode> {
        use Opcode::*;
        let op = match name.primary_name {
            "add" => {
                if args.len() == 3 {
                    Add(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "addi" => {
                if args.len() == 3 {
                    Addi(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "and" => {
                if args.len() == 3 {
                    And(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "andi" => {
                if args.len() == 3 {
                    Andi(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "div" => {
                if args.len() == 3 {
                    Div(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "divi" => {
                if args.len() == 3 {
                    Divi(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "mod" => {
                if args.len() == 3 {
                    Mod(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "modi" => {
                if args.len() == 3 {
                    Modi(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "eq" => {
                if args.len() == 3 {
                    Eq(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "gt" => {
                if args.len() == 3 {
                    Gt(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "mult" => {
                if args.len() == 3 {
                    Mult(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "multi" => {
                if args.len() == 3 {
                    Multi(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
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
                    Not(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "or" => {
                if args.len() == 3 {
                    Or(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ori" => {
                if args.len() == 3 {
                    Ori(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sll" => {
                if args.len() == 3 {
                    Sll(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sllv" => {
                if args.len() == 3 {
                    Sllv(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sltiu" => {
                if args.len() == 3 {
                    Sltiu(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sltu" => {
                if args.len() == 3 {
                    Sltu(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sra" => {
                if args.len() == 3 {
                    Sra(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srl" => {
                if args.len() == 3 {
                    Srl(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srlv" => {
                if args.len() == 3 {
                    Srlv(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srav" => {
                if args.len() == 3 {
                    Srav(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sub" => {
                if args.len() == 3 {
                    Sub(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "subi" => {
                if args.len() == 3 {
                    Subi(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "xor" => {
                if args.len() == 3 {
                    Xor(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "xori" => {
                if args.len() == 3 {
                    Xori(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "exp" => {
                if args.len() == 3 {
                    Exp(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "expi" => {
                if args.len() == 3 {
                    Expi(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "cimv" => {
                if args.len() == 3 {
                    CIMV(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ctmv" => {
                if args.len() == 2 {
                    CTMV(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ji" => {
                if args.len() == 1 {
                    Ji(match immediate {
                        Some(i) => i,
                        None => {
                            return err(
                                vec![],
                                vec![CompileError::MissingImmediate {
                                    span: name.span.clone(),
                                }],
                            )
                        }
                    })
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "jnzi" => {
                if args.len() == 2 {
                    Jnzi(
                        args[0].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ret" => {
                if args.len() == 1 {
                    Ret(args[0].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "cfe" => {
                if args.len() == 1 {
                    Cfe(args[0].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "cfs" => {
                if args.len() == 1 {
                    Cfs(args[0].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "lb" => {
                if args.len() == 3 {
                    Lb(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "lw" => {
                if args.len() == 3 {
                    Lw(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "malloc" => {
                if args.len() == 1 {
                    Malloc(args[0].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "memclear" => {
                if args.len() == 2 {
                    MemClear(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "memcp" => {
                if args.len() == 2 {
                    MemCp(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "memeq" => {
                if args.len() == 4 {
                    MemEq(
                        args[0].into(),
                        args[1].into(),
                        args[2].into(),
                        args[3].into(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sb" => {
                if args.len() == 3 {
                    Sb(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sw" => {
                if args.len() == 3 {
                    Sw(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "blockhash" => {
                if args.len() == 2 {
                    BlockHash(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "blockheight" => {
                if args.len() == 1 {
                    BlockHeight(args[0].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "call" => {
                if args.len() == 4 {
                    Call(
                        args[0].into(),
                        args[1].into(),
                        args[2].into(),
                        args[3].into(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "codecopy" => {
                if args.len() == 3 {
                    CodeCopy(
                        args[0].into(),
                        args[1].into(),
                        match immediate {
                            Some(i) => i,
                            None => {
                                return err(
                                    vec![],
                                    vec![CompileError::MissingImmediate {
                                        span: name.span.clone(),
                                    }],
                                )
                            }
                        },
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "coderoot" => {
                if args.len() == 2 {
                    CodeRoot(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "codesize" => {
                if args.len() == 2 {
                    Codesize(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "coinbase" => {
                if args.len() == 1 {
                    Coinbase(args[0].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "loadcode" => {
                if args.len() == 3 {
                    LoadCode(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sloadcode" => {
                if args.len() == 3 {
                    SLoadCode(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "log" => {
                if args.len() == 4 {
                    Log(
                        args[0].into(),
                        args[1].into(),
                        args[2].into(),
                        args[3].into(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "revert" => {
                if args.len() == 1 {
                    Revert(args[0].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srw" => {
                if args.len() == 2 {
                    Srw(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "srwx" => {
                if args.len() == 2 {
                    Srwx(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sww" => {
                if args.len() == 2 {
                    Sww(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "swwx" => {
                if args.len() == 2 {
                    Swwx(args[0].into(), args[1].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "transfer" => {
                if args.len() == 3 {
                    Transfer(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "transferout" => {
                if args.len() == 4 {
                    TransferOut(
                        args[0].into(),
                        args[1].into(),
                        args[2].into(),
                        args[3].into(),
                    )
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "ecrecover" => {
                if args.len() == 3 {
                    Ecrecover(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "keccak256" => {
                if args.len() == 3 {
                    Keccak256(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "sha256" => {
                if args.len() == 3 {
                    Sha256(args[0].into(), args[1].into(), args[2].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            "flag" => {
                if args.len() == 1 {
                    Flag(args[0].into())
                } else {
                    todo!("ArgMismatchError")
                }
            }
            _ => todo!("unknown op error"),
        };
        ok(op, vec![], vec![])
    }
    pub(crate) fn get_register_names(&self) -> HashSet<AsmRegister> {
        use Opcode::*;
        let regs: Vec<&String> = match self {
            Add(r1, r2, r3) => vec![r1, r2, r3],
            Addi(r1, r2, _imm) => vec![r1, r2],
            And(r1, r2, r3) => vec![r1, r2, r3],
            Andi(r1, r2, _imm) => vec![r1, r2],
            Div(r1, r2, r3) => vec![r1, r2, r3],
            Divi(r1, r2, _imm) => vec![r1, r2],
            Mod(r1, r2, r3) => vec![r1, r2, r3],
            Modi(r1, r2, _imm) => vec![r1, r2],
            Eq(r1, r2, r3) => vec![r1, r2, r3],
            Gt(r1, r2, r3) => vec![r1, r2, r3],
            Mult(r1, r2, r3) => vec![r1, r2, r3],
            Multi(r1, r2, _imm) => vec![r1, r2],
            Noop() => vec![],
            Not(r1, r2) => vec![r1, r2],
            Or(r1, r2, r3) => vec![r1, r2, r3],
            Ori(r1, r2, _imm) => vec![r1, r2],
            Sll(r1, r2, _imm) => vec![r1, r2],
            Sllv(r1, r2, r3) => vec![r1, r2, r3],
            Sltiu(r1, r2, _imm) => vec![r1, r2],
            Sltu(r1, r2, r3) => vec![r1, r2, r3],
            Sra(r1, r2, _imm) => vec![r1, r2],
            Srl(r1, r2, _imm) => vec![r1, r2],
            Srlv(r1, r2, r3) => vec![r1, r2, r3],
            Srav(r1, r2, r3) => vec![r1, r2, r3],
            Sub(r1, r2, r3) => vec![r1, r2, r3],
            Subi(r1, r2, _imm) => vec![r1, r2],
            Xor(r1, r2, r3) => vec![r1, r2, r3],
            Xori(r1, r2, _imm) => vec![r1, r2],
            Exp(r1, r2, r3) => vec![r1, r2, r3],
            Expi(r1, r2, _imm) => vec![r1, r2],
            CIMV(r1, r2, _imm) => vec![r1, r2],
            CTMV(r1, r2) => vec![r1, r2],
            Ji(_imm) => vec![],
            Jnzi(r1, _imm) => vec![r1],
            Ret(r1) => vec![r1],
            Cfe(r1) => vec![r1],
            Cfs(r1) => vec![r1],
            Lb(r1, r2, _imm) => vec![r1, r2],
            Lw(r1, r2, _imm) => vec![r1, r2],
            Malloc(r1) => vec![r1],
            MemClear(r1, r2) => vec![r1, r2],
            MemCp(r1, r2, r3) => vec![r1, r2, r3],
            MemEq(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            Sb(r1, r2, _imm) => vec![r1, r2],
            Sw(r1, r2, _imm) => vec![r1, r2],
            BlockHash(r1, r2) => vec![r1, r2],
            BlockHeight(r1) => vec![r1],
            Call(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CodeCopy(r1, r2, _imm) => vec![r1, r2],
            CodeRoot(r1, r2) => vec![r1, r2],
            Codesize(r1, r2) => vec![r1, r2],
            Coinbase(r1) => vec![r1],
            LoadCode(r1, r2, r3) => vec![r1, r2, r3],
            SLoadCode(r1, r2, r3) => vec![r1, r2, r3],
            Log(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            Revert(r1) => vec![r1],
            Srw(r1, r2) => vec![r1, r2],
            Srwx(r1, r2) => vec![r1, r2],
            Sww(r1, r2) => vec![r1, r2],
            Swwx(r1, r2) => vec![r1, r2],
            Transfer(r1, r2, r3) => vec![r1, r2, r3],
            TransferOut(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            Ecrecover(r1, r2, r3) => vec![r1, r2, r3],
            Keccak256(r1, r2, r3) => vec![r1, r2, r3],
            Sha256(r1, r2, r3) => vec![r1, r2, r3],
            Flag(r1) => vec![r1],
        };

        regs.into_iter()
            .map(|x| AsmRegister {
                name: x.to_string(),
            })
            .collect()
    }
}

// internal representation for register ids
// simpler to represent as usize since it avoids casts
pub type RegisterId = String;

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

#[derive(Clone)]
pub(crate) struct Label(uuid_b64::UuidB64);

// Convenience opcodes for the compiler -- will be optimized out or removed
// these do not reflect actual ops in the VM
pub(crate) enum OrganizationalOp {
    // copies the second register into the first register
    RMove(RegisterId, RegisterId),
    // Labels the code for jumps, will later be interpreted into offsets
    Label(Label),
    // Just a comment that will be inserted into the asm without an op
    Comment,
    // Jumps to a label
    Jump(Label),
}
