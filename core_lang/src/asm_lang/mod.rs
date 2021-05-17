//! This module contains things that I need from the VM to build that we will eventually import
//! from the VM when it is ready.
//! Basically this is copy-pasted until things are public and it can be properly imported.
//!
//! Only things needed for opcode serialization and generation are included here.
#![allow(dead_code)]

use crate::{asm_generation::DataId, error::*, parse_tree::AsmRegister, Ident};
use either::Either;
use pest::Span;
use std::{collections::HashSet, fmt};

/// The column where the ; for comments starts
const COMMENT_START_COLUMN: usize = 40;

#[macro_export]
macro_rules! opcodes {
    (
        $(
            $op:ident ( $($inits:ident),* ) = $val:expr
        ),+
    ) => {
        #[derive(Clone, PartialEq, Debug)]
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
        RegisterId::Virtual(o.name.clone())
    }
}

#[derive(Clone)]
pub(crate) struct Op<'sc> {
    pub(crate) opcode: Either<Opcode, OrganizationalOp>,
    /// A descriptive comment for debugging
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span<'sc>>,
}

impl<'sc> Op<'sc> {
    /// Write value in given [RegisterId] `value_to_write` to given memory address that is held within the
    /// [RegisterId] `destination_address`
    pub(crate) fn write_register_to_memory(
        destination_address: RegisterId,
        value_to_write: RegisterId,
        offset: ImmediateValue,
        span: Span<'sc>,
    ) -> Self {
        Op {
            opcode: Either::Left(Opcode::Sw(destination_address, value_to_write, offset)),
            comment: String::new(),
            owning_span: Some(span),
        }
    }
    /// Write value in given [RegisterId] `value_to_write` to given memory address that is held within the
    /// [RegisterId] `destination_address`, with the provided comment.
    pub(crate) fn write_register_to_memory_comment(
        destination_address: RegisterId,
        value_to_write: RegisterId,
        offset: ImmediateValue,
        span: Span<'sc>,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Left(Opcode::Sw(destination_address, value_to_write, offset)),
            comment: comment.into(),
            owning_span: Some(span),
        }
    }
    /// Moves the stack pointer by the given amount (i.e. allocates stack memory)
    pub(crate) fn unowned_stack_allocate_memory(size_to_allocate_in_words: u32) -> Self {
        Op {
            opcode: Either::Left(Opcode::Cfei(size_to_allocate_in_words)),
            comment: String::new(),
            owning_span: None,
        }
    }
    pub(crate) fn unowned_new_with_comment(opcode: Opcode, comment: impl Into<String>) -> Self {
        Op {
            opcode: Either::Left(opcode),
            comment: comment.into(),
            owning_span: None,
        }
    }
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
    /// Loads the data from [DataId] `data` into [RegisterId] `reg`.
    pub(crate) fn unowned_load_data_comment(
        reg: RegisterId,
        data: DataId,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Ld(reg, data)),
            comment: comment.into(),
            owning_span: None,
        }
    }

    /// Given a label, creates the actual asm line to put in the ASM which represents a label.
    /// Also attaches a comment to it.
    pub(crate) fn unowned_jump_label_comment(label: Label, comment: impl Into<String>) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Label(label)),
            comment: comment.into(),
            owning_span: None,
        }
    }

    /// Given a label, creates the actual asm line to put in the ASM which represents a label.
    /// Also attaches a comment to it.
    pub(crate) fn jump_label_comment(
        label: Label,
        owning_span: Span<'sc>,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Label(label)),
            comment: comment.into(),
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

    /// Moves the register in the second argument into the register in the first argument
    pub(crate) fn unowned_register_move(r1: RegisterId, r2: RegisterId) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::RMove(r1, r2)),
            comment: String::new(),
            owning_span: None,
        }
    }
    pub(crate) fn register_move_comment(
        r1: RegisterId,
        r2: RegisterId,
        owning_span: Span<'sc>,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::RMove(r1, r2)),
            comment: comment.into(),
            owning_span: Some(owning_span),
        }
    }

    /// Moves the register in the second argument into the register in the first argument
    pub(crate) fn unowned_register_move_comment(
        r1: RegisterId,
        r2: RegisterId,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::RMove(r1, r2)),
            comment: comment.into(),
            owning_span: None,
        }
    }

    pub(crate) fn new_comment(comm: impl Into<String>) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Comment),
            comment: comm.into(),
            owning_span: None,
        }
    }

    pub(crate) fn jump_to_label(label: Label) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Jump(label)),
            comment: String::new(),
            owning_span: None,
        }
    }

    pub(crate) fn jump_to_label_comment(label: Label, comment: impl Into<String>) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Jump(label)),
            comment: comment.into(),
            owning_span: None,
        }
    }

    /// Jumps to [Label] `label`  if the given [RegisterId] `reg1` is not equal to `reg0`.
    pub(crate) fn jump_if_not_equal(reg0: RegisterId, reg1: RegisterId, label: Label) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::JumpIfNotEq(reg0, reg1, label)),
            comment: String::new(),
            owning_span: None,
        }
    }

    pub(crate) fn parse_opcode(
        name: &Ident<'sc>,
        args: &[&RegisterId],
        immediate: Option<ImmediateValue>,
    ) -> CompileResult<'sc, Opcode> {
        Opcode::parse(name, args, immediate)
    }
}

impl Into<RegisterId> for &RegisterId {
    fn into(self) -> RegisterId {
        self.clone()
    }
}

impl Opcode {
    /// If this name matches an opcode and there are the correct number and
    /// type of arguments, parse the given inputs into an opcode.
    pub(crate) fn parse<'sc>(
        name: &Ident<'sc>,
        args: &[&RegisterId],
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
            "cfei" => {
                if args.len() == 1 {
                    Cfei(match immediate {
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
            "memcleari" => {
                if args.len() == 2 {
                    MemClearImmediate(
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
            other => {
                return err(
                    vec![],
                    vec![CompileError::UnrecognizedOp {
                        op_name: other,
                        span: name.clone().span,
                    }],
                )
            }
        };
        ok(op, vec![], vec![])
    }
    pub(crate) fn registers(&mut self) -> HashSet<&mut RegisterId> {
        use Opcode::*;
        let regs: Vec<&mut RegisterId> = match self {
            Add(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Addi(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            And(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Andi(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Div(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Divi(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Mod(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Modi(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Eq(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Gt(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Mult(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Multi(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Noop() => vec![],
            Not(ref mut r1, ref mut r2) => vec![r1, r2],
            Or(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Ori(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Sll(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Sllv(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Sltiu(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Sltu(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Sra(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Srl(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Srlv(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Srav(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Sub(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Subi(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Xor(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Xori(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Exp(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Expi(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            CIMV(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            CTMV(ref mut r1, ref mut r2) => vec![r1, r2],
            Ji(_imm) => vec![],
            Jnzi(ref mut r1, _imm) => vec![r1],
            Ret(ref mut r1) => vec![r1],
            Cfei(_imm) => vec![],
            Cfs(ref mut r1) => vec![r1],
            Lb(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Lw(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Malloc(ref mut r1) => vec![r1],
            MemClearImmediate(ref mut r1, _imm) => vec![r1],
            MemCp(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            MemEq(ref mut r1, ref mut r2, ref mut r3, ref mut r4) => vec![r1, r2, r3, r4],
            Sb(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            Sw(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            BlockHash(ref mut r1, ref mut r2) => vec![r1, r2],
            BlockHeight(ref mut r1) => vec![r1],
            Call(ref mut r1, ref mut r2, ref mut r3, ref mut r4) => vec![r1, r2, r3, r4],
            CodeCopy(ref mut r1, ref mut r2, _imm) => vec![r1, r2],
            CodeRoot(ref mut r1, ref mut r2) => vec![r1, r2],
            Codesize(ref mut r1, ref mut r2) => vec![r1, r2],
            Coinbase(ref mut r1) => vec![r1],
            LoadCode(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            SLoadCode(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Log(ref mut r1, ref mut r2, ref mut r3, ref mut r4) => vec![r1, r2, r3, r4],
            Revert(ref mut r1) => vec![r1],
            Srw(ref mut r1, ref mut r2) => vec![r1, r2],
            Srwx(ref mut r1, ref mut r2) => vec![r1, r2],
            Sww(ref mut r1, ref mut r2) => vec![r1, r2],
            Swwx(ref mut r1, ref mut r2) => vec![r1, r2],
            Transfer(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            TransferOut(ref mut r1, ref mut r2, ref mut r3, ref mut r4) => vec![r1, r2, r3, r4],
            Ecrecover(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Keccak256(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Sha256(ref mut r1, ref mut r2, ref mut r3) => vec![r1, r2, r3],
            Flag(ref mut r1) => vec![r1],
        };

        regs.into_iter().collect()
    }
}

// internal representation for register ids
// simpler to represent as usize since it avoids casts
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum RegisterId {
    Virtual(String),
    Constant(ConstantRegister),
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
    Cfei(ImmediateValue) = 60,
    Cfs(RegisterId) = 61,
    Lb(RegisterId, RegisterId, ImmediateValue) = 62,
    Lw(RegisterId, RegisterId, ImmediateValue) = 63,
    Malloc(RegisterId) = 64,
    MemClearImmediate(RegisterId, ImmediateValue) = 65,
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
impl fmt::Display for RegisterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterId::Virtual(name) => write!(f, "$r{}", name),
            RegisterId::Constant(name) => {
                write!(f, "{}", name)
            }
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}", self.0)
    }
}

impl fmt::Display for Op<'_> {
    // very clunky but lets us tweak assembly language most easily
    // below code was constructed with vim macros -- easier to regenerate rather than rewrite.
    // @alex if you want to change the format and save yourself the pain.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Opcode::*;
        use OrganizationalOp::*;
        let op_str = match &self.opcode {
            Either::Left(opcode) => match opcode {
                Add(a, b, c) => format!("add {} {} {}", a, b, c),
                Addi(a, b, c) => format!("addi {} {} {}", a, b, c),
                And(a, b, c) => format!("and {} {} {}", a, b, c),
                Andi(a, b, c) => format!("andi {} {} {}", a, b, c),
                Div(a, b, c) => format!("div {} {} {}", a, b, c),
                Divi(a, b, c) => format!("divi {} {} {}", a, b, c),
                Mod(a, b, c) => format!("mod {} {} {}", a, b, c),
                Modi(a, b, c) => format!("modi {} {} {}", a, b, c),
                Eq(a, b, c) => format!("eq {} {} {}", a, b, c),
                Gt(a, b, c) => format!("gt {} {} {}", a, b, c),
                Mult(a, b, c) => format!("mult {} {} {}", a, b, c),
                Multi(a, b, c) => format!("multi {} {} {}", a, b, c),
                Noop() => "noop".to_string(),
                Not(a, b) => format!("not {} {}", a, b),
                Or(a, b, c) => format!("or {} {} {}", a, b, c),
                Ori(a, b, c) => format!("ori {} {} {}", a, b, c),
                Sll(a, b, c) => format!("sll {} {} {}", a, b, c),
                Sllv(a, b, c) => format!("sllv {} {} {}", a, b, c),
                Sltiu(a, b, c) => format!("sltiu {} {} {}", a, b, c),

                Sltu(a, b, c) => format!("sltu {} {} {}", a, b, c),
                Sra(a, b, c) => format!("sra {} {} {}", a, b, c),
                Srl(a, b, c) => format!("srl {} {} {}", a, b, c),
                Srlv(a, b, c) => format!("srlv {} {} {}", a, b, c),
                Srav(a, b, c) => format!("srav {} {} {}", a, b, c),
                Sub(a, b, c) => format!("sub {} {} {}", a, b, c),
                Subi(a, b, c) => format!("subi {} {} {}", a, b, c),
                Xor(a, b, c) => format!("xor {} {} {}", a, b, c),
                Xori(a, b, c) => format!("xori {} {} {}", a, b, c),
                Exp(a, b, c) => format!("exp {} {} {}", a, b, c),
                Expi(a, b, c) => format!("expi {} {} {}", a, b, c),

                CIMV(a, b, c) => format!("cimv {} {} {}", a, b, c),
                CTMV(a, b) => format!("ctmv {} {}", a, b),
                Ji(a) => format!("ji {}", a),
                Jnzi(a, b) => format!("jnzi {} {}", a, b),
                Ret(a) => format!("ret {}", a),

                Cfei(a) => format!("cfei {}", a),
                Cfs(a) => format!("cfs {}", a),
                Lb(a, b, c) => format!("lb {} {} {}", a, b, c),
                Lw(a, b, c) => format!("lw {} {} {}", a, b, c),
                Malloc(a) => format!("malloc {}", a),
                MemClearImmediate(a, b) => format!("memcleari {} {}", a, b),
                MemCp(a, b, c) => format!("memcp {} {} {}", a, b, c),
                MemEq(a, b, c, d) => format!("memeq {} {} {} {}", a, b, c, d),
                Sb(a, b, c) => format!("sb {} {} {}", a, b, c),
                Sw(a, b, c) => format!("sw {} {} {}", a, b, c),

                BlockHash(a, b) => format!("blockhash {} {}", a, b),
                BlockHeight(a) => format!("blockheight {}", a),
                Call(a, b, c, d) => format!("call {} {} {} {}", a, b, c, d),
                CodeCopy(a, b, c) => format!("codecopy {} {} {}", a, b, c),
                CodeRoot(a, b) => format!("coderoot {} {}", a, b),
                Codesize(a, b) => format!("codesize {} {}", a, b),
                Coinbase(a) => format!("coinbase {}", a),
                LoadCode(a, b, c) => format!("loadcode {} {} {}", a, b, c),
                SLoadCode(a, b, c) => format!("sloadcode {} {} {}", a, b, c),
                Log(a, b, c, d) => format!("log {} {} {} {}", a, b, c, d),
                Revert(a) => format!("revert {}", a),
                Srw(a, b) => format!("srw {} {}", a, b),
                Srwx(a, b) => format!("srwx {} {}", a, b),
                Sww(a, b) => format!("sww {} {}", a, b),
                Swwx(a, b) => format!("swwx {} {}", a, b),
                Transfer(a, b, c) => format!("transfer {} {} {}", a, b, c),
                TransferOut(a, b, c, d) => format!("transferout {} {} {} {}", a, b, c, d),

                Ecrecover(a, b, c) => format!("ecrecover {} {} {}", a, b, c),
                Keccak256(a, b, c) => format!("keccak256 {} {} {}", a, b, c),
                Sha256(a, b, c) => format!("sha256 {} {} {}", a, b, c),

                Flag(a) => format!("flag {}", a),
            },
            Either::Right(opcode) => match opcode {
                Label(l) => format!("{}", l),
                RMove(r1, r2) => format!("move {} {}", r1, r2),
                Comment => "".into(),
                Jump(label) => format!("jump {}", label),
                Ld(register, data_id) => format!("ld {} {}", register, data_id),
                JumpIfNotEq(reg0, reg1, label) => format!("jnei {} {} {}", reg0, reg1, label),
            },
        };
        // we want the comment to always be 40 characters offset to the right
        // to not interfere with the ASM but to be aligned
        let mut op_and_comment = op_str;
        if self.comment.len() > 0 {
            while op_and_comment.len() < COMMENT_START_COLUMN {
                op_and_comment.push_str(" ");
            }
            op_and_comment.push_str(&format!("; {}", self.comment))
        }

        write!(f, "{}", op_and_comment)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct Label(pub(crate) usize);

// Convenience opcodes for the compiler -- will be optimized out or removed
// these do not reflect actual ops in the VM and will be compiled to bytecode
#[derive(Clone)]
pub(crate) enum OrganizationalOp {
    // copies the second register into the first register
    RMove(RegisterId, RegisterId),
    // Labels the code for jumps, will later be interpreted into offsets
    Label(Label),
    // Just a comment that will be inserted into the asm without an op
    Comment,
    // Jumps to a label
    Jump(Label),
    // Loads from the data section into a register
    // "load data"
    Ld(RegisterId, DataId),
    //
    JumpIfNotEq(RegisterId, RegisterId, Label),
}


impl OrganizationalOp {
    pub(crate) fn registers(&mut self) -> HashSet<&mut RegisterId> {
        use OrganizationalOp::*;
        (match self {
            RMove(ref mut r1, ref mut r2) => vec![r1, r2],
            Label(_) | Comment | Jump(_) => vec![],
            Ld(r1, _) => vec![r1],
            JumpIfNotEq(r1, r2, _l) => vec![r1, r2]
        }).into_iter().collect()
    }
}