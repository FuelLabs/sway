//! This module contains things that I need from the VM to build that we will eventually import
//! from the VM when it is ready.
//! Basically this is copy-pasted until things are public and it can be properly imported.
//!
//! Only things needed for opcode serialization and generation are included here.
#![allow(dead_code)]

pub(crate) mod allocated_ops;
pub(crate) mod virtual_immediate;
pub(crate) mod virtual_ops;
pub(crate) mod virtual_register;
pub(crate) use virtual_immediate::*;
pub(crate) use virtual_ops::*;
pub(crate) use virtual_register::*;

use crate::{
    asm_generation::{DataId, RegisterPool},
    asm_lang::allocated_ops::{AllocatedOpcode, AllocatedRegister},
    error::*,
    language::AsmRegister,
    Ident,
};

use sway_error::error::CompileError;
use sway_types::{span::Span, Spanned};

use either::Either;
use std::{
    collections::{BTreeSet, HashMap},
    fmt::{self, Write},
    hash::Hash,
};

/// The column where the ; for comments starts
const COMMENT_START_COLUMN: usize = 40;

impl From<&AsmRegister> for VirtualRegister {
    fn from(o: &AsmRegister) -> Self {
        VirtualRegister::Virtual(o.name.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Op {
    pub(crate) opcode: Either<VirtualOp, OrganizationalOp>,
    /// A descriptive comment for ASM readability
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span>,
}

#[derive(Clone, Debug)]
pub(crate) struct AllocatedAbstractOp {
    pub(crate) opcode: Either<AllocatedOpcode, ControlFlowOp<AllocatedRegister>>,
    /// A descriptive comment for ASM readability
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span>,
}

#[derive(Clone, Debug)]
pub(crate) struct RealizedOp {
    pub(crate) opcode: AllocatedOpcode,
    /// A descriptive comment for ASM readability
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span>,
    pub(crate) offset: u64,
}

impl Op {
    /// Write value in given [VirtualRegister] `value_to_write` to given memory address that is held within the
    /// [VirtualRegister] `destination_address`
    pub(crate) fn write_register_to_memory(
        destination_address: VirtualRegister,
        value_to_write: VirtualRegister,
        offset: VirtualImmediate12,
        span: Span,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::SW(destination_address, value_to_write, offset)),
            comment: String::new(),
            owning_span: Some(span),
        }
    }
    /// Write value in given [VirtualRegister] `value_to_write` to given memory address that is held within the
    /// [VirtualRegister] `destination_address`, with the provided comment.
    pub(crate) fn write_register_to_memory_comment(
        destination_address: VirtualRegister,
        value_to_write: VirtualRegister,
        offset: VirtualImmediate12,
        span: Span,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::SW(destination_address, value_to_write, offset)),
            comment: comment.into(),
            owning_span: Some(span),
        }
    }
    /// Moves the stack pointer by the given amount (i.e. allocates stack memory)
    pub(crate) fn unowned_stack_allocate_memory(
        size_to_allocate_in_bytes: VirtualImmediate24,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::CFEI(size_to_allocate_in_bytes)),
            comment: String::new(),
            owning_span: None,
        }
    }
    pub(crate) fn unowned_new_with_comment(opcode: VirtualOp, comment: impl Into<String>) -> Self {
        Op {
            opcode: Either::Left(opcode),
            comment: comment.into(),
            owning_span: None,
        }
    }
    pub(crate) fn new(opcode: VirtualOp, owning_span: Span) -> Self {
        Op {
            opcode: Either::Left(opcode),
            comment: String::new(),
            owning_span: Some(owning_span),
        }
    }
    pub(crate) fn new_with_comment(
        opcode: VirtualOp,
        owning_span: Span,
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
    pub(crate) fn jump_label(label: Label, owning_span: Span) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Label(label)),
            comment: String::new(),
            owning_span: Some(owning_span),
        }
    }
    /// Loads the data from [DataId] `data` into [VirtualRegister] `reg`.
    pub(crate) fn unowned_load_data_comment(
        reg: VirtualRegister,
        data: DataId,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::LWDataId(reg, data)),
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
        owning_span: Span,
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

    /// Move an address at a label into a register.
    pub(crate) fn move_address(
        reg: VirtualRegister,
        label: Label,
        comment: impl Into<String>,
        owning_span: Option<Span>,
    ) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::MoveAddress(reg, label)),
            comment: comment.into(),
            owning_span,
        }
    }

    /// Moves the register in the second argument into the register in the first argument
    pub(crate) fn register_move(
        r1: VirtualRegister,
        r2: VirtualRegister,
        comment: impl Into<String>,
        owning_span: Option<Span>,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::MOVE(r1, r2)),
            comment: comment.into(),
            owning_span,
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

    /// Jumps to [Label] `label`  if the given [VirtualRegister] `reg1` is not equal to `reg0`.
    pub(crate) fn jump_if_not_equal(
        reg0: VirtualRegister,
        reg1: VirtualRegister,
        label: Label,
    ) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::JumpIfNotEq(reg0, reg1, label)),
            comment: String::new(),
            owning_span: None,
        }
    }

    /// Jumps to [Label] `label`  if the given [VirtualRegister] `reg0` is not equal to zero.
    pub(crate) fn jump_if_not_zero(reg0: VirtualRegister, label: Label) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::JumpIfNotZero(reg0, label)),
            comment: String::new(),
            owning_span: None,
        }
    }

    /// Dymamically jumps to a register value.
    pub(crate) fn jump_to_register(
        reg: VirtualRegister,
        comment: impl Into<String>,
        owning_span: Option<Span>,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::JMP(reg)),
            comment: comment.into(),
            owning_span,
        }
    }

    pub(crate) fn parse_opcode(
        name: &Ident,
        args: &[VirtualRegister],
        immediate: &Option<Ident>,
        whole_op_span: Span,
    ) -> CompileResult<VirtualOp> {
        let mut warnings = vec![];
        let mut errors = vec![];
        ok(
            match name.as_str() {
                "add" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ADD(r1, r2, r3)
                }
                "addi" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ADDI(r1, r2, imm)
                }
                "and" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::AND(r1, r2, r3)
                }
                "andi" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ANDI(r1, r2, imm)
                }
                "div" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::DIV(r1, r2, r3)
                }
                "divi" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::DIVI(r1, r2, imm)
                }
                "eq" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::EQ(r1, r2, r3)
                }
                "exp" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::EXP(r1, r2, r3)
                }
                "expi" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::EXPI(r1, r2, imm)
                }
                "gt" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::GT(r1, r2, r3)
                }
                "gtf" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::GTF(r1, r2, imm)
                }
                "lt" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LT(r1, r2, r3)
                }
                "mlog" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MLOG(r1, r2, r3)
                }
                "mroo" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MROO(r1, r2, r3)
                }
                "mod" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MOD(r1, r2, r3)
                }
                "modi" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MODI(r1, r2, imm)
                }
                "move" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MOVE(r1, r2)
                }
                "movi" => {
                    let (r1, imm) = check!(
                        single_reg_imm_18(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MOVI(r1, imm)
                }
                "mul" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MUL(r1, r2, r3)
                }
                "muli" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MULI(r1, r2, imm)
                }
                "not" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::NOT(r1, r2)
                }
                "or" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::OR(r1, r2, r3)
                }
                "ori" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ORI(r1, r2, imm)
                }
                "sll" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SLL(r1, r2, r3)
                }
                "slli" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SLLI(r1, r2, imm)
                }
                "smo" => {
                    let (r1, r2, r3, r4) = check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SMO(r1, r2, r3, r4)
                }
                "srl" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SRL(r1, r2, r3)
                }
                "srli" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SRLI(r1, r2, imm)
                }
                "sub" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SUB(r1, r2, r3)
                }
                "subi" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SUBI(r1, r2, imm)
                }
                "xor" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::XOR(r1, r2, r3)
                }
                "xori" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::XORI(r1, r2, imm)
                }
                "ji" => {
                    let imm = check!(
                        single_imm_24(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::JI(imm)
                }
                "jnei" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::JNEI(r1, r2, imm)
                }
                "jnzi" => {
                    let (r1, imm) = check!(
                        single_reg_imm_18(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::JNZI(r1, imm)
                }
                "ret" => {
                    let r1 = check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::RET(r1)
                }
                "retd" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::RETD(r1, r2)
                }
                "cfei" => {
                    let imm = check!(
                        single_imm_24(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CFEI(imm)
                }
                "cfsi" => {
                    let imm = check!(
                        single_imm_24(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CFSI(imm)
                }
                "lb" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LB(r1, r2, imm)
                }
                "lw" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LW(r1, r2, imm)
                }
                "aloc" => {
                    let r1 = check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ALOC(r1)
                }
                "mcl" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MCL(r1, r2)
                }
                "mcli" => {
                    let (r1, imm) = check!(
                        single_reg_imm_18(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MCLI(r1, imm)
                }
                "mcp" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MCP(r1, r2, r3)
                }
                "mcpi" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MCPI(r1, r2, imm)
                }
                "meq" => {
                    let (r1, r2, r3, r4) = check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MEQ(r1, r2, r3, r4)
                }
                "sb" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SB(r1, r2, imm)
                }
                "sw" => {
                    let (r1, r2, imm) = check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SW(r1, r2, imm)
                }
                "bal" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::BAL(r1, r2, r3)
                }
                "bhsh" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::BHSH(r1, r2)
                }
                "bhei" => {
                    let r1 = check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::BHEI(r1)
                }
                "burn" => {
                    let r1 = check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::BURN(r1)
                }
                "call" => {
                    let (r1, r2, r3, r4) = check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CALL(r1, r2, r3, r4)
                }
                "ccp" => {
                    let (r1, r2, r3, r4) = check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CCP(r1, r2, r3, r4)
                }
                "croo" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CROO(r1, r2)
                }
                "csiz" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CSIZ(r1, r2)
                }
                "cb" => {
                    let r1 = check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CB(r1)
                }
                "ldc" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LDC(r1, r2, r3)
                }
                "log" => {
                    let (r1, r2, r3, r4) = check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LOG(r1, r2, r3, r4)
                }
                "logd" => {
                    let (r1, r2, r3, r4) = check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LOGD(r1, r2, r3, r4)
                }
                "mint" => {
                    let r1 = check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MINT(r1)
                }
                "rvrt" => {
                    let r1 = check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::RVRT(r1)
                }
                "srw" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SRW(r1, r2)
                }
                "srwq" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SRWQ(r1, r2)
                }
                "sww" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SWW(r1, r2)
                }
                "swwq" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SWWQ(r1, r2)
                }
                "time" => {
                    let (r1, r2) = check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::TIME(r1, r2)
                }
                "tr" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::TR(r1, r2, r3)
                }
                "tro" => {
                    let (r1, r2, r3, r4) = check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::TRO(r1, r2, r3, r4)
                }
                "ecr" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ECR(r1, r2, r3)
                }
                "k256" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::K256(r1, r2, r3)
                }
                "s256" => {
                    let (r1, r2, r3) = check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::S256(r1, r2, r3)
                }
                "noop" => VirtualOp::NOOP,
                "flag" => {
                    let r1 = check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::FLAG(r1)
                }
                "gm" => {
                    let (r1, imm) = check!(
                        single_reg_imm_18(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::GM(r1, imm)
                }
                _ => {
                    errors.push(CompileError::UnrecognizedOp {
                        op_name: name.clone(),
                        span: name.span(),
                    });
                    return err(warnings, errors);
                }
            },
            warnings,
            errors,
        )
    }

    pub(crate) fn registers(&self) -> BTreeSet<&VirtualRegister> {
        match &self.opcode {
            Either::Left(virt_op) => virt_op.registers(),
            Either::Right(org_op) => org_op.registers(),
        }
    }

    pub(crate) fn use_registers(&self) -> BTreeSet<&VirtualRegister> {
        match &self.opcode {
            Either::Left(virt_op) => virt_op.use_registers(),
            Either::Right(org_op) => org_op.use_registers(),
        }
    }

    pub(crate) fn def_registers(&self) -> BTreeSet<&VirtualRegister> {
        match &self.opcode {
            Either::Left(virt_op) => virt_op.def_registers(),
            Either::Right(org_op) => org_op.def_registers(),
        }
    }

    pub(crate) fn successors(&self, index: usize, ops: &[Op]) -> Vec<usize> {
        match &self.opcode {
            Either::Left(virt_op) => virt_op.successors(index, ops),
            Either::Right(org_op) => org_op.successors(index, ops),
        }
    }

    pub(crate) fn update_register(
        &self,
        reg_to_reg_map: &HashMap<VirtualRegister, VirtualRegister>,
    ) -> Self {
        Op {
            opcode: match &self.opcode {
                Either::Left(virt_op) => Either::Left(virt_op.update_register(reg_to_reg_map)),
                Either::Right(org_op) => Either::Right(org_op.update_register(reg_to_reg_map)),
            },
            comment: self.comment.clone(),
            owning_span: self.owning_span.clone(),
        }
    }

    pub(crate) fn allocate_registers(
        &self,
        pool: &RegisterPool,
    ) -> Either<AllocatedOpcode, ControlFlowOp<AllocatedRegister>> {
        match &self.opcode {
            Either::Left(virt_op) => Either::Left(virt_op.allocate_registers(pool)),
            Either::Right(org_op) => Either::Right(org_op.allocate_registers(pool)),
        }
    }
}

fn single_reg(
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> CompileResult<VirtualRegister> {
    let warnings = vec![];
    let mut errors = vec![];
    if args.len() > 1 {
        errors.push(CompileError::IncorrectNumberOfAsmRegisters {
            expected: 1,
            received: args.len(),
            span: whole_op_span.clone(),
        });
    }

    let reg = match args.get(0) {
        Some(reg) => reg,
        _ => {
            errors.push(CompileError::IncorrectNumberOfAsmRegisters {
                span: whole_op_span,
                expected: 1,
                received: args.len(),
            });
            return err(warnings, errors);
        }
    };
    match immediate {
        None => (),
        Some(i) => {
            errors.push(CompileError::UnnecessaryImmediate { span: i.span() });
        }
    };

    ok(reg.clone(), warnings, errors)
}

fn two_regs(
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> CompileResult<(VirtualRegister, VirtualRegister)> {
    let warnings = vec![];
    let mut errors = vec![];
    if args.len() > 2 {
        errors.push(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 2,
            received: args.len(),
        });
    }

    let (reg, reg2) = match (args.get(0), args.get(1)) {
        (Some(reg), Some(reg2)) => (reg, reg2),
        _ => {
            errors.push(CompileError::IncorrectNumberOfAsmRegisters {
                span: whole_op_span,
                expected: 2,
                received: args.len(),
            });
            return err(warnings, errors);
        }
    };
    match immediate {
        None => (),
        Some(i) => errors.push(CompileError::UnnecessaryImmediate { span: i.span() }),
    };

    ok((reg.clone(), reg2.clone()), warnings, errors)
}

fn four_regs(
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> CompileResult<(
    VirtualRegister,
    VirtualRegister,
    VirtualRegister,
    VirtualRegister,
)> {
    let warnings = vec![];
    let mut errors = vec![];
    if args.len() > 4 {
        errors.push(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 4,
            received: args.len(),
        });
    }

    let (reg, reg2, reg3, reg4) = match (args.get(0), args.get(1), args.get(2), args.get(3)) {
        (Some(reg), Some(reg2), Some(reg3), Some(reg4)) => (reg, reg2, reg3, reg4),
        _ => {
            errors.push(CompileError::IncorrectNumberOfAsmRegisters {
                span: whole_op_span,
                expected: 4,
                received: args.len(),
            });
            return err(warnings, errors);
        }
    };
    match immediate {
        None => (),
        Some(i) => {
            errors.push(CompileError::MissingImmediate { span: i.span() });
        }
    };

    impl ConstantRegister {
        pub(crate) fn parse_register_name(raw: &str) -> Option<ConstantRegister> {
            use ConstantRegister::*;
            Some(match raw {
                "zero" => Zero,
                "one" => One,
                "of" => Overflow,
                "pc" => ProgramCounter,
                "ssp" => StackStartPointer,
                "sp" => StackPointer,
                "fp" => FramePointer,
                "hp" => HeapPointer,
                "err" => Error,
                "ggas" => GlobalGas,
                "cgas" => ContextGas,
                "bal" => Balance,
                "is" => InstructionStart,
                "flag" => Flags,
                "retl" => ReturnLength,
                "ret" => ReturnValue,
                "ds" => DataSectionStart,
                _ => return None,
            })
        }
    }

    // Immediate Value.
    pub type ImmediateValue = u32;

    ok(
        (reg.clone(), reg2.clone(), reg3.clone(), reg4.clone()),
        warnings,
        errors,
    )
}

fn three_regs(
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> CompileResult<(VirtualRegister, VirtualRegister, VirtualRegister)> {
    let warnings = vec![];
    let mut errors = vec![];
    if args.len() > 3 {
        errors.push(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 3,
            received: args.len(),
        });
    }

    let (reg, reg2, reg3) = match (args.get(0), args.get(1), args.get(2)) {
        (Some(reg), Some(reg2), Some(reg3)) => (reg, reg2, reg3),
        _ => {
            errors.push(CompileError::IncorrectNumberOfAsmRegisters {
                span: whole_op_span,
                expected: 3,
                received: args.len(),
            });
            return err(warnings, errors);
        }
    };
    match immediate {
        None => (),
        Some(i) => {
            errors.push(CompileError::UnnecessaryImmediate { span: i.span() });
        }
    };

    ok((reg.clone(), reg2.clone(), reg3.clone()), warnings, errors)
}
fn single_imm_24(
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> CompileResult<VirtualImmediate24> {
    let warnings = vec![];
    let mut errors = vec![];
    if !args.is_empty() {
        errors.push(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 0,
            received: args.len(),
        });
    }
    let (imm, imm_span): (u64, _) = match immediate {
        None => {
            errors.push(CompileError::MissingImmediate {
                span: whole_op_span,
            });
            return err(warnings, errors);
        }
        Some(i) => match i.as_str()[1..].parse() {
            Ok(o) => (o, i.span()),
            Err(_) => {
                errors.push(CompileError::InvalidImmediateValue { span: i.span() });
                return err(warnings, errors);
            }
        },
    };

    let imm = match VirtualImmediate24::new(imm, imm_span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e);
            return err(warnings, errors);
        }
    };

    ok(imm, warnings, errors)
}
fn single_reg_imm_18(
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> CompileResult<(VirtualRegister, VirtualImmediate18)> {
    let warnings = vec![];
    let mut errors = vec![];
    if args.len() > 1 {
        errors.push(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 1,
            received: args.len(),
        });
    }
    let reg = match args.get(0) {
        Some(reg) => reg,
        _ => {
            errors.push(CompileError::IncorrectNumberOfAsmRegisters {
                span: whole_op_span,
                expected: 1,
                received: args.len(),
            });
            return err(warnings, errors);
        }
    };
    let (imm, imm_span): (u64, _) = match immediate {
        None => {
            errors.push(CompileError::MissingImmediate {
                span: whole_op_span,
            });
            return err(warnings, errors);
        }
        Some(i) => match i.as_str()[1..].parse() {
            Ok(o) => (o, i.span()),
            Err(_) => {
                errors.push(CompileError::InvalidImmediateValue { span: i.span() });
                return err(warnings, errors);
            }
        },
    };

    let imm = match VirtualImmediate18::new(imm, imm_span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e);
            return err(warnings, errors);
        }
    };

    ok((reg.clone(), imm), warnings, errors)
}
fn two_regs_imm_12(
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> CompileResult<(VirtualRegister, VirtualRegister, VirtualImmediate12)> {
    let warnings = vec![];
    let mut errors = vec![];
    if args.len() > 2 {
        errors.push(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 2,
            received: args.len(),
        });
    }
    let (reg, reg2) = match (args.get(0), args.get(1)) {
        (Some(reg), Some(reg2)) => (reg, reg2),
        _ => {
            errors.push(CompileError::IncorrectNumberOfAsmRegisters {
                span: whole_op_span,
                expected: 2,
                received: args.len(),
            });
            return err(warnings, errors);
        }
    };
    let (imm, imm_span): (u64, _) = match immediate {
        None => {
            errors.push(CompileError::MissingImmediate {
                span: whole_op_span,
            });
            return err(warnings, errors);
        }
        Some(i) => match i.as_str()[1..].parse() {
            Ok(o) => (o, i.span()),
            Err(_) => {
                errors.push(CompileError::InvalidImmediateValue { span: i.span() });
                return err(warnings, errors);
            }
        },
    };

    let imm = match VirtualImmediate12::new(imm, imm_span) {
        Ok(o) => o,
        Err(e) => {
            errors.push(e);
            return err(warnings, errors);
        }
    };

    ok((reg.clone(), reg2.clone(), imm), warnings, errors)
}

impl fmt::Display for Op {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We want the comment to always be 40 characters offset to the right to not interfere with
        // the ASM but to be aligned.
        let mut op_and_comment = self.opcode.to_string();
        if !self.comment.is_empty() {
            while op_and_comment.len() < COMMENT_START_COLUMN {
                op_and_comment.push(' ');
            }
            write!(op_and_comment, "; {}", self.comment)?;
        }

        write!(fmtr, "{}", op_and_comment)
    }
}

impl fmt::Display for VirtualOp {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        use VirtualOp::*;
        match self {
            ADD(a, b, c) => write!(fmtr, "add {} {} {}", a, b, c),
            ADDI(a, b, c) => write!(fmtr, "addi {} {} {}", a, b, c),
            AND(a, b, c) => write!(fmtr, "and {} {} {}", a, b, c),
            ANDI(a, b, c) => write!(fmtr, "andi {} {} {}", a, b, c),
            DIV(a, b, c) => write!(fmtr, "div {} {} {}", a, b, c),
            DIVI(a, b, c) => write!(fmtr, "divi {} {} {}", a, b, c),
            EQ(a, b, c) => write!(fmtr, "eq {} {} {}", a, b, c),
            EXP(a, b, c) => write!(fmtr, "exp {} {} {}", a, b, c),
            EXPI(a, b, c) => write!(fmtr, "expi {} {} {}", a, b, c),
            GT(a, b, c) => write!(fmtr, "gt {} {} {}", a, b, c),
            GTF(a, b, c) => write!(fmtr, "gtf {} {} {}", a, b, c),
            LT(a, b, c) => write!(fmtr, "lt {} {} {}", a, b, c),
            MLOG(a, b, c) => write!(fmtr, "mlog {} {} {}", a, b, c),
            MROO(a, b, c) => write!(fmtr, "mroo {} {} {}", a, b, c),
            MOD(a, b, c) => write!(fmtr, "mod {} {} {}", a, b, c),
            MODI(a, b, c) => write!(fmtr, "modi {} {} {}", a, b, c),
            MOVE(a, b) => write!(fmtr, "move {} {}", a, b),
            MOVI(a, b) => write!(fmtr, "movi {} {}", a, b),
            MUL(a, b, c) => write!(fmtr, "mul {} {} {}", a, b, c),
            MULI(a, b, c) => write!(fmtr, "muli {} {} {}", a, b, c),
            NOT(a, b) => write!(fmtr, "not {} {}", a, b),
            OR(a, b, c) => write!(fmtr, "or {} {} {}", a, b, c),
            ORI(a, b, c) => write!(fmtr, "ori {} {} {}", a, b, c),
            SLL(a, b, c) => write!(fmtr, "sll {} {} {}", a, b, c),
            SLLI(a, b, c) => write!(fmtr, "slli {} {} {}", a, b, c),
            SMO(a, b, c, d) => write!(fmtr, "smo {} {} {} {}", a, b, c, d),
            SRL(a, b, c) => write!(fmtr, "srl {} {} {}", a, b, c),
            SRLI(a, b, c) => write!(fmtr, "srli {} {} {}", a, b, c),
            SUB(a, b, c) => write!(fmtr, "sub {} {} {}", a, b, c),
            SUBI(a, b, c) => write!(fmtr, "subi {} {} {}", a, b, c),
            XOR(a, b, c) => write!(fmtr, "xor {} {} {}", a, b, c),
            XORI(a, b, c) => write!(fmtr, "xori {} {} {}", a, b, c),
            JMP(a) => write!(fmtr, "jmp {}", a),
            JI(a) => write!(fmtr, "ji {}", a),
            JNEI(a, b, c) => write!(fmtr, "jnei {} {} {}", a, b, c),
            JNZI(a, b) => write!(fmtr, "jnzi {} {}", a, b),
            RET(a) => write!(fmtr, "ret {}", a),
            RETD(a, b) => write!(fmtr, "retd {} {}", a, b),
            CFEI(a) => write!(fmtr, "cfei {}", a),
            CFSI(a) => write!(fmtr, "cfsi {}", a),
            LB(a, b, c) => write!(fmtr, "lb {} {} {}", a, b, c),
            LWDataId(a, b) => write!(fmtr, "lw {} {}", a, b),
            LW(a, b, c) => write!(fmtr, "lw {} {} {}", a, b, c),
            ALOC(a) => write!(fmtr, "aloc {}", a),
            MCL(a, b) => write!(fmtr, "mcl {} {}", a, b),
            MCLI(a, b) => write!(fmtr, "mcli {} {}", a, b),
            MCP(a, b, c) => write!(fmtr, "mcp {} {} {}", a, b, c),
            MCPI(a, b, c) => write!(fmtr, "mcpi {} {} {}", a, b, c),
            MEQ(a, b, c, d) => write!(fmtr, "meq {} {} {} {}", a, b, c, d),
            SB(a, b, c) => write!(fmtr, "sb {} {} {}", a, b, c),
            SW(a, b, c) => write!(fmtr, "sw {} {} {}", a, b, c),
            BAL(a, b, c) => write!(fmtr, "bal {} {} {}", a, b, c),
            BHSH(a, b) => write!(fmtr, "bhsh {} {}", a, b),
            BHEI(a) => write!(fmtr, "bhei {}", a),
            BURN(a) => write!(fmtr, "burn {}", a),
            CALL(a, b, c, d) => write!(fmtr, "call {} {} {} {}", a, b, c, d),
            CCP(a, b, c, d) => write!(fmtr, "ccp {} {} {} {}", a, b, c, d),
            CROO(a, b) => write!(fmtr, "croo {} {}", a, b),
            CSIZ(a, b) => write!(fmtr, "csiz {} {}", a, b),
            CB(a) => write!(fmtr, "cb {}", a),
            LDC(a, b, c) => write!(fmtr, "ldc {} {} {}", a, b, c),
            LOG(a, b, c, d) => write!(fmtr, "log {} {} {} {}", a, b, c, d),
            LOGD(a, b, c, d) => write!(fmtr, "logd {} {} {} {}", a, b, c, d),
            MINT(a) => write!(fmtr, "mint {}", a),
            RVRT(a) => write!(fmtr, "rvrt {}", a),
            SRW(a, b) => write!(fmtr, "srw {} {}", a, b),
            SRWQ(a, b) => write!(fmtr, "srwq {} {}", a, b),
            SWW(a, b) => write!(fmtr, "sww {} {}", a, b),
            SWWQ(a, b) => write!(fmtr, "swwq {} {}", a, b),
            TIME(a, b) => write!(fmtr, "time {} {}", a, b),
            TR(a, b, c) => write!(fmtr, "tr {} {} {}", a, b, c),
            TRO(a, b, c, d) => write!(fmtr, "tro {} {} {} {}", a, b, c, d),
            ECR(a, b, c) => write!(fmtr, "ecr {} {} {}", a, b, c),
            K256(a, b, c) => write!(fmtr, "k256 {} {} {}", a, b, c),
            S256(a, b, c) => write!(fmtr, "s256 {} {} {}", a, b, c),
            NOOP => Ok(()),
            FLAG(a) => write!(fmtr, "flag {}", a),
            GM(a, b) => write!(fmtr, "gm {} {}", a, b),

            Undefined => write!(fmtr, "undefined op"),

            DataSectionOffsetPlaceholder => write!(fmtr, "data section offset placeholder"),
            DataSectionRegisterLoadPlaceholder => {
                write!(fmtr, "data section register load placeholder")
            }
        }
    }
}

impl fmt::Display for AllocatedAbstractOp {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We want the comment to always be 40 characters offset to the right to not interfere with
        // the ASM but to be aligned.
        let mut op_and_comment = self.opcode.to_string();
        if !self.comment.is_empty() {
            while op_and_comment.len() < COMMENT_START_COLUMN {
                op_and_comment.push(' ');
            }
            write!(op_and_comment, "; {}", self.comment)?;
        }

        write!(fmtr, "{}", op_and_comment)
    }
}

// Convenience opcodes for the compiler -- will be optimized out or removed
// these do not reflect actual ops in the VM and will be compiled to bytecode
#[derive(Debug, Clone)]
pub(crate) enum ControlFlowOp<Reg> {
    // Labels the code for jumps, will later be interpreted into offsets
    Label(Label),
    // Just a comment that will be inserted into the asm without an op
    Comment,
    // Jumps to a label
    Jump(Label),
    // Jumps to a label if the two registers are different
    JumpIfNotEq(Reg, Reg, Label),
    // Jumps to a label if the register is not equal to zero
    JumpIfNotZero(Reg, Label),
    // Jumps to a label, similarly to Jump, though semantically expecting to return.
    Call(Label),
    // Save a label address in a register.
    MoveAddress(Reg, Label),
    // placeholder for the DataSection offset
    DataSectionOffsetPlaceholder,
    // Save all currently live general purpose registers, using a label as a handle.
    PushAll(Label),
    // Restore all previously saved general purpose registers.
    PopAll(Label),
}

pub(crate) type OrganizationalOp = ControlFlowOp<VirtualRegister>;

impl<Reg: fmt::Display> fmt::Display for ControlFlowOp<Reg> {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ControlFlowOp::*;
        write!(
            fmtr,
            "{}",
            match self {
                Label(lab) => format!("{}", lab),
                Jump(lab) => format!("ji  {}", lab),
                Comment => "".into(),
                JumpIfNotEq(r1, r2, lab) => format!("jnei {} {} {}", r1, r2, lab),
                JumpIfNotZero(r1, lab) => format!("jnzi {} {}", r1, lab),
                Call(lab) => format!("fncall {lab}"),
                MoveAddress(r1, lab) => format!("mova {} {}", r1, lab),
                DataSectionOffsetPlaceholder =>
                    "DATA SECTION OFFSET[0..32]\nDATA SECTION OFFSET[32..64]".into(),
                PushAll(lab) => format!("pusha {lab}"),
                PopAll(lab) => format!("popa {lab}"),
            }
        )
    }
}

impl<Reg: Clone + Eq + Ord + Hash> ControlFlowOp<Reg> {
    pub(crate) fn registers(&self) -> BTreeSet<&Reg> {
        use ControlFlowOp::*;
        (match self {
            Label(_)
            | Comment
            | Jump(_)
            | Call(_)
            | DataSectionOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => vec![],

            JumpIfNotEq(r1, r2, _) => vec![r1, r2],
            JumpIfNotZero(r1, _) | MoveAddress(r1, _) => vec![r1],
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn use_registers(&self) -> BTreeSet<&Reg> {
        use ControlFlowOp::*;
        (match self {
            Label(_)
            | Comment
            | Jump(_)
            | Call(_)
            | MoveAddress(..)
            | DataSectionOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => vec![],

            JumpIfNotZero(r1, _) => vec![r1],
            JumpIfNotEq(r1, r2, _) => vec![r1, r2],
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn def_registers(&self) -> BTreeSet<&Reg> {
        use ControlFlowOp::*;
        (match self {
            MoveAddress(reg, _) => vec![reg],

            Label(_)
            | Comment
            | Jump(_)
            | JumpIfNotEq(..)
            | JumpIfNotZero(..)
            | Call(_)
            | DataSectionOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => vec![],
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn update_register(&self, reg_to_reg_map: &HashMap<Reg, Reg>) -> Self {
        let update_reg = |reg: &Reg| -> Reg {
            reg_to_reg_map
                .get(reg)
                .cloned()
                .unwrap_or_else(|| reg.clone())
        };

        use ControlFlowOp::*;
        match self {
            Comment
            | Label(_)
            | Jump(_)
            | Call(_)
            | DataSectionOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => self.clone(),

            JumpIfNotEq(r1, r2, label) => Self::JumpIfNotEq(update_reg(r1), update_reg(r2), *label),
            JumpIfNotZero(r1, label) => Self::JumpIfNotZero(update_reg(r1), *label),
            MoveAddress(r1, label) => Self::MoveAddress(update_reg(r1), *label),
        }
    }

    pub(crate) fn successors(&self, index: usize, ops: &[Op]) -> Vec<usize> {
        use ControlFlowOp::*;

        let mut next_ops = Vec::new();

        if index + 1 < ops.len() && !matches!(self, Jump(_)) {
            next_ops.push(index + 1);
        };

        match self {
            Label(_)
            | Comment
            | Call(_)
            | MoveAddress(..)
            | DataSectionOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => (),

            Jump(jump_label) | JumpIfNotEq(_, _, jump_label) | JumpIfNotZero(_, jump_label) => {
                // Find the label in the ops list.
                for (idx, op) in ops.iter().enumerate() {
                    if let Either::Right(ControlFlowOp::Label(op_label)) = op.opcode {
                        if op_label == *jump_label {
                            next_ops.push(idx);
                            break;
                        }
                    }
                }
            }
        };

        next_ops
    }
}

impl ControlFlowOp<VirtualRegister> {
    // Copied directly from VirtualOp::allocate_registers().
    pub(crate) fn allocate_registers(
        &self,
        pool: &RegisterPool,
    ) -> ControlFlowOp<AllocatedRegister> {
        let virtual_registers = self.registers();
        let register_allocation_result = virtual_registers
            .clone()
            .into_iter()
            .map(|x| match x {
                VirtualRegister::Constant(c) => (x, Some(AllocatedRegister::Constant(*c))),
                VirtualRegister::Virtual(_) => (x, pool.get_register(x)),
            })
            .map(|(x, register_opt)| register_opt.map(|register| (x, register)))
            .collect::<Option<Vec<_>>>();

        // Maps virtual registers to their allocated equivalent
        let mut mapping: HashMap<&VirtualRegister, AllocatedRegister> = HashMap::default();
        match register_allocation_result {
            Some(o) => {
                for (key, val) in o {
                    mapping.insert(key, val);
                }
            }
            None => {
                unimplemented!(
                    "The allocator cannot resolve a register mapping for this program.
                 This is a temporary artifact of the extremely early stage version of this language.
                 Try to lower the number of variables you use."
                );
            }
        };

        let map_reg = |reg: &VirtualRegister| mapping.get(reg).unwrap().clone();

        use ControlFlowOp::*;
        match self {
            Label(label) => Label(*label),
            Comment => Comment,
            Jump(label) => Jump(*label),
            Call(label) => Call(*label),
            DataSectionOffsetPlaceholder => DataSectionOffsetPlaceholder,
            PushAll(label) => PushAll(*label),
            PopAll(label) => PopAll(*label),

            JumpIfNotEq(r1, r2, label) => JumpIfNotEq(map_reg(r1), map_reg(r2), *label),
            JumpIfNotZero(r1, label) => JumpIfNotZero(map_reg(r1), *label),
            MoveAddress(r1, label) => MoveAddress(map_reg(r1), *label),
        }
    }
}
