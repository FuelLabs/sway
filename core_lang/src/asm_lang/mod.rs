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
use virtual_ops::{
    Label, VirtualImmediate06, VirtualImmediate12, VirtualImmediate18, VirtualImmediate24,
    VirtualOp, VirtualRegister,
};

pub(crate) mod allocated_ops;
pub(crate) mod virtual_ops;

/// The column where the ; for comments starts
const COMMENT_START_COLUMN: usize = 40;

impl From<&AsmRegister> for VirtualRegister {
    fn from(o: &AsmRegister) -> Self {
        VirtualRegister::Virtual(o.name.clone())
    }
}

#[derive(Clone)]
pub(crate) struct Op<'sc> {
    pub(crate) opcode: Either<VirtualOp, OrganizationalOp>,
    /// A descriptive comment for ASM readability
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span<'sc>>,
}

#[derive(Clone)]
pub(crate) struct RealizedOp<'sc> {
    pub(crate) opcode: VirtualOp,
    /// A descriptive comment for ASM readability
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span<'sc>>,
}

impl<'sc> Op<'sc> {
    /// Write value in given [VirtualRegister] `value_to_write` to given memory address that is held within the
    /// [VirtualRegister] `destination_address`
    pub(crate) fn write_register_to_memory(
        destination_address: VirtualRegister,
        value_to_write: VirtualRegister,
        offset: VirtualImmediate12,
        span: Span<'sc>,
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
        span: Span<'sc>,
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
        size_to_allocate_in_words: VirtualImmediate24,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::CFEI(size_to_allocate_in_words)),
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
    pub(crate) fn new(opcode: VirtualOp, owning_span: Span<'sc>) -> Self {
        Op {
            opcode: Either::Left(opcode),
            comment: String::new(),
            owning_span: Some(owning_span),
        }
    }
    pub(crate) fn new_with_comment(
        opcode: VirtualOp,
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
    /// Loads the data from [DataId] `data` into [VirtualRegister] `reg`.
    pub(crate) fn unowned_load_data_comment(
        reg: VirtualRegister,
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
    pub(crate) fn register_move(
        r1: VirtualRegister,
        r2: VirtualRegister,
        owning_span: Span<'sc>,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::MOVE(r1, r2)),
            comment: String::new(),
            owning_span: Some(owning_span),
        }
    }

    /// Moves the register in the second argument into the register in the first argument
    pub(crate) fn unowned_register_move(r1: VirtualRegister, r2: VirtualRegister) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::MOVE(r1, r2)),
            comment: String::new(),
            owning_span: None,
        }
    }
    pub(crate) fn register_move_comment(
        r1: VirtualRegister,
        r2: VirtualRegister,
        owning_span: Span<'sc>,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::MOVE(r1, r2)),
            comment: comment.into(),
            owning_span: Some(owning_span),
        }
    }

    /// Moves the register in the second argument into the register in the first argument
    pub(crate) fn unowned_register_move_comment(
        r1: VirtualRegister,
        r2: VirtualRegister,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::MOVE(r1, r2)),
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

    pub(crate) fn parse_opcode(
        name: &Ident<'sc>,
        args: &[&VirtualRegister],
        immediate: &Option<Ident<'sc>>,
        whole_op_span: Span<'sc>,
    ) -> CompileResult<'sc, VirtualOp> {
        let mut warnings = vec![];
        let mut errors = vec![];
        ok(
            match name.primary_name {
                "add" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ADD(r1, r2, r3)
                }
                "addi" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ADDI(r1, r2, imm)
                }
                "and" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::AND(r1, r2, r3)
                }
                "andi" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ANDI(r1, r2, imm)
                }
                "div" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::DIV(r1, r2, r3)
                }
                "divi" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::DIVI(r1, r2, imm)
                }
                "eq" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::EQ(r1, r2, r3)
                }
                "exp" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::EXP(r1, r2, r3)
                }
                "expi" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::EXPI(r1, r2, imm)
                }
                "gt" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::GT(r1, r2, r3)
                }
                "mlog" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MLOG(r1, r2, r3)
                }
                "mroo" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MROO(r1, r2, r3)
                }
                "mod" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MOD(r1, r2, r3)
                }
                "modi" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MODI(r1, r2, imm)
                }
                "move" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MOVE(r1, r2)
                }
                "mul" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MUL(r1, r2, r3)
                }
                "muli" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MULI(r1, r2, imm)
                }
                "not" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::NOT(r1, r2)
                }
                "or" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::OR(r1, r2, r3)
                }
                "ori" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ORI(r1, r2, imm)
                }
                "sll" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SLL(r1, r2, r3)
                }
                "slli" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SLLI(r1, r2, imm)
                }
                "srl" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SRL(r1, r2, r3)
                }
                "srli" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SRLI(r1, r2, imm)
                }
                "sub" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SUB(r1, r2, r3)
                }
                "subi" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SUBI(r1, r2, imm)
                }
                "xor" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::XOR(r1, r2, r3)
                }
                "xori" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::XORI(r1, r2, imm)
                }
                "cimv" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CIMV(r1, r2, r3)
                }
                "ctmv" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CTMV(r1, r2)
                }
                "ji" => {
                    errors.push(CompileError::DisallowedJi {
                        span: name.span.clone(),
                    });
                    return err(warnings, errors);
                }
                "jnei" => {
                    errors.push(CompileError::DisallowedJnei {
                        span: name.span.clone(),
                    });
                    return err(warnings, errors);
                }
                "ret" => {
                    let r1 = type_check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::RET(r1)
                }
                "cfei" => {
                    let imm = type_check!(
                        single_imm_24(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CFEI(imm)
                }
                "cfsi" => {
                    let imm = type_check!(
                        single_imm_24(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CFSI(imm)
                }
                "lb" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LB(r1, r2, imm)
                }
                "lw" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LW(r1, r2, imm)
                }
                "aloc" => {
                    let r1 = type_check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ALOC(r1)
                }
                "mcl" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MCL(r1, r2)
                }
                "mcli" => {
                    let (r1, imm) = type_check!(
                        single_reg_imm_18(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MCLI(r1, imm)
                }
                "mcp" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MCP(r1, r2, r3)
                }
                "meq" => {
                    let (r1, r2, r3, r4) = type_check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MEQ(r1, r2, r3, r4)
                }
                "sb" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SB(r1, r2, imm)
                }
                "sw" => {
                    let (r1, r2, imm) = type_check!(
                        two_regs_imm_12(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SW(r1, r2, imm)
                }
                "bhsh" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::BHSH(r1, r2)
                }
                "bhei" => {
                    let r1 = type_check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::BHEI(r1)
                }
                "burn" => {
                    let r1 = type_check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::BURN(r1)
                }
                "call" => {
                    let (r1, r2, r3, r4) = type_check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CALL(r1, r2, r3, r4)
                }
                "ccp" => {
                    let (r1, r2, r3, r4) = type_check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CCP(r1, r2, r3, r4)
                }
                "croo" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CROO(r1, r2)
                }
                "csiz" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CSIZ(r1, r2)
                }
                "cb" => {
                    let r1 = type_check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::CB(r1)
                }
                "ldc" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LDC(r1, r2, r3)
                }
                "log" => {
                    let (r1, r2, r3, r4) = type_check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::LOG(r1, r2, r3, r4)
                }
                "mint" => {
                    let r1 = type_check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::MINT(r1)
                }
                "rvrt" => {
                    let r1 = type_check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::RVRT(r1)
                }
                "sldc" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SLDC(r1, r2, r3)
                }
                "srw" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SRW(r1, r2)
                }
                "srwq" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SRWQ(r1, r2)
                }
                "sww" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SWW(r1, r2)
                }
                "swwq" => {
                    let (r1, r2) = type_check!(
                        two_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::SWWQ(r1, r2)
                }
                "tr" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::TR(r1, r2, r3)
                }
                "tro" => {
                    let (r1, r2, r3, r4) = type_check!(
                        four_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::TRO(r1, r2, r3, r4)
                }
                "ecr" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::ECR(r1, r2, r3)
                }
                "k256" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::K256(r1, r2, r3)
                }
                "s256" => {
                    let (r1, r2, r3) = type_check!(
                        three_regs(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::S256(r1, r2, r3)
                }
                "noop" => VirtualOp::NOOP,
                "flag" => {
                    let r1 = type_check!(
                        single_reg(args, immediate, whole_op_span),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    VirtualOp::FLAG(r1)
                }

                other => {
                    errors.push(CompileError::UnrecognizedOp {
                        op_name: other,
                        span: name.span.clone(),
                    });
                    return err(warnings, errors);
                }
            },
            warnings,
            errors,
        )
    }
}

fn single_reg<'sc>(
    args: &[&VirtualRegister],
    immediate: &Option<Ident<'sc>>,
    whole_op_span: Span<'sc>,
) -> CompileResult<'sc, VirtualRegister> {
    let mut warnings = vec![];
    let mut errors = vec![];
    if args.len() > 1 {
        errors.push(CompileError::IncorrectNumberOfAsmRegisters {
            expected: 1,
            received: args.len(),
            span: whole_op_span,
        });
    }

    let reg = match args.get(0) {
        Some(reg) => *reg,
        _ => todo!("Not enough registers error"),
    };
    match immediate {
        None => (),
        Some(_) => todo!("Err unnecessary immediate"),
    };

    ok(reg.clone(), warnings, errors)
}

fn two_regs<'sc>(
    args: &[&VirtualRegister],
    immediate: &Option<Ident<'sc>>,
    whole_op_span: Span<'sc>,
) -> CompileResult<'sc, (VirtualRegister, VirtualRegister)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    if args.len() > 2 {
        todo!("Unnecessary registers err")
    }

    let (reg, reg2) = match (args.get(0), args.get(1)) {
        (Some(reg), Some(reg2)) => (*reg, *reg2),
        _ => todo!("Not enough registers error"),
    };
    match immediate {
        None => (),
        Some(_) => todo!("Err unnecessary immediate"),
    };

    ok((reg.clone(), reg2.clone()), warnings, errors)
}

fn four_regs<'sc>(
    args: &[&VirtualRegister],
    immediate: &Option<Ident<'sc>>,
    whole_op_span: Span<'sc>,
) -> CompileResult<
    'sc,
    (
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
> {
    let mut warnings = vec![];
    let mut errors = vec![];
    if args.len() > 4 {
        todo!("Unnecessary registers err");
    }

    let (reg, reg2, reg3, reg4) = match (args.get(0), args.get(1), args.get(2), args.get(3)) {
        (Some(reg), Some(reg2), Some(reg3), Some(reg4)) => (*reg, *reg2, *reg3, *reg4),
        _ => todo!("Not enough registers error"),
    };
    match immediate {
        None => (),
        Some(_) => todo!("Err unnecessary immediate"),
    };

    ok(
        (reg.clone(), reg2.clone(), reg3.clone(), reg4.clone()),
        warnings,
        errors,
    )
}

fn three_regs<'sc>(
    args: &[&VirtualRegister],
    immediate: &Option<Ident<'sc>>,
    whole_op_span: Span<'sc>,
) -> CompileResult<'sc, (VirtualRegister, VirtualRegister, VirtualRegister)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    if args.len() > 3 {
        todo!("Unnecessary registers err");
    }

    let (reg, reg2, reg3) = match (args.get(0), args.get(1), args.get(2)) {
        (Some(reg), Some(reg2), Some(reg3)) => (*reg, *reg2, *reg3),
        _ => todo!("Not enough registers error"),
    };
    match immediate {
        None => (),
        Some(_) => todo!("Err unnecessary immediate"),
    };

    ok((reg.clone(), reg2.clone(), reg3.clone()), warnings, errors)
}
fn single_imm_24<'sc>(
    args: &[&VirtualRegister],
    immediate: &Option<Ident<'sc>>,
    whole_op_span: Span<'sc>,
) -> CompileResult<'sc, VirtualImmediate24> {
    let mut warnings = vec![];
    let mut errors = vec![];
    if args.len() > 0 {
        todo!("Unnecessary registers err");
    }
    let (imm, imm_span): (u64, _) = match immediate {
        None => todo!("Err missing immediate"),
        Some(i) => match i.primary_name.parse() {
            Ok(o) => (o, i.span.clone()),
            Err(_) => {
                errors.push(CompileError::InvalidImmediateValue {
                    span: i.span.clone(),
                });
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
fn single_reg_imm_18<'sc>(
    args: &[&VirtualRegister],
    immediate: &Option<Ident<'sc>>,
    whole_op_span: Span<'sc>,
) -> CompileResult<'sc, (VirtualRegister, VirtualImmediate18)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    if args.len() > 1 {
        todo!("Unnecessary registers err");
    }
    let reg = match args.get(0) {
        Some(reg) => *reg,
        _ => todo!("Not enough registers error"),
    };
    let (imm, imm_span): (u64, _) = match immediate {
        None => todo!("Err missing immediate"),
        Some(i) => match i.primary_name.parse() {
            Ok(o) => (o, i.span.clone()),
            Err(_) => {
                errors.push(CompileError::InvalidImmediateValue {
                    span: i.span.clone(),
                });
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
fn two_regs_imm_12<'sc>(
    args: &[&VirtualRegister],
    immediate: &Option<Ident<'sc>>,
    whole_op_span: Span<'sc>,
) -> CompileResult<'sc, (VirtualRegister, VirtualRegister, VirtualImmediate12)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    if args.len() > 2 {
        todo!("Unnecessary registers err");
    }
    let (reg, reg2) = match (args.get(0), args.get(1)) {
        (Some(reg), Some(reg2)) => (*reg, *reg2),
        _ => todo!("Not enough registers error"),
    };
    let (imm, imm_span): (u64, _) = match immediate {
        None => todo!("Err missing immediate"),
        Some(i) => match i.primary_name.parse() {
            Ok(o) => (o, i.span.clone()),
            Err(_) => {
                errors.push(CompileError::InvalidImmediateValue {
                    span: i.span.clone(),
                });
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

impl fmt::Display for Op<'_> {
    // very clunky but lets us tweak assembly language most easily
    // below code was constructed with vim macros -- easier to regenerate rather than rewrite.
    // @alex if you want to change the format and save yourself the pain.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
        /*
        use VirtualOp::*;
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
                MemClearVirtualImmediate(a, b) => format!("memcleari {} {}", a, b),
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
            */
    }
}

// Convenience opcodes for the compiler -- will be optimized out or removed
// these do not reflect actual ops in the VM and will be compiled to bytecode
#[derive(Clone)]
pub(crate) enum OrganizationalOp {
    // Labels the code for jumps, will later be interpreted into offsets
    Label(Label),
    // Just a comment that will be inserted into the asm without an op
    Comment,
    // Jumps to a label
    Jump(Label),
    // Jumps to a label
    JumpIfNotEq(VirtualRegister, VirtualRegister, Label),
    // Loads from the data section into a register
    // "load data"
    Ld(VirtualRegister, DataId),
}

impl OrganizationalOp {
    pub(crate) fn registers(&self) -> HashSet<&VirtualRegister> {
        use OrganizationalOp::*;
        (match self {
            Label(_) | Comment | Jump(_) => vec![],
            Ld(r1, _) => vec![r1],
            JumpIfNotEq(r1, r2, _) => vec![r1, r2],
        })
        .into_iter()
        .collect()
    }
}
