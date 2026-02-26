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
use indexmap::IndexMap;
pub(crate) use virtual_immediate::*;
pub(crate) use virtual_ops::*;
pub(crate) use virtual_register::*;

use crate::{
    asm_generation::fuel::{data_section::DataId, register_allocator::RegisterPool},
    asm_lang::allocated_ops::{AllocatedInstruction, AllocatedRegister},
    language::AsmRegister,
    Ident,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{span::Span, Spanned};

use either::Either;
use std::{
    collections::{BTreeSet, HashMap},
    fmt::{self, Write},
    hash::Hash,
};

/// The column where the ; for comments starts
const COMMENT_START_COLUMN: usize = 40;

fn fmt_opcode_and_comment(
    opcode: String,
    comment: &str,
    fmtr: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    // We want the comment to be at the `COMMENT_START_COLUMN` offset to the right,
    // to not interfere with the ASM but to be aligned.
    // Some operations like, e.g., data section offset, can span multiple lines.
    // In that case, we put the comment at the end of the last line, aligned.
    let mut op_and_comment = opcode;
    if !comment.is_empty() {
        let mut op_length = match op_and_comment.rfind('\n') {
            Some(new_line_index) => op_and_comment.len() - new_line_index - 1,
            None => op_and_comment.len(),
        };
        while op_length < COMMENT_START_COLUMN {
            op_and_comment.push(' ');
            op_length += 1;
        }
        write!(op_and_comment, "; {comment}")?;
    }

    write!(fmtr, "{op_and_comment}")
}

impl From<&AsmRegister> for VirtualRegister {
    fn from(o: &AsmRegister) -> Self {
        VirtualRegister::Virtual(o.name.clone())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Op {
    pub(crate) opcode: Either<VirtualOp, OrganizationalOp>,
    /// A descriptive comment for ASM readability.
    ///
    /// Comments are a part of the compiler output and meant to
    /// help both Sway developers interested in the generated ASM
    /// and the Sway compiler developers.
    ///
    /// Comments follow these guidelines:
    ///   - they start with an imperative verb. E.g.: "allocate" and not "allocating".
    ///   - they start with a lowercase letter. E.g.: "allocate" and not "Allocate".
    ///   - they do not end in punctuation. E.g.: "store value" and not "store value.".
    ///   - they use full words. E.g.: "load return address" and not "load reta" or "load return addr".
    ///   - abbreviations are written in upper-case. E.g.: "ABI" and not "abi".
    ///   - names (e.g., function, argument, etc.) are written without quotes. E.g. "main" and not "'main'".
    ///   - assembly operations are written in lowercase. E.g.: "move" and not "MOVE".
    ///   - they are short and concise.
    ///   - if an operation is a part of a logical group of operations, start the comment
    ///     by a descriptive group name enclosed in square brackets and followed by colon.
    ///     The remaining part of the comment follows the above guidelines. E.g.:
    ///     "[bitcast to bool]: convert value to inverted boolean".
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span>,
}

#[derive(Clone, Debug)]
pub(crate) struct AllocatedAbstractOp {
    pub(crate) opcode: Either<AllocatedInstruction, ControlFlowOp<AllocatedRegister>>,
    /// A descriptive comment for ASM readability.
    ///
    /// For writing guidelines, see [Op::comment].
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span>,
}

#[derive(Clone, Debug)]
pub(crate) struct RealizedOp {
    pub(crate) opcode: AllocatedInstruction,
    /// A descriptive comment for ASM readability.
    ///
    /// For writing guidelines, see [Op::comment].
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span>,
}

impl Op {
    /// Moves the stack pointer by the given amount (i.e. allocates stack memory)
    pub(crate) fn unowned_stack_allocate_memory(
        size_to_allocate_in_bytes: VirtualImmediate24,
    ) -> Self {
        Op {
            opcode: Either::Left(VirtualOp::CFEI(
                VirtualRegister::Constant(ConstantRegister::StackPointer),
                size_to_allocate_in_bytes,
            )),
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
            opcode: Either::Left(VirtualOp::LoadDataId(reg, data)),
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
            opcode: Either::Right(OrganizationalOp::Jump {
                to: label,
                type_: JumpType::Unconditional,
            }),
            comment: String::new(),
            owning_span: None,
        }
    }

    pub(crate) fn jump_to_label_comment(label: Label, comment: impl Into<String>) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Jump {
                to: label,
                type_: JumpType::Unconditional,
            }),
            comment: comment.into(),
            owning_span: None,
        }
    }

    /// Jumps to [Label] `label` if the given [VirtualRegister] `reg0` is not equal to zero.
    pub(crate) fn jump_if_not_zero(reg0: VirtualRegister, label: Label) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Jump {
                to: label,
                type_: JumpType::NotZero(reg0),
            }),
            comment: String::new(),
            owning_span: None,
        }
    }

    /// Jumps to [Label] `label` if the given [VirtualRegister] `reg0` is not equal to zero.
    pub(crate) fn jump_if_not_zero_comment(
        reg0: VirtualRegister,
        label: Label,
        comment: impl Into<String>,
    ) -> Self {
        Op {
            opcode: Either::Right(OrganizationalOp::Jump {
                to: label,
                type_: JumpType::NotZero(reg0),
            }),
            comment: comment.into(),
            owning_span: None,
        }
    }

    pub(crate) fn parse_opcode(
        handler: &Handler,
        name: &Ident,
        args: &[VirtualRegister],
        immediate: &Option<Ident>,
        whole_op_span: Span,
    ) -> Result<VirtualOp, ErrorEmitted> {
        Ok(match name.as_str() {
            /* Arithmetic/Logic (ALU) Instructions */
            "add" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::ADD(r1, r2, r3)
            }
            "addi" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::ADDI(r1, r2, imm)
            }
            "and" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::AND(r1, r2, r3)
            }
            "andi" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::ANDI(r1, r2, imm)
            }
            "div" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::DIV(r1, r2, r3)
            }
            "divi" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::DIVI(r1, r2, imm)
            }
            "eq" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::EQ(r1, r2, r3)
            }
            "exp" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::EXP(r1, r2, r3)
            }
            "expi" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::EXPI(r1, r2, imm)
            }
            "gt" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::GT(r1, r2, r3)
            }
            "lt" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::LT(r1, r2, r3)
            }
            "mlog" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MLOG(r1, r2, r3)
            }
            "mod" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MOD(r1, r2, r3)
            }
            "modi" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::MODI(r1, r2, imm)
            }
            "move" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MOVE(r1, r2)
            }
            "movi" => {
                let (r1, imm) = single_reg_imm_18(handler, args, immediate, whole_op_span)?;
                VirtualOp::MOVI(r1, imm)
            }
            "mroo" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MROO(r1, r2, r3)
            }
            "mul" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MUL(r1, r2, r3)
            }
            "muli" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::MULI(r1, r2, imm)
            }
            "noop" => VirtualOp::NOOP,
            "not" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::NOT(r1, r2)
            }
            "or" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::OR(r1, r2, r3)
            }
            "ori" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::ORI(r1, r2, imm)
            }
            "sll" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::SLL(r1, r2, r3)
            }
            "slli" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::SLLI(r1, r2, imm)
            }
            "srl" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::SRL(r1, r2, r3)
            }
            "srli" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::SRLI(r1, r2, imm)
            }
            "sub" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::SUB(r1, r2, r3)
            }
            "subi" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::SUBI(r1, r2, imm)
            }
            "wqcm" => {
                let (r1, r2, r3, imm) = three_regs_imm_06(handler, args, immediate, whole_op_span)?;
                VirtualOp::WQCM(r1, r2, r3, imm)
            }
            "wqop" => {
                let (r1, r2, r3, imm) = three_regs_imm_06(handler, args, immediate, whole_op_span)?;
                VirtualOp::WQOP(r1, r2, r3, imm)
            }
            "wqml" => {
                let (r1, r2, r3, imm) = three_regs_imm_06(handler, args, immediate, whole_op_span)?;
                VirtualOp::WQML(r1, r2, r3, imm)
            }
            "wqdv" => {
                let (r1, r2, r3, imm) = three_regs_imm_06(handler, args, immediate, whole_op_span)?;
                VirtualOp::WQDV(r1, r2, r3, imm)
            }
            "wqmd" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::WQMD(r1, r2, r3, r4)
            }
            "wqam" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::WQAM(r1, r2, r3, r4)
            }
            "wqmm" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::WQMM(r1, r2, r3, r4)
            }
            "xor" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::XOR(r1, r2, r3)
            }
            "xori" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::XORI(r1, r2, imm)
            }

            /* Control Flow Instructions */
            "jmp" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::JMP(r1)
            }
            "ji" => {
                let imm = single_imm_24(handler, args, immediate, whole_op_span)?;
                VirtualOp::JI(imm)
            }
            "jne" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::JNE(r1, r2, r3)
            }
            "jnei" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::JNEI(r1, r2, imm)
            }
            "jnzi" => {
                let (r1, imm) = single_reg_imm_18(handler, args, immediate, whole_op_span)?;
                VirtualOp::JNZI(r1, imm)
            }
            "jmpb" => {
                let (r1, imm) = single_reg_imm_18(handler, args, immediate, whole_op_span)?;
                VirtualOp::JMPB(r1, imm)
            }
            "jmpf" => {
                let (r1, imm) = single_reg_imm_18(handler, args, immediate, whole_op_span)?;
                VirtualOp::JMPF(r1, imm)
            }
            "jnzb" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::JNZB(r1, r2, imm)
            }
            "jnzf" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::JNZF(r1, r2, imm)
            }
            "jneb" => {
                let (r1, r2, r3, imm) = three_regs_imm_06(handler, args, immediate, whole_op_span)?;
                VirtualOp::JNEB(r1, r2, r3, imm)
            }
            "jnef" => {
                let (r1, r2, r3, imm) = three_regs_imm_06(handler, args, immediate, whole_op_span)?;
                VirtualOp::JNEF(r1, r2, r3, imm)
            }
            "jal" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::JAL(r1, r2, imm)
            }
            "ret" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::RET(r1)
            }

            /* Memory Instructions */
            "aloc" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::ALOC(VirtualRegister::Constant(ConstantRegister::HeapPointer), r1)
            }
            "cfei" => {
                let imm = single_imm_24(handler, args, immediate, whole_op_span)?;
                VirtualOp::CFEI(
                    VirtualRegister::Constant(ConstantRegister::StackPointer),
                    imm,
                )
            }
            "cfsi" => {
                let imm = single_imm_24(handler, args, immediate, whole_op_span)?;
                VirtualOp::CFSI(
                    VirtualRegister::Constant(ConstantRegister::StackPointer),
                    imm,
                )
            }
            "cfe" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::CFE(
                    VirtualRegister::Constant(ConstantRegister::StackPointer),
                    r1,
                )
            }
            "cfs" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::CFS(
                    VirtualRegister::Constant(ConstantRegister::StackPointer),
                    r1,
                )
            }
            "lb" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::LB(r1, r2, imm)
            }
            "lw" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::LW(r1, r2, imm)
            }
            "mcl" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MCL(r1, r2)
            }
            "mcli" => {
                let (r1, imm) = single_reg_imm_18(handler, args, immediate, whole_op_span)?;
                VirtualOp::MCLI(r1, imm)
            }
            "mcp" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MCP(r1, r2, r3)
            }
            "mcpi" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::MCPI(r1, r2, imm)
            }
            "meq" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MEQ(r1, r2, r3, r4)
            }
            "sb" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::SB(r1, r2, imm)
            }
            "sw" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::SW(r1, r2, imm)
            }

            /* Contract Instructions */
            "bal" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::BAL(r1, r2, r3)
            }
            "bhei" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::BHEI(r1)
            }
            "bhsh" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::BHSH(r1, r2)
            }
            "burn" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::BURN(r1, r2)
            }
            "call" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::CALL(r1, r2, r3, r4)
            }
            "cb" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::CB(r1)
            }
            "ccp" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::CCP(r1, r2, r3, r4)
            }
            "croo" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::CROO(r1, r2)
            }
            "csiz" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::CSIZ(r1, r2)
            }
            "bsiz" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::BSIZ(r1, r2)
            }
            "ldc" => {
                let (r1, r2, r3, i0) = three_regs_imm_06(handler, args, immediate, whole_op_span)?;
                VirtualOp::LDC(r1, r2, r3, i0)
            }
            "bldd" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::BLDD(r1, r2, r3, r4)
            }
            "log" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::LOG(r1, r2, r3, r4)
            }
            "logd" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::LOGD(r1, r2, r3, r4)
            }
            "mint" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::MINT(r1, r2)
            }
            "retd" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::RETD(r1, r2)
            }
            "rvrt" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::RVRT(r1)
            }
            "smo" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::SMO(r1, r2, r3, r4)
            }
            "scwq" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::SCWQ(r1, r2, r3)
            }
            "srw" => {
                let (r1, r2, r3, imm) = three_regs_imm_06(handler, args, immediate, whole_op_span)?;
                VirtualOp::SRW(r1, r2, r3, imm)
            }
            "srwq" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::SRWQ(r1, r2, r3, r4)
            }
            "sww" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::SWW(r1, r2, r3)
            }
            "swwq" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::SWWQ(r1, r2, r3, r4)
            }
            "time" => {
                let (r1, r2) = two_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::TIME(r1, r2)
            }
            "tr" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::TR(r1, r2, r3)
            }
            "tro" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::TRO(r1, r2, r3, r4)
            }

            /* Cryptographic Instructions */
            "eck1" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::ECK1(r1, r2, r3)
            }
            "ecr1" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::ECR1(r1, r2, r3)
            }
            "ed19" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::ED19(r1, r2, r3, r4)
            }
            "k256" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::K256(r1, r2, r3)
            }
            "s256" => {
                let (r1, r2, r3) = three_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::S256(r1, r2, r3)
            }
            "ecop" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::ECOP(r1, r2, r3, r4)
            }
            "epar" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::EPAR(r1, r2, r3, r4)
            }

            /* Other Instructions */
            "ecal" => {
                let (r1, r2, r3, r4) = four_regs(handler, args, immediate, whole_op_span)?;
                VirtualOp::ECAL(r1, r2, r3, r4)
            }
            "flag" => {
                let r1 = single_reg(handler, args, immediate, whole_op_span)?;
                VirtualOp::FLAG(r1)
            }
            "gm" => {
                let (r1, imm) = single_reg_imm_18(handler, args, immediate, whole_op_span)?;
                VirtualOp::GM(r1, imm)
            }
            "gtf" => {
                let (r1, r2, imm) = two_regs_imm_12(handler, args, immediate, whole_op_span)?;
                VirtualOp::GTF(r1, r2, imm)
            }

            /* Non-VM Instructions */
            "blob" => {
                let imm = single_imm_24(handler, args, immediate, whole_op_span)?;
                VirtualOp::BLOB(imm)
            }
            _ => {
                return Err(handler.emit_err(CompileError::UnrecognizedOp {
                    op_name: name.clone(),
                    span: name.span(),
                }));
            }
        })
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

    pub(crate) fn use_registers_mut(&mut self) -> BTreeSet<&mut VirtualRegister> {
        match &mut self.opcode {
            Either::Left(virt_op) => virt_op.use_registers_mut(),
            Either::Right(org_op) => org_op.use_registers_mut(),
        }
    }

    pub(crate) fn def_registers(&self) -> BTreeSet<&VirtualRegister> {
        match &self.opcode {
            Either::Left(virt_op) => virt_op.def_registers(),
            Either::Right(org_op) => org_op.def_registers(),
        }
    }

    pub(crate) fn def_const_registers(&self) -> BTreeSet<&VirtualRegister> {
        match &self.opcode {
            Either::Left(virt_op) => virt_op.def_const_registers(),
            Either::Right(org_op) => org_op.def_const_registers(),
        }
    }

    pub(crate) fn successors(
        &self,
        index: usize,
        ops: &[Op],
        label_to_index: &HashMap<Label, usize>,
    ) -> Vec<usize> {
        match &self.opcode {
            Either::Left(virt_op) => virt_op.successors(index, ops),
            Either::Right(org_op) => org_op.successors(index, ops, label_to_index),
        }
    }

    pub(crate) fn update_register(
        &self,
        reg_to_reg_map: &IndexMap<&VirtualRegister, &VirtualRegister>,
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
    ) -> Either<AllocatedInstruction, ControlFlowOp<AllocatedRegister>> {
        match &self.opcode {
            Either::Left(virt_op) => Either::Left(virt_op.allocate_registers(pool)),
            Either::Right(org_op) => Either::Right(org_op.allocate_registers(pool)),
        }
    }
}

fn single_reg(
    handler: &Handler,
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> Result<VirtualRegister, ErrorEmitted> {
    if args.len() > 1 {
        handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
            expected: 1,
            received: args.len(),
            span: whole_op_span.clone(),
        });
    }

    let reg = match args.first() {
        Some(reg) => reg,
        _ => {
            return Err(
                handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
                    span: whole_op_span,
                    expected: 1,
                    received: args.len(),
                }),
            );
        }
    };
    match immediate {
        None => (),
        Some(i) => {
            handler.emit_err(CompileError::UnnecessaryImmediate { span: i.span() });
        }
    };

    Ok(reg.clone())
}

fn two_regs(
    handler: &Handler,
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> Result<(VirtualRegister, VirtualRegister), ErrorEmitted> {
    if args.len() > 2 {
        handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 2,
            received: args.len(),
        });
    }

    let (reg, reg2) = match (args.first(), args.get(1)) {
        (Some(reg), Some(reg2)) => (reg, reg2),
        _ => {
            return Err(
                handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
                    span: whole_op_span,
                    expected: 2,
                    received: args.len(),
                }),
            );
        }
    };
    match immediate {
        None => (),
        Some(i) => {
            handler.emit_err(CompileError::UnnecessaryImmediate { span: i.span() });
        }
    };

    Ok((reg.clone(), reg2.clone()))
}

fn four_regs(
    handler: &Handler,
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> Result<
    (
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    ErrorEmitted,
> {
    if args.len() > 4 {
        handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 4,
            received: args.len(),
        });
    }

    let (reg, reg2, reg3, reg4) = match (args.first(), args.get(1), args.get(2), args.get(3)) {
        (Some(reg), Some(reg2), Some(reg3), Some(reg4)) => (reg, reg2, reg3, reg4),
        _ => {
            return Err(
                handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
                    span: whole_op_span,
                    expected: 4,
                    received: args.len(),
                }),
            );
        }
    };
    match immediate {
        None => (),
        Some(i) => {
            handler.emit_err(CompileError::MissingImmediate { span: i.span() });
        }
    };

    // Immediate Value.
    pub type ImmediateValue = u32;

    Ok((reg.clone(), reg2.clone(), reg3.clone(), reg4.clone()))
}

fn three_regs(
    handler: &Handler,
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> Result<(VirtualRegister, VirtualRegister, VirtualRegister), ErrorEmitted> {
    if args.len() > 3 {
        handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 3,
            received: args.len(),
        });
    }

    let (reg, reg2, reg3) = match (args.first(), args.get(1), args.get(2)) {
        (Some(reg), Some(reg2), Some(reg3)) => (reg, reg2, reg3),
        _ => {
            return Err(
                handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
                    span: whole_op_span,
                    expected: 3,
                    received: args.len(),
                }),
            );
        }
    };
    match immediate {
        None => (),
        Some(i) => {
            handler.emit_err(CompileError::UnnecessaryImmediate { span: i.span() });
        }
    };

    Ok((reg.clone(), reg2.clone(), reg3.clone()))
}
fn single_imm_24(
    handler: &Handler,
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> Result<VirtualImmediate24, ErrorEmitted> {
    if !args.is_empty() {
        handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 0,
            received: args.len(),
        });
    }
    let (imm, imm_span): (u64, _) = match immediate {
        None => {
            return Err(handler.emit_err(CompileError::MissingImmediate {
                span: whole_op_span,
            }));
        }
        Some(i) => match i.as_str()[1..].parse() {
            Ok(o) => (o, i.span()),
            Err(_) => {
                return Err(
                    handler.emit_err(CompileError::InvalidImmediateValue { span: i.span() })
                );
            }
        },
    };

    let imm = match VirtualImmediate24::try_new(imm, imm_span) {
        Ok(o) => o,
        Err(e) => {
            return Err(handler.emit_err(e));
        }
    };

    Ok(imm)
}
fn single_reg_imm_18(
    handler: &Handler,
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> Result<(VirtualRegister, VirtualImmediate18), ErrorEmitted> {
    if args.len() > 1 {
        handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 1,
            received: args.len(),
        });
    }
    let reg = match args.first() {
        Some(reg) => reg,
        _ => {
            return Err(
                handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
                    span: whole_op_span,
                    expected: 1,
                    received: args.len(),
                }),
            );
        }
    };
    let (imm, imm_span): (u64, _) = match immediate {
        None => {
            return Err(handler.emit_err(CompileError::MissingImmediate {
                span: whole_op_span,
            }));
        }
        Some(i) => match i.as_str()[1..].parse() {
            Ok(o) => (o, i.span()),
            Err(_) => {
                return Err(
                    handler.emit_err(CompileError::InvalidImmediateValue { span: i.span() })
                );
            }
        },
    };

    let imm = match VirtualImmediate18::try_new(imm, imm_span) {
        Ok(o) => o,
        Err(e) => {
            return Err(handler.emit_err(e));
        }
    };

    Ok((reg.clone(), imm))
}
fn two_regs_imm_12(
    handler: &Handler,
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> Result<(VirtualRegister, VirtualRegister, VirtualImmediate12), ErrorEmitted> {
    if args.len() > 2 {
        handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 2,
            received: args.len(),
        });
    }
    let (reg, reg2) = match (args.first(), args.get(1)) {
        (Some(reg), Some(reg2)) => (reg, reg2),
        _ => {
            return Err(
                handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
                    span: whole_op_span,
                    expected: 2,
                    received: args.len(),
                }),
            );
        }
    };
    let (imm, imm_span): (u64, _) = match immediate {
        None => {
            return Err(handler.emit_err(CompileError::MissingImmediate {
                span: whole_op_span,
            }));
        }
        Some(i) => match i.as_str()[1..].parse() {
            Ok(o) => (o, i.span()),
            Err(_) => {
                return Err(
                    handler.emit_err(CompileError::InvalidImmediateValue { span: i.span() })
                );
            }
        },
    };

    let imm = match VirtualImmediate12::try_new(imm, imm_span) {
        Ok(o) => o,
        Err(e) => {
            return Err(handler.emit_err(e));
        }
    };

    Ok((reg.clone(), reg2.clone(), imm))
}

fn three_regs_imm_06(
    handler: &Handler,
    args: &[VirtualRegister],
    immediate: &Option<Ident>,
    whole_op_span: Span,
) -> Result<
    (
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualImmediate06,
    ),
    ErrorEmitted,
> {
    if args.len() > 3 {
        handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
            span: whole_op_span.clone(),
            expected: 3,
            received: args.len(),
        });
    }
    let (reg, reg2, reg3) = match (args.first(), args.get(1), args.get(2)) {
        (Some(reg), Some(reg2), Some(reg3)) => (reg, reg2, reg3),
        _ => {
            return Err(
                handler.emit_err(CompileError::IncorrectNumberOfAsmRegisters {
                    span: whole_op_span,
                    expected: 3,
                    received: args.len(),
                }),
            );
        }
    };
    let (imm, imm_span): (u64, _) = match immediate {
        None => {
            return Err(handler.emit_err(CompileError::MissingImmediate {
                span: whole_op_span,
            }));
        }
        Some(i) => match i.as_str()[1..].parse() {
            Ok(o) => (o, i.span()),
            Err(_) => {
                return Err(
                    handler.emit_err(CompileError::InvalidImmediateValue { span: i.span() })
                );
            }
        },
    };

    let imm = match VirtualImmediate06::try_new(imm, imm_span) {
        Ok(o) => o,
        Err(e) => {
            return Err(handler.emit_err(e));
        }
    };

    Ok((reg.clone(), reg2.clone(), reg3.clone(), imm))
}

impl fmt::Display for Op {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_opcode_and_comment(self.opcode.to_string(), &self.comment, fmtr)
    }
}

impl fmt::Display for VirtualOp {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        use VirtualOp::*;
        match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(a, b, c) => write!(fmtr, "add {a} {b} {c}"),
            ADDI(a, b, c) => write!(fmtr, "addi {a} {b} {c}"),
            AND(a, b, c) => write!(fmtr, "and {a} {b} {c}"),
            ANDI(a, b, c) => write!(fmtr, "andi {a} {b} {c}"),
            DIV(a, b, c) => write!(fmtr, "div {a} {b} {c}"),
            DIVI(a, b, c) => write!(fmtr, "divi {a} {b} {c}"),
            EQ(a, b, c) => write!(fmtr, "eq {a} {b} {c}"),
            EXP(a, b, c) => write!(fmtr, "exp {a} {b} {c}"),
            EXPI(a, b, c) => write!(fmtr, "expi {a} {b} {c}"),
            GT(a, b, c) => write!(fmtr, "gt {a} {b} {c}"),
            LT(a, b, c) => write!(fmtr, "lt {a} {b} {c}"),
            MLOG(a, b, c) => write!(fmtr, "mlog {a} {b} {c}"),
            MOD(a, b, c) => write!(fmtr, "mod {a} {b} {c}"),
            MODI(a, b, c) => write!(fmtr, "modi {a} {b} {c}"),
            MOVE(a, b) => write!(fmtr, "move {a} {b}"),
            MOVI(a, b) => write!(fmtr, "movi {a} {b}"),
            MROO(a, b, c) => write!(fmtr, "mroo {a} {b} {c}"),
            MUL(a, b, c) => write!(fmtr, "mul {a} {b} {c}"),
            MULI(a, b, c) => write!(fmtr, "muli {a} {b} {c}"),
            NOOP => Ok(()),
            NOT(a, b) => write!(fmtr, "not {a} {b}"),
            OR(a, b, c) => write!(fmtr, "or {a} {b} {c}"),
            ORI(a, b, c) => write!(fmtr, "ori {a} {b} {c}"),
            SLL(a, b, c) => write!(fmtr, "sll {a} {b} {c}"),
            SLLI(a, b, c) => write!(fmtr, "slli {a} {b} {c}"),
            SRL(a, b, c) => write!(fmtr, "srl {a} {b} {c}"),
            SRLI(a, b, c) => write!(fmtr, "srli {a} {b} {c}"),
            SUB(a, b, c) => write!(fmtr, "sub {a} {b} {c}"),
            SUBI(a, b, c) => write!(fmtr, "subi {a} {b} {c}"),
            XOR(a, b, c) => write!(fmtr, "xor {a} {b} {c}"),
            XORI(a, b, c) => write!(fmtr, "xori {a} {b} {c}"),
            WQOP(a, b, c, d) => write!(fmtr, "wqop {a} {b} {c} {d}"),
            WQML(a, b, c, d) => write!(fmtr, "wqml {a} {b} {c} {d}"),
            WQDV(a, b, c, d) => write!(fmtr, "wqdv {a} {b} {c} {d}"),
            WQMD(a, b, c, d) => write!(fmtr, "wqmd {a} {b} {c} {d}"),
            WQCM(a, b, c, d) => write!(fmtr, "wqcm {a} {b} {c} {d}"),
            WQAM(a, b, c, d) => write!(fmtr, "wqam {a} {b} {c} {d}"),
            WQMM(a, b, c, d) => write!(fmtr, "wqmm {a} {b} {c} {d}"),

            /* Control Flow Instructions */
            JMP(a) => write!(fmtr, "jmp {a}"),
            JI(a) => write!(fmtr, "ji {a}"),
            JNE(a, b, c) => write!(fmtr, "jne {a} {b} {c}"),
            JNEI(a, b, c) => write!(fmtr, "jnei {a} {b} {c}"),
            JNZI(a, b) => write!(fmtr, "jnzi {a} {b}"),
            JMPB(a, b) => write!(fmtr, "jmpb {a} {b}"),
            JMPF(a, b) => write!(fmtr, "jmpf {a} {b}"),
            JNZB(a, b, c) => write!(fmtr, "jnzb {a} {b} {c}"),
            JNZF(a, b, c) => write!(fmtr, "jnzf {a} {b} {c}"),
            JNEB(a, b, c, d) => write!(fmtr, "jneb {a} {b} {c} {d}"),
            JNEF(a, b, c, d) => write!(fmtr, "jnef {a} {b} {c} {d}"),
            JAL(a, b, c) => write!(fmtr, "jal {a} {b} {c}"),
            RET(a) => write!(fmtr, "ret {a}"),

            /* Memory Instructions */
            ALOC(_hp, a) => write!(fmtr, "aloc {a}"),
            CFEI(_sp, a) => write!(fmtr, "cfei {a}"),
            CFSI(_sp, a) => write!(fmtr, "cfsi {a}"),
            CFE(_sp, a) => write!(fmtr, "cfe {a}"),
            CFS(_sp, a) => write!(fmtr, "cfs {a}"),
            LB(a, b, c) => write!(fmtr, "lb {a} {b} {c}"),
            LW(a, b, c) => write!(fmtr, "lw {a} {b} {c}"),
            MCL(a, b) => write!(fmtr, "mcl {a} {b}"),
            MCLI(a, b) => write!(fmtr, "mcli {a} {b}"),
            MCP(a, b, c) => write!(fmtr, "mcp {a} {b} {c}"),
            MCPI(a, b, c) => write!(fmtr, "mcpi {a} {b} {c}"),
            MEQ(a, b, c, d) => write!(fmtr, "meq {a} {b} {c} {d}"),
            SB(a, b, c) => write!(fmtr, "sb {a} {b} {c}"),
            SW(a, b, c) => write!(fmtr, "sw {a} {b} {c}"),

            /* Contract Instructions */
            BAL(a, b, c) => write!(fmtr, "bal {a} {b} {c}"),
            BHEI(a) => write!(fmtr, "bhei {a}"),
            BHSH(a, b) => write!(fmtr, "bhsh {a} {b}"),
            BURN(a, b) => write!(fmtr, "burn {a} {b}"),
            CALL(a, b, c, d) => write!(fmtr, "call {a} {b} {c} {d}"),
            CB(a) => write!(fmtr, "cb {a}"),
            CCP(a, b, c, d) => write!(fmtr, "ccp {a} {b} {c} {d}"),
            CROO(a, b) => write!(fmtr, "croo {a} {b}"),
            CSIZ(a, b) => write!(fmtr, "csiz {a} {b}"),
            BSIZ(a, b) => write!(fmtr, "bsiz {a} {b}"),
            LDC(a, b, c, d) => write!(fmtr, "ldc {a} {b} {c} {d}"),
            BLDD(a, b, c, d) => write!(fmtr, "bldd {a} {b} {c} {d}"),
            LOG(a, b, c, d) => write!(fmtr, "log {a} {b} {c} {d}"),
            LOGD(a, b, c, d) => write!(fmtr, "logd {a} {b} {c} {d}"),
            MINT(a, b) => write!(fmtr, "mint {a} {b}"),
            RETD(a, b) => write!(fmtr, "retd {a} {b}"),
            RVRT(a) => write!(fmtr, "rvrt {a}"),
            SMO(a, b, c, d) => write!(fmtr, "smo {a} {b} {c} {d}"),
            SCWQ(a, b, c) => write!(fmtr, "scwq {a} {b} {c}"),
            SRW(a, b, c, d) => write!(fmtr, "srw {a} {b} {c} {d}"),
            SRWQ(a, b, c, d) => write!(fmtr, "srwq {a} {b} {c} {d}"),
            SWW(a, b, c) => write!(fmtr, "sww {a} {b} {c}"),
            SWWQ(a, b, c, d) => write!(fmtr, "swwq {a} {b} {c} {d}"),
            TIME(a, b) => write!(fmtr, "time {a} {b}"),
            TR(a, b, c) => write!(fmtr, "tr {a} {b} {c}"),
            TRO(a, b, c, d) => write!(fmtr, "tro {a} {b} {c} {d}"),

            /* Cryptographic Instructions */
            ECK1(a, b, c) => write!(fmtr, "eck1 {a} {b} {c}"),
            ECR1(a, b, c) => write!(fmtr, "ecr1 {a} {b} {c}"),
            ED19(a, b, c, d) => write!(fmtr, "ed19 {a} {b} {c} {d}"),
            K256(a, b, c) => write!(fmtr, "k256 {a} {b} {c}"),
            S256(a, b, c) => write!(fmtr, "s256 {a} {b} {c}"),
            ECOP(a, b, c, d) => write!(fmtr, "ecop {a} {b} {c} {d}"),
            EPAR(a, b, c, d) => write!(fmtr, "epar {a} {b} {c} {d}"),

            /* Other Instructions */
            ECAL(a, b, c, d) => write!(fmtr, "ecal {a} {b} {c} {d}"),
            FLAG(a) => write!(fmtr, "flag {a}"),
            GM(a, b) => write!(fmtr, "gm {a} {b}"),
            GTF(a, b, c) => write!(fmtr, "gtf {a} {b} {c}"),

            /* Non-VM Instructions */
            BLOB(a) => write!(fmtr, "blob {a}"),
            DataSectionOffsetPlaceholder => write!(fmtr, "data section offset placeholder"),
            ConfigurablesOffsetPlaceholder => write!(fmtr, "configurables offset placeholder"),
            LoadDataId(a, b) => write!(fmtr, "load {a} {b}"),
            AddrDataId(a, b) => write!(fmtr, "addr {a} {b}"),
            Undefined => write!(fmtr, "undefined op"),
        }
    }
}

impl fmt::Display for AllocatedAbstractOp {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_opcode_and_comment(self.opcode.to_string(), &self.comment, fmtr)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum JumpType<Reg> {
    /// A simple unconditional jump
    Unconditional,
    /// A jump conditional on the register not being zero
    NotZero(Reg),
    /// Unconditional jump but semantically a call
    Call,
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
    Jump {
        /// Target label
        to: Label,
        /// Jump type
        type_: JumpType<Reg>,
    },
    // Placeholder for the offset into the configurables section.
    ConfigurablesOffsetPlaceholder,
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
                Label(lab) => format!("{lab}"),
                Jump { to, type_, .. } => match type_ {
                    JumpType::Unconditional => format!("ji  {to}"),
                    JumpType::NotZero(cond) => format!("jnzi {cond} {to}"),
                    JumpType::Call => format!("fncall {to}"),
                },
                Comment => "".into(),
                DataSectionOffsetPlaceholder =>
                    "DATA SECTION OFFSET[0..32]\nDATA SECTION OFFSET[32..64]".into(),
                ConfigurablesOffsetPlaceholder =>
                    "CONFIGURABLES_OFFSET[0..32]\nCONFIGURABLES_OFFSET[32..64]".into(),
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
            | DataSectionOffsetPlaceholder
            | ConfigurablesOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => vec![],

            Jump { type_, .. } => match type_ {
                JumpType::Unconditional => vec![],
                JumpType::NotZero(r1) => vec![r1],
                JumpType::Call => vec![],
            },
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn use_registers(&self) -> BTreeSet<&Reg> {
        use ControlFlowOp::*;
        (match self {
            Label(_)
            | Comment
            | DataSectionOffsetPlaceholder
            | ConfigurablesOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => vec![],

            Jump { type_, .. } => match type_ {
                JumpType::Unconditional => vec![],
                JumpType::NotZero(r1) => vec![r1],
                JumpType::Call => vec![],
            },
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn use_registers_mut(&mut self) -> BTreeSet<&mut Reg> {
        use ControlFlowOp::*;
        (match self {
            Label(_)
            | Comment
            | DataSectionOffsetPlaceholder
            | ConfigurablesOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => vec![],
            Jump { type_, .. } => match type_ {
                JumpType::Unconditional => vec![],
                JumpType::NotZero(r1) => vec![r1],
                JumpType::Call => vec![],
            },
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn def_registers(&self) -> BTreeSet<&Reg> {
        BTreeSet::new()
    }

    pub(crate) fn def_const_registers(&self) -> BTreeSet<&VirtualRegister> {
        BTreeSet::new()
    }

    pub(crate) fn update_register(&self, reg_to_reg_map: &IndexMap<&Reg, &Reg>) -> Self {
        let update_reg = |reg: &Reg| -> Reg { (*reg_to_reg_map.get(reg).unwrap_or(&reg)).clone() };

        use ControlFlowOp::*;
        match self {
            Comment
            | Label(_)
            | DataSectionOffsetPlaceholder
            | ConfigurablesOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => self.clone(),

            Jump { to, type_ } => match type_ {
                JumpType::NotZero(r1) => Self::Jump {
                    to: *to,
                    type_: JumpType::NotZero(update_reg(r1)),
                },
                _ => self.clone(),
            },
        }
    }

    pub(crate) fn successors(
        &self,
        index: usize,
        ops: &[Op],
        label_to_index: &HashMap<Label, usize>,
    ) -> Vec<usize> {
        use ControlFlowOp::*;

        let mut next_ops = Vec::new();

        match self {
            Label(_)
            | Comment
            | DataSectionOffsetPlaceholder
            | ConfigurablesOffsetPlaceholder
            | PushAll(_)
            | PopAll(_) => {
                if index + 1 < ops.len() {
                    next_ops.push(index + 1);
                }
            }

            Jump { to, type_, .. } => match type_ {
                JumpType::Unconditional => {
                    next_ops.push(label_to_index[to]);
                }
                JumpType::NotZero(_) => {
                    next_ops.push(label_to_index[to]);
                    if index + 1 < ops.len() {
                        next_ops.push(index + 1);
                    }
                }
                JumpType::Call => {
                    if index + 1 < ops.len() {
                        next_ops.push(index + 1);
                    }
                }
            },
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
            Jump { to, type_ } => Jump {
                to: *to,
                type_: match type_ {
                    JumpType::NotZero(r1) => JumpType::NotZero(map_reg(r1)),
                    JumpType::Unconditional => JumpType::Unconditional,
                    JumpType::Call => JumpType::Call,
                },
            },
            DataSectionOffsetPlaceholder => DataSectionOffsetPlaceholder,
            ConfigurablesOffsetPlaceholder => ConfigurablesOffsetPlaceholder,
            PushAll(label) => PushAll(*label),
            PopAll(label) => PopAll(*label),
        }
    }
}
