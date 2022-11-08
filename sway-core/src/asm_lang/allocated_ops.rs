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

use super::DataId;
use super::*;
use crate::{
    asm_generation::{compiler_constants::DATA_SECTION_REGISTER, Entry, VirtualDataSection},
    fuel_prelude::fuel_asm::{self, Opcode as VmOp},
};
use either::Either;
use std::fmt::{self, Write};
use sway_types::span::Span;

const COMMENT_START_COLUMN: usize = 30;

/// Represents registers that have gone through register allocation. The value in the [Allocated]
/// variant is guaranteed to be between 0 and [compiler_constants::NUM_ALLOCATABLE_REGISTERS].
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum AllocatedRegister {
    Allocated(u8),
    Constant(super::ConstantRegister),
}

impl fmt::Display for AllocatedRegister {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocatedRegister::Allocated(name) => write!(fmtr, "$r{}", name),
            AllocatedRegister::Constant(name) => {
                write!(fmtr, "{}", name)
            }
        }
    }
}

impl AllocatedRegister {
    fn to_register_id(&self) -> fuel_asm::RegisterId {
        match self {
            AllocatedRegister::Allocated(a) => (a + 16) as fuel_asm::RegisterId,
            AllocatedRegister::Constant(constant) => constant.to_register_id(),
        }
    }
}

/// This enum is unfortunately a redundancy of the [fuel_asm::Opcode] and [crate::VirtualOp] enums. This variant, however,
/// allows me to use the compiler's internal [AllocatedRegister] types and maintain type safety
/// between virtual ops and those which have gone through register allocation.
/// A bit of copy/paste seemed worth it for that safety,
/// so here it is.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub(crate) enum AllocatedOpcode {
    /* Arithmetic/Logic (ALU) Instructions */
    ADD(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    ADDI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    AND(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    ANDI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    DIV(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    DIVI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    EQ(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    EXP(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    EXPI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    GT(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    LT(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MLOG(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MOD(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MODI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    MOVE(AllocatedRegister, AllocatedRegister),
    MOVI(AllocatedRegister, VirtualImmediate18),
    MROO(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MUL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MULI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    NOOP,
    NOT(AllocatedRegister, AllocatedRegister),
    OR(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    ORI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SLL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SLLI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SRL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SRLI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SUB(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SUBI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    XOR(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    XORI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /* Conrol Flow Instructions */
    JMP(AllocatedRegister),
    JI(VirtualImmediate24),
    JNE(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    JNEI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    JNZI(AllocatedRegister, VirtualImmediate18),
    RET(AllocatedRegister),

    /* Memory Instructions */
    ALOC(AllocatedRegister),
    CFEI(VirtualImmediate24),
    CFSI(VirtualImmediate24),
    LB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    LW(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    MCL(AllocatedRegister, AllocatedRegister),
    MCLI(AllocatedRegister, VirtualImmediate18),
    MCP(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MCPI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    MEQ(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    SB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SW(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /* Contract Instructions */
    BAL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    BHEI(AllocatedRegister),
    BHSH(AllocatedRegister, AllocatedRegister),
    BURN(AllocatedRegister),
    CALL(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    CB(AllocatedRegister),
    CCP(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    CROO(AllocatedRegister, AllocatedRegister),
    CSIZ(AllocatedRegister, AllocatedRegister),
    LDC(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    LOG(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    LOGD(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    MINT(AllocatedRegister),
    RETD(AllocatedRegister, AllocatedRegister),
    RVRT(AllocatedRegister),
    SMO(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    SCWQ(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SRW(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SRWQ(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    SWW(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SWWQ(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    TIME(AllocatedRegister, AllocatedRegister),
    TR(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    TRO(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),

    /* Cryptographic Instructions */
    ECR(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    K256(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    S256(AllocatedRegister, AllocatedRegister, AllocatedRegister),

    /* Other Instructions */
    FLAG(AllocatedRegister),
    GM(AllocatedRegister, VirtualImmediate18),
    GTF(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /* Non-VM Instructions */
    BLOB(VirtualImmediate24),
    DataSectionOffsetPlaceholder,
    DataSectionRegisterLoadPlaceholder,
    LWDataId(AllocatedRegister, DataId),
    Undefined,
}

impl AllocatedOpcode {
    pub(crate) fn def_registers(&self) -> BTreeSet<&AllocatedRegister> {
        use AllocatedOpcode::*;
        (match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(r1, _r2, _r3) => vec![r1],
            ADDI(r1, _r2, _i) => vec![r1],
            AND(r1, _r2, _r3) => vec![r1],
            ANDI(r1, _r2, _i) => vec![r1],
            DIV(r1, _r2, _r3) => vec![r1],
            DIVI(r1, _r2, _i) => vec![r1],
            EQ(r1, _r2, _r3) => vec![r1],
            EXP(r1, _r2, _r3) => vec![r1],
            EXPI(r1, _r2, _i) => vec![r1],
            GT(r1, _r2, _r3) => vec![r1],
            LT(r1, _r2, _r3) => vec![r1],
            MLOG(r1, _r2, _r3) => vec![r1],
            MOD(r1, _r2, _r3) => vec![r1],
            MODI(r1, _r2, _i) => vec![r1],
            MOVE(r1, _r2) => vec![r1],
            MOVI(r1, _i) => vec![r1],
            MROO(r1, _r2, _r3) => vec![r1],
            MUL(r1, _r2, _r3) => vec![r1],
            MULI(r1, _r2, _i) => vec![r1],
            NOOP => vec![],
            NOT(r1, _r2) => vec![r1],
            OR(r1, _r2, _r3) => vec![r1],
            ORI(r1, _r2, _i) => vec![r1],
            SLL(r1, _r2, _r3) => vec![r1],
            SLLI(r1, _r2, _i) => vec![r1],
            SRL(r1, _r2, _r3) => vec![r1],
            SRLI(r1, _r2, _i) => vec![r1],
            SUB(r1, _r2, _r3) => vec![r1],
            SUBI(r1, _r2, _i) => vec![r1],
            XOR(r1, _r2, _r3) => vec![r1],
            XORI(r1, _r2, _i) => vec![r1],

            /* Control Flow Instructions */
            JMP(_r1) => vec![],
            JI(_im) => vec![],
            JNE(_r1, _r2, _r3) => vec![],
            JNEI(_r1, _r2, _i) => vec![],
            JNZI(_r1, _i) => vec![],
            RET(_r1) => vec![],

            /* Memory Instructions */
            ALOC(_r1) => vec![],
            CFEI(_imm) => vec![],
            CFSI(_imm) => vec![],
            LB(r1, _r2, _i) => vec![r1],
            LW(r1, _r2, _i) => vec![r1],
            MCL(_r1, _r2) => vec![],
            MCLI(_r1, _imm) => vec![],
            MCP(_r1, _r2, _r3) => vec![],
            MCPI(_r1, _r2, _imm) => vec![],
            MEQ(r1, _r2, _r3, _r4) => vec![r1],
            SB(_r1, _r2, _i) => vec![],
            SW(_r1, _r2, _i) => vec![],

            /* Contract Instructions */
            BAL(r1, _r2, _r3) => vec![r1],
            BHEI(r1) => vec![r1],
            BHSH(_r1, _r2) => vec![],
            BURN(_r1) => vec![],
            CALL(_r1, _r2, _r3, _r4) => vec![],
            CB(_r1) => vec![],
            CCP(_r1, _r2, _r3, _r4) => vec![],
            CROO(_r1, _r2) => vec![],
            CSIZ(r1, _r2) => vec![r1],
            LDC(_r1, _r2, _r3) => vec![],
            LOG(_r1, _r2, _r3, _r4) => vec![],
            LOGD(_r1, _r2, _r3, _r4) => vec![],
            MINT(_r1) => vec![],
            RETD(_r1, _r2) => vec![],
            RVRT(_r1) => vec![],
            SMO(_r1, _r2, _r3, _r4) => vec![],
            SCWQ(_r1, r2, _r3) => vec![r2],
            SRW(r1, r2, _r3) => vec![r1, r2],
            SRWQ(_r1, r2, _r3, _r4) => vec![r2],
            SWW(_r1, r2, _r3) => vec![r2],
            SWWQ(_r1, r2, _r3, _r4) => vec![r2],
            TIME(r1, _r2) => vec![r1],
            TR(_r1, _r2, _r3) => vec![],
            TRO(_r1, _r2, _r3, _r4) => vec![],

            /* Cryptographic Instructions */
            ECR(_r1, _r2, _r3) => vec![],
            K256(_r1, _r2, _r3) => vec![],
            S256(_r1, _r2, _r3) => vec![],

            /* Other Instructions */
            FLAG(_r1) => vec![],
            GM(r1, _imm) => vec![r1],
            GTF(r1, _r2, _i) => vec![r1],

            /* Non-VM Instructions */
            BLOB(_imm) => vec![],
            DataSectionOffsetPlaceholder => vec![],
            DataSectionRegisterLoadPlaceholder => vec![&AllocatedRegister::Constant(
                ConstantRegister::DataSectionStart,
            )],
            LWDataId(r1, _i) => vec![r1],
            Undefined => vec![],
        })
        .into_iter()
        .collect()
    }
}

impl fmt::Display for AllocatedOpcode {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AllocatedOpcode::*;
        match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(a, b, c) => write!(fmtr, "add  {} {} {}", a, b, c),
            ADDI(a, b, c) => write!(fmtr, "addi {} {} {}", a, b, c),
            AND(a, b, c) => write!(fmtr, "and  {} {} {}", a, b, c),
            ANDI(a, b, c) => write!(fmtr, "andi {} {} {}", a, b, c),
            DIV(a, b, c) => write!(fmtr, "div  {} {} {}", a, b, c),
            DIVI(a, b, c) => write!(fmtr, "divi {} {} {}", a, b, c),
            EQ(a, b, c) => write!(fmtr, "eq   {} {} {}", a, b, c),
            EXP(a, b, c) => write!(fmtr, "exp  {} {} {}", a, b, c),
            EXPI(a, b, c) => write!(fmtr, "expi {} {} {}", a, b, c),
            GT(a, b, c) => write!(fmtr, "gt   {} {} {}", a, b, c),
            LT(a, b, c) => write!(fmtr, "lt   {} {} {}", a, b, c),
            MLOG(a, b, c) => write!(fmtr, "mlog {} {} {}", a, b, c),
            MOD(a, b, c) => write!(fmtr, "mod  {} {} {}", a, b, c),
            MODI(a, b, c) => write!(fmtr, "modi {} {} {}", a, b, c),
            MOVE(a, b) => write!(fmtr, "move {} {}", a, b),
            MOVI(a, b) => write!(fmtr, "movi {} {}", a, b),
            MROO(a, b, c) => write!(fmtr, "mroo {} {} {}", a, b, c),
            MUL(a, b, c) => write!(fmtr, "mul  {} {} {}", a, b, c),
            MULI(a, b, c) => write!(fmtr, "muli {} {} {}", a, b, c),
            NOOP => write!(fmtr, "noop"),
            NOT(a, b) => write!(fmtr, "not  {} {}", a, b),
            OR(a, b, c) => write!(fmtr, "or   {} {} {}", a, b, c),
            ORI(a, b, c) => write!(fmtr, "ori  {} {} {}", a, b, c),
            SLL(a, b, c) => write!(fmtr, "sll  {} {} {}", a, b, c),
            SLLI(a, b, c) => write!(fmtr, "slli {} {} {}", a, b, c),
            SRL(a, b, c) => write!(fmtr, "srl  {} {} {}", a, b, c),
            SRLI(a, b, c) => write!(fmtr, "srli {} {} {}", a, b, c),
            SUB(a, b, c) => write!(fmtr, "sub  {} {} {}", a, b, c),
            SUBI(a, b, c) => write!(fmtr, "subi {} {} {}", a, b, c),
            XOR(a, b, c) => write!(fmtr, "xor  {} {} {}", a, b, c),
            XORI(a, b, c) => write!(fmtr, "xori {} {} {}", a, b, c),

            /* Control Flow Instructions */
            JMP(a) => write!(fmtr, "jmp {}", a),
            JI(a) => write!(fmtr, "ji   {}", a),
            JNE(a, b, c) => write!(fmtr, "jne  {a} {b} {c}"),
            JNEI(a, b, c) => write!(fmtr, "jnei {} {} {}", a, b, c),
            JNZI(a, b) => write!(fmtr, "jnzi {} {}", a, b),
            RET(a) => write!(fmtr, "ret  {}", a),

            /* Memory Instructions */
            ALOC(a) => write!(fmtr, "aloc {}", a),
            CFEI(a) => write!(fmtr, "cfei {}", a),
            CFSI(a) => write!(fmtr, "cfsi {}", a),
            LB(a, b, c) => write!(fmtr, "lb   {} {} {}", a, b, c),
            LW(a, b, c) => write!(fmtr, "lw   {} {} {}", a, b, c),
            MCL(a, b) => write!(fmtr, "mcl  {} {}", a, b),
            MCLI(a, b) => write!(fmtr, "mcli {} {}", a, b),
            MCP(a, b, c) => write!(fmtr, "mcp  {} {} {}", a, b, c),
            MCPI(a, b, c) => write!(fmtr, "mcpi {} {} {}", a, b, c),
            MEQ(a, b, c, d) => write!(fmtr, "meq  {} {} {} {}", a, b, c, d),
            SB(a, b, c) => write!(fmtr, "sb   {} {} {}", a, b, c),
            SW(a, b, c) => write!(fmtr, "sw   {} {} {}", a, b, c),

            /* Contract Instructions */
            BAL(a, b, c) => write!(fmtr, "bal  {} {} {}", a, b, c),
            BHEI(a) => write!(fmtr, "bhei {}", a),
            BHSH(a, b) => write!(fmtr, "bhsh {} {}", a, b),
            BURN(a) => write!(fmtr, "burn {}", a),
            CALL(a, b, c, d) => write!(fmtr, "call {} {} {} {}", a, b, c, d),
            CB(a) => write!(fmtr, "cb   {}", a),
            CCP(a, b, c, d) => write!(fmtr, "ccp  {} {} {} {}", a, b, c, d),
            CROO(a, b) => write!(fmtr, "croo {} {}", a, b),
            CSIZ(a, b) => write!(fmtr, "csiz {} {}", a, b),
            LDC(a, b, c) => write!(fmtr, "ldc  {} {} {}", a, b, c),
            LOG(a, b, c, d) => write!(fmtr, "log  {} {} {} {}", a, b, c, d),
            LOGD(a, b, c, d) => write!(fmtr, "logd {} {} {} {}", a, b, c, d),
            MINT(a) => write!(fmtr, "mint {}", a),
            RETD(a, b) => write!(fmtr, "retd  {} {}", a, b),
            RVRT(a) => write!(fmtr, "rvrt {}", a),
            SMO(a, b, c, d) => write!(fmtr, "smo  {} {} {} {}", a, b, c, d),
            SCWQ(a, b, c) => write!(fmtr, "scwq  {} {} {}", a, b, c),
            SRW(a, b, c) => write!(fmtr, "srw  {} {} {}", a, b, c),
            SRWQ(a, b, c, d) => write!(fmtr, "srwq {} {} {} {}", a, b, c, d),
            SWW(a, b, c) => write!(fmtr, "sww  {} {} {}", a, b, c),
            SWWQ(a, b, c, d) => write!(fmtr, "swwq {} {} {} {}", a, b, c, d),
            TIME(a, b) => write!(fmtr, "time {} {}", a, b),
            TR(a, b, c) => write!(fmtr, "tr   {} {} {}", a, b, c),
            TRO(a, b, c, d) => write!(fmtr, "tro  {} {} {} {}", a, b, c, d),

            /* Cryptographic Instructions */
            ECR(a, b, c) => write!(fmtr, "ecr  {} {} {}", a, b, c),
            K256(a, b, c) => write!(fmtr, "k256 {} {} {}", a, b, c),
            S256(a, b, c) => write!(fmtr, "s256 {} {} {}", a, b, c),

            /* Other Instructions */
            FLAG(a) => write!(fmtr, "flag {}", a),
            GM(a, b) => write!(fmtr, "gm {} {}", a, b),
            GTF(a, b, c) => write!(fmtr, "gtf  {} {} {}", a, b, c),

            /* Non-VM Instructions */
            BLOB(a) => write!(fmtr, "blob {a}"),
            DataSectionOffsetPlaceholder => {
                write!(
                    fmtr,
                    "DATA_SECTION_OFFSET[0..32]\nDATA_SECTION_OFFSET[32..64]"
                )
            }
            DataSectionRegisterLoadPlaceholder => write!(fmtr, "lw   $ds $is 1"),
            LWDataId(a, b) => write!(fmtr, "lw   {} {}", a, b),
            Undefined => write!(fmtr, "undefined op"),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AllocatedOp {
    pub(crate) opcode: AllocatedOpcode,
    /// A descriptive comment for ASM readability
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span>,
}

impl fmt::Display for AllocatedOp {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        // We want the comment to always be COMMENT_START_COLUMN characters offset to the right to
        // not interfere with the ASM but to be aligned.
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

type DoubleWideData = [u8; 8];

impl AllocatedOp {
    pub(crate) fn to_fuel_asm(
        &self,
        offset_to_data_section: u64,
        data_section: &mut VirtualDataSection,
    ) -> Either<Vec<fuel_asm::Opcode>, DoubleWideData> {
        use AllocatedOpcode::*;
        Either::Left(vec![match &self.opcode {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(a, b, c) => VmOp::ADD(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            ADDI(a, b, c) => VmOp::ADDI(a.to_register_id(), b.to_register_id(), c.value),
            AND(a, b, c) => VmOp::AND(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            ANDI(a, b, c) => VmOp::ANDI(a.to_register_id(), b.to_register_id(), c.value),
            DIV(a, b, c) => VmOp::DIV(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            DIVI(a, b, c) => VmOp::DIVI(a.to_register_id(), b.to_register_id(), c.value),
            EQ(a, b, c) => VmOp::EQ(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            EXP(a, b, c) => VmOp::EXP(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            EXPI(a, b, c) => VmOp::EXPI(a.to_register_id(), b.to_register_id(), c.value),
            GT(a, b, c) => VmOp::GT(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            LT(a, b, c) => VmOp::LT(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MLOG(a, b, c) => VmOp::MLOG(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MOD(a, b, c) => VmOp::MOD(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MODI(a, b, c) => VmOp::MODI(a.to_register_id(), b.to_register_id(), c.value),
            MOVE(a, b) => VmOp::MOVE(a.to_register_id(), b.to_register_id()),
            MOVI(a, b) => VmOp::MOVI(a.to_register_id(), b.value),
            MROO(a, b, c) => VmOp::MROO(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MUL(a, b, c) => VmOp::MUL(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MULI(a, b, c) => VmOp::MULI(a.to_register_id(), b.to_register_id(), c.value),
            NOOP => VmOp::NOOP,
            NOT(a, b) => VmOp::NOT(a.to_register_id(), b.to_register_id()),
            OR(a, b, c) => VmOp::OR(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            ORI(a, b, c) => VmOp::ORI(a.to_register_id(), b.to_register_id(), c.value),
            SLL(a, b, c) => VmOp::SLL(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SLLI(a, b, c) => VmOp::SLLI(a.to_register_id(), b.to_register_id(), c.value),
            SRL(a, b, c) => VmOp::SRL(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SRLI(a, b, c) => VmOp::SRLI(a.to_register_id(), b.to_register_id(), c.value),
            SUB(a, b, c) => VmOp::SUB(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SUBI(a, b, c) => VmOp::SUBI(a.to_register_id(), b.to_register_id(), c.value),
            XOR(a, b, c) => VmOp::XOR(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            XORI(a, b, c) => VmOp::XORI(a.to_register_id(), b.to_register_id(), c.value),

            /* Control Flow Instructions */
            JMP(a) => VmOp::JMP(a.to_register_id()),
            JI(a) => VmOp::JI(a.value),
            JNE(a, b, c) => VmOp::JNE(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            JNEI(a, b, c) => VmOp::JNEI(a.to_register_id(), b.to_register_id(), c.value),
            JNZI(a, b) => VmOp::JNZI(a.to_register_id(), b.value),
            RET(a) => VmOp::RET(a.to_register_id()),

            /* Memory Instructions */
            ALOC(a) => VmOp::ALOC(a.to_register_id()),
            CFEI(a) => VmOp::CFEI(a.value),
            CFSI(a) => VmOp::CFSI(a.value),
            LB(a, b, c) => VmOp::LB(a.to_register_id(), b.to_register_id(), c.value),
            LW(a, b, c) => VmOp::LW(a.to_register_id(), b.to_register_id(), c.value),
            MCL(a, b) => VmOp::MCL(a.to_register_id(), b.to_register_id()),
            MCLI(a, b) => VmOp::MCLI(a.to_register_id(), b.value),
            MCP(a, b, c) => VmOp::MCP(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MCPI(a, b, c) => VmOp::MCPI(a.to_register_id(), b.to_register_id(), c.value),
            MEQ(a, b, c, d) => VmOp::MEQ(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),
            SB(a, b, c) => VmOp::SB(a.to_register_id(), b.to_register_id(), c.value),
            SW(a, b, c) => VmOp::SW(a.to_register_id(), b.to_register_id(), c.value),

            /* Contract Instructions */
            BAL(a, b, c) => VmOp::BAL(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            BHEI(a) => VmOp::BHEI(a.to_register_id()),
            BHSH(a, b) => VmOp::BHSH(a.to_register_id(), b.to_register_id()),
            BURN(a) => VmOp::BURN(a.to_register_id()),
            CALL(a, b, c, d) => VmOp::CALL(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),
            CB(a) => VmOp::CB(a.to_register_id()),
            CCP(a, b, c, d) => VmOp::CCP(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),
            CROO(a, b) => VmOp::CROO(a.to_register_id(), b.to_register_id()),
            CSIZ(a, b) => VmOp::CSIZ(a.to_register_id(), b.to_register_id()),
            LDC(a, b, c) => VmOp::LDC(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            LOG(a, b, c, d) => VmOp::LOG(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),
            LOGD(a, b, c, d) => VmOp::LOGD(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),
            MINT(a) => VmOp::MINT(a.to_register_id()),
            RETD(a, b) => VmOp::RETD(a.to_register_id(), b.to_register_id()),
            RVRT(a) => VmOp::RVRT(a.to_register_id()),
            SMO(a, b, c, d) => VmOp::SMO(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),
            SCWQ(a, b, c) => VmOp::SRW(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SRW(a, b, c) => VmOp::SRW(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SRWQ(a, b, c, d) => VmOp::SRWQ(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),
            SWW(a, b, c) => VmOp::SWW(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SWWQ(a, b, c, d) => VmOp::SWWQ(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),
            TIME(a, b) => VmOp::TIME(a.to_register_id(), b.to_register_id()),
            TR(a, b, c) => VmOp::TR(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            TRO(a, b, c, d) => VmOp::TRO(
                a.to_register_id(),
                b.to_register_id(),
                c.to_register_id(),
                d.to_register_id(),
            ),

            /* Cryptographic Instructions */
            ECR(a, b, c) => VmOp::ECR(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            K256(a, b, c) => VmOp::K256(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            S256(a, b, c) => VmOp::S256(a.to_register_id(), b.to_register_id(), c.to_register_id()),

            /* Other Instructions */
            FLAG(a) => VmOp::FLAG(a.to_register_id()),
            GM(a, b) => VmOp::GM(a.to_register_id(), b.value),
            GTF(a, b, c) => VmOp::GTF(a.to_register_id(), b.to_register_id(), c.value),

            /* Non-VM Instructions */
            BLOB(a) => {
                return Either::Left(
                    std::iter::repeat(VmOp::NOOP)
                        .take(a.value as usize)
                        .collect(),
                )
            }
            DataSectionOffsetPlaceholder => {
                return Either::Right(offset_to_data_section.to_be_bytes())
            }
            DataSectionRegisterLoadPlaceholder => VmOp::LW(
                DATA_SECTION_REGISTER as fuel_asm::RegisterId,
                ConstantRegister::InstructionStart.to_register_id(),
                1,
            ),
            LWDataId(a, b) => {
                return Either::Left(realize_lw(a, b, data_section, offset_to_data_section))
            }
            Undefined => VmOp::Undefined,
        }])
    }
}

/// Converts a virtual load word instruction which uses data labels into one which uses
/// actual bytewise offsets for use in bytecode.
/// Returns one op if the type is less than one word big, but two ops if it has to construct
/// a pointer and add it to $is.
fn realize_lw(
    dest: &AllocatedRegister,
    data_id: &DataId,
    data_section: &mut VirtualDataSection,
    offset_to_data_section: u64,
) -> Vec<VmOp> {
    // all data is word-aligned right now, and `offset_to_id` returns the offset in bytes
    let offset_bytes = data_section.offset_to_id(data_id) as u64;
    let offset_words = offset_bytes / 8;
    let offset = match VirtualImmediate12::new(offset_words, Span::new(" ".into(), 0, 0, None).unwrap()) {
        Ok(value) => value,
        Err(_) => panic!("Unable to offset into the data section more than 2^12 bits. Unsupported data section length.")
    };
    // if this data is larger than a word, instead of loading the data directly
    // into the register, we want to load a pointer to the data into the register
    // this appends onto the data section and mutates it by adding the pointer as a literal
    let has_copy_type = data_section.has_copy_type(data_id).expect(
        "Internal miscalculation in data section -- data id did not match up to any actual data",
    );
    if !has_copy_type {
        // load the pointer itself into the register
        // `offset_to_data_section` is in bytes. We want a byte
        // address here
        let pointer_offset_from_instruction_start = offset_to_data_section + offset_bytes;
        // insert the pointer as bytes as a new data section entry at the end of the data
        let data_id_for_pointer = data_section
            .insert_data_value(Entry::new_word(pointer_offset_from_instruction_start, None));
        // now load the pointer we just created into the `dest`ination
        let mut buf = Vec::with_capacity(2);
        buf.append(&mut realize_lw(
            dest,
            &data_id_for_pointer,
            data_section,
            offset_to_data_section,
        ));
        // add $is to the pointer since it is relative to the data section
        buf.push(VmOp::ADD(
            dest.to_register_id(),
            dest.to_register_id(),
            ConstantRegister::InstructionStart.to_register_id(),
        ));
        buf
    } else {
        vec![VmOp::LW(
            dest.to_register_id(),
            DATA_SECTION_REGISTER as usize,
            offset.value,
        )]
    }
}
