//! This module contains things that I need from the VM to build that we will eventually import
//! from the VM when it is ready.
//! Basically this is copy-pasted until things are public and it can be properly imported.
//!
//! Only things needed for opcode serialization and generation are included here.
#![allow(dead_code)]

use crate::{asm_generation::DataId, error::*, parse_tree::AsmRegister, Ident};
use either::Either;
use pest::Span;
use std::str::FromStr;
use std::{collections::HashSet, fmt};
use virtual_ops::{
    Label, VirtualImmediate06, VirtualImmediate12, VirtualImmediate18, VirtualImmediate24,
    VirtualOp, VirtualRegister,
};

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
            opcode: Either::Left(VirtualOp::JNEI(reg0, reg1, label)),
            comment: String::new(),
            owning_span: None,
        }
    }

    pub(crate) fn parse_opcode(
        name: &Ident<'sc>,
        args: &[&Ident<'sc>],
    ) -> CompileResult<'sc, VirtualOp> {
        todo!()
    }
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
    // Loads from the data section into a register
    // "load data"
    Ld(VirtualRegister, DataId),
}

impl OrganizationalOp {
    pub(crate) fn registers(&mut self) -> HashSet<&mut VirtualRegister> {
        use OrganizationalOp::*;
        (match self {
            Label(_) | Comment | Jump(_) => vec![],
            Ld(r1, _) => vec![r1],
        })
        .into_iter()
        .collect()
    }
}
