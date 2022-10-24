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
use crate::asm_generation::DataSection;
use either::Either;
use fuel_asm::Opcode as VmOp;
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
    GTF(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    LT(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MLOG(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MROO(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MOD(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MODI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    MOVE(AllocatedRegister, AllocatedRegister),
    MOVI(AllocatedRegister, VirtualImmediate18),
    MUL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MULI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    NOT(AllocatedRegister, AllocatedRegister),
    OR(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    ORI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SLL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SLLI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SMO(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    SRL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SRLI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SUB(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SUBI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    XOR(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    XORI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    JI(VirtualImmediate24),
    JNEI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    JNZI(AllocatedRegister, VirtualImmediate18),
    JMP(AllocatedRegister),
    JNE(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    RET(AllocatedRegister),
    RETD(AllocatedRegister, AllocatedRegister),
    CFEI(VirtualImmediate24),
    CFSI(VirtualImmediate24),
    LB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    LWDataId(AllocatedRegister, DataId),
    LW(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    ALOC(AllocatedRegister),
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
    BAL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    BHSH(AllocatedRegister, AllocatedRegister),
    BHEI(AllocatedRegister),
    BURN(AllocatedRegister),
    CALL(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    CCP(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    CROO(AllocatedRegister, AllocatedRegister),
    CSIZ(AllocatedRegister, AllocatedRegister),
    CB(AllocatedRegister),
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
    RVRT(AllocatedRegister),
    SRW(AllocatedRegister, AllocatedRegister),
    SRWQ(AllocatedRegister, AllocatedRegister),
    SWW(AllocatedRegister, AllocatedRegister),
    SWWQ(AllocatedRegister, AllocatedRegister),
    TIME(AllocatedRegister, AllocatedRegister),
    TR(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    TRO(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    ECR(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    K256(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    S256(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    NOOP,
    BLOB(VirtualImmediate24),
    FLAG(AllocatedRegister),
    GM(AllocatedRegister, VirtualImmediate18),
    Undefined,
    DataSectionOffsetPlaceholder,
    DataSectionRegisterLoadPlaceholder,
}

impl AllocatedOpcode {
    pub(crate) fn def_registers(&self) -> BTreeSet<&AllocatedRegister> {
        use AllocatedOpcode::*;
        (match self {
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
            GTF(r1, _r2, _i) => vec![r1],
            LT(r1, _r2, _r3) => vec![r1],
            MLOG(r1, _r2, _r3) => vec![r1],
            MROO(r1, _r2, _r3) => vec![r1],
            MOD(r1, _r2, _r3) => vec![r1],
            MODI(r1, _r2, _i) => vec![r1],
            MOVE(r1, _r2) => vec![r1],
            MOVI(r1, _i) => vec![r1],
            MUL(r1, _r2, _r3) => vec![r1],
            MULI(r1, _r2, _i) => vec![r1],
            NOT(r1, _r2) => vec![r1],
            OR(r1, _r2, _r3) => vec![r1],
            ORI(r1, _r2, _i) => vec![r1],
            SLL(r1, _r2, _r3) => vec![r1],
            SLLI(r1, _r2, _i) => vec![r1],
            SMO(_r1, _r2, _r3, _r4) => vec![],
            SRL(r1, _r2, _r3) => vec![r1],
            SRLI(r1, _r2, _i) => vec![r1],
            SUB(r1, _r2, _r3) => vec![r1],
            SUBI(r1, _r2, _i) => vec![r1],
            XOR(r1, _r2, _r3) => vec![r1],
            XORI(r1, _r2, _i) => vec![r1],
            JI(_im) => vec![],
            JNEI(_r1, _r2, _i) => vec![],
            JNZI(_r1, _i) => vec![],
            JMP(_r1) => vec![],
            JNE(_r1, _r2, _r3) => vec![],
            RET(_r1) => vec![],
            RETD(_r1, _r2) => vec![],
            CFEI(_imm) => vec![],
            CFSI(_imm) => vec![],
            LB(r1, _r2, _i) => vec![r1],
            LWDataId(r1, _i) => vec![r1],
            LW(r1, _r2, _i) => vec![r1],
            ALOC(_r1) => vec![],
            MCL(_r1, _r2) => vec![],
            MCLI(_r1, _imm) => vec![],
            MCP(_r1, _r2, _r3) => vec![],
            MEQ(r1, _r2, _r3, _r4) => vec![r1],
            MCPI(_r1, _r2, _imm) => vec![],
            SB(_r1, _r2, _i) => vec![],
            SW(_r1, _r2, _i) => vec![],
            BAL(r1, _r2, _r3) => vec![r1],
            BHSH(_r1, _r2) => vec![],
            BHEI(r1) => vec![r1],
            BURN(_r1) => vec![],
            CALL(_r1, _r2, _r3, _r4) => vec![],
            CCP(_r1, _r2, _r3, _r4) => vec![],
            CROO(_r1, _r2) => vec![],
            CSIZ(r1, _r2) => vec![r1],
            CB(_r1) => vec![],
            LDC(_r1, _r2, _r3) => vec![],
            LOG(_r1, _r2, _r3, _r4) => vec![],
            LOGD(_r1, _r2, _r3, _r4) => vec![],
            MINT(_r1) => vec![],
            RVRT(_r1) => vec![],
            SRW(r1, _r2) => vec![r1],
            SRWQ(_r1, _r2) => vec![],
            SWW(_r1, _r2) => vec![],
            SWWQ(_r1, _r2) => vec![],
            TIME(r1, _r2) => vec![r1],
            TR(_r1, _r2, _r3) => vec![],
            TRO(_r1, _r2, _r3, _r4) => vec![],
            ECR(_r1, _r2, _r3) => vec![],
            K256(_r1, _r2, _r3) => vec![],
            S256(_r1, _r2, _r3) => vec![],
            NOOP => vec![],
            BLOB(_imm) => vec![],
            FLAG(_r1) => vec![],
            GM(r1, _imm) => vec![r1],
            Undefined | DataSectionOffsetPlaceholder => vec![],
            DataSectionRegisterLoadPlaceholder => vec![&AllocatedRegister::Constant(
                ConstantRegister::DataSectionStart,
            )],
        })
        .into_iter()
        .collect()
    }
}

impl fmt::Display for AllocatedOpcode {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AllocatedOpcode::*;
        match self {
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
            GTF(a, b, c) => write!(fmtr, "gtf  {} {} {}", a, b, c),
            LT(a, b, c) => write!(fmtr, "lt   {} {} {}", a, b, c),
            MLOG(a, b, c) => write!(fmtr, "mlog {} {} {}", a, b, c),
            MROO(a, b, c) => write!(fmtr, "mroo {} {} {}", a, b, c),
            MOD(a, b, c) => write!(fmtr, "mod  {} {} {}", a, b, c),
            MODI(a, b, c) => write!(fmtr, "modi {} {} {}", a, b, c),
            MOVE(a, b) => write!(fmtr, "move {} {}", a, b),
            MOVI(a, b) => write!(fmtr, "movi {} {}", a, b),
            MUL(a, b, c) => write!(fmtr, "mul  {} {} {}", a, b, c),
            MULI(a, b, c) => write!(fmtr, "muli {} {} {}", a, b, c),
            NOT(a, b) => write!(fmtr, "not  {} {}", a, b),
            OR(a, b, c) => write!(fmtr, "or   {} {} {}", a, b, c),
            ORI(a, b, c) => write!(fmtr, "ori  {} {} {}", a, b, c),
            SLL(a, b, c) => write!(fmtr, "sll  {} {} {}", a, b, c),
            SLLI(a, b, c) => write!(fmtr, "slli {} {} {}", a, b, c),
            SMO(a, b, c, d) => write!(fmtr, "smo  {} {} {} {}", a, b, c, d),
            SRL(a, b, c) => write!(fmtr, "srl  {} {} {}", a, b, c),
            SRLI(a, b, c) => write!(fmtr, "srli {} {} {}", a, b, c),
            SUB(a, b, c) => write!(fmtr, "sub  {} {} {}", a, b, c),
            SUBI(a, b, c) => write!(fmtr, "subi {} {} {}", a, b, c),
            XOR(a, b, c) => write!(fmtr, "xor  {} {} {}", a, b, c),
            XORI(a, b, c) => write!(fmtr, "xori {} {} {}", a, b, c),
            JI(a) => write!(fmtr, "ji   {}", a),
            JNEI(a, b, c) => write!(fmtr, "jnei {} {} {}", a, b, c),
            JNZI(a, b) => write!(fmtr, "jnzi {} {}", a, b),
            JMP(a) => write!(fmtr, "jmp {}", a),
            JNE(a, b, c) => write!(fmtr, "jne  {a} {b} {c}"),
            RET(a) => write!(fmtr, "ret  {}", a),
            RETD(a, b) => write!(fmtr, "retd  {} {}", a, b),
            CFEI(a) => write!(fmtr, "cfei {}", a),
            CFSI(a) => write!(fmtr, "cfsi {}", a),
            LB(a, b, c) => write!(fmtr, "lb   {} {} {}", a, b, c),
            LWDataId(a, b) => write!(fmtr, "lw   {} {}", a, b),
            LW(a, b, c) => write!(fmtr, "lw   {} {} {}", a, b, c),
            ALOC(a) => write!(fmtr, "aloc {}", a),
            MCL(a, b) => write!(fmtr, "mcl  {} {}", a, b),
            MCLI(a, b) => write!(fmtr, "mcli {} {}", a, b),
            MCP(a, b, c) => write!(fmtr, "mcp  {} {} {}", a, b, c),
            MCPI(a, b, c) => write!(fmtr, "mcpi {} {} {}", a, b, c),
            MEQ(a, b, c, d) => write!(fmtr, "meq  {} {} {} {}", a, b, c, d),
            SB(a, b, c) => write!(fmtr, "sb   {} {} {}", a, b, c),
            SW(a, b, c) => write!(fmtr, "sw   {} {} {}", a, b, c),
            BAL(a, b, c) => write!(fmtr, "bal  {} {} {}", a, b, c),
            BHSH(a, b) => write!(fmtr, "bhsh {} {}", a, b),
            BHEI(a) => write!(fmtr, "bhei {}", a),
            BURN(a) => write!(fmtr, "burn {}", a),
            CALL(a, b, c, d) => write!(fmtr, "call {} {} {} {}", a, b, c, d),
            CCP(a, b, c, d) => write!(fmtr, "ccp  {} {} {} {}", a, b, c, d),
            CROO(a, b) => write!(fmtr, "croo {} {}", a, b),
            CSIZ(a, b) => write!(fmtr, "csiz {} {}", a, b),
            CB(a) => write!(fmtr, "cb   {}", a),
            LDC(a, b, c) => write!(fmtr, "ldc  {} {} {}", a, b, c),
            LOG(a, b, c, d) => write!(fmtr, "log  {} {} {} {}", a, b, c, d),
            LOGD(a, b, c, d) => write!(fmtr, "logd {} {} {} {}", a, b, c, d),
            MINT(a) => write!(fmtr, "mint {}", a),
            RVRT(a) => write!(fmtr, "rvrt {}", a),
            SRW(a, b) => write!(fmtr, "srw  {} {}", a, b),
            SRWQ(a, b) => write!(fmtr, "srwq {} {}", a, b),
            SWW(a, b) => write!(fmtr, "sww  {} {}", a, b),
            SWWQ(a, b) => write!(fmtr, "swwq {} {}", a, b),
            TIME(a, b) => write!(fmtr, "time {} {}", a, b),
            TR(a, b, c) => write!(fmtr, "tr   {} {} {}", a, b, c),
            TRO(a, b, c, d) => write!(fmtr, "tro  {} {} {} {}", a, b, c, d),
            ECR(a, b, c) => write!(fmtr, "ecr  {} {} {}", a, b, c),
            K256(a, b, c) => write!(fmtr, "k256 {} {} {}", a, b, c),
            S256(a, b, c) => write!(fmtr, "s256 {} {} {}", a, b, c),
            NOOP => write!(fmtr, "noop"),
            BLOB(a) => write!(fmtr, "blob {a}"),
            FLAG(a) => write!(fmtr, "flag {}", a),
            GM(a, b) => write!(fmtr, "gm {} {}", a, b),
            Undefined => write!(fmtr, "undefined op"),
            DataSectionOffsetPlaceholder => {
                write!(
                    fmtr,
                    "DATA_SECTION_OFFSET[0..32]\nDATA_SECTION_OFFSET[32..64]"
                )
            }
            DataSectionRegisterLoadPlaceholder => write!(fmtr, "lw   $ds $is 1"),
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
        data_section: &mut DataSection,
    ) -> Either<Vec<fuel_asm::Opcode>, DoubleWideData> {
        use AllocatedOpcode::*;
        #[rustfmt::skip]
         let fuel_op = Either::Left(vec![match &self.opcode {
            ADD (a, b, c)   => VmOp::ADD (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            ADDI(a, b, c)   => VmOp::ADDI(a.to_register_id(), b.to_register_id(), c.value),
            AND (a, b, c)   => VmOp::AND (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            ANDI(a, b, c)   => VmOp::ANDI(a.to_register_id(), b.to_register_id(), c.value),
            DIV (a, b, c)   => VmOp::DIV (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            DIVI(a, b, c)   => VmOp::DIVI(a.to_register_id(), b.to_register_id(), c.value),
            EQ  (a, b, c)   => VmOp::EQ  (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            EXP (a, b, c)   => VmOp::EXP (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            EXPI(a, b, c)   => VmOp::EXPI(a.to_register_id(), b.to_register_id(), c.value),
            GT  (a, b, c)   => VmOp::GT  (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            GTF (a, b, c)   => VmOp::GTF (a.to_register_id(), b.to_register_id(), c.value),
            LT  (a, b, c)   => VmOp::LT  (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MLOG(a, b, c)   => VmOp::MLOG(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MROO(a, b, c)   => VmOp::MROO(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MOD (a, b, c)   => VmOp::MOD (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MODI(a, b, c)   => VmOp::MODI(a.to_register_id(), b.to_register_id(), c.value),
            MOVE(a, b)      => VmOp::MOVE(a.to_register_id(), b.to_register_id()),
            MOVI(a, b)      => VmOp::MOVI(a.to_register_id(), b.value),
            MUL (a, b, c)   => VmOp::MUL (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MULI(a, b, c)   => VmOp::MULI(a.to_register_id(), b.to_register_id(), c.value),
            NOT (a, b)      => VmOp::NOT (a.to_register_id(), b.to_register_id()),
            OR  (a, b, c)   => VmOp::OR  (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            ORI (a, b, c)   => VmOp::ORI (a.to_register_id(), b.to_register_id(), c.value),
            SLL (a, b, c)   => VmOp::SLL (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SLLI(a, b, c)   => VmOp::SLLI(a.to_register_id(), b.to_register_id(), c.value),
            SMO (a, b, c, d)=> VmOp::SMO (a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            SRL (a, b, c)   => VmOp::SRL (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SRLI(a, b, c)   => VmOp::SRLI(a.to_register_id(), b.to_register_id(), c.value),
            SUB (a, b, c)   => VmOp::SUB (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SUBI(a, b, c)   => VmOp::SUBI(a.to_register_id(), b.to_register_id(), c.value),
            XOR (a, b, c)   => VmOp::XOR (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            XORI(a, b, c)   => VmOp::XORI(a.to_register_id(), b.to_register_id(), c.value),
            JI  (a)         => VmOp::JI  (a.value),
            JNEI(a, b, c)   => VmOp::JNEI(a.to_register_id(), b.to_register_id(), c.value),
            JNZI(a, b)      => VmOp::JNZI(a.to_register_id(), b.value),
            JMP (a)         => VmOp::JMP(a.to_register_id()),
            JNE (a, b, c)   => VmOp::JNE(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            RET (a)         => VmOp::RET (a.to_register_id()),
            RETD(a, b)      => VmOp::RETD (a.to_register_id(), b.to_register_id()),
            CFEI(a)         => VmOp::CFEI(a.value),
            CFSI(a)         => VmOp::CFSI(a.value),
            LB  (a, b, c)   => VmOp::LB  (a.to_register_id(), b.to_register_id(), c.value),
            LW  (a, b, c)   => VmOp::LW(a.to_register_id(), b.to_register_id(), c.value),
            LWDataId (a, b) => return Either::Left(realize_lw(a, b, data_section, offset_to_data_section)),
            ALOC(a)         => VmOp::ALOC(a.to_register_id()),
            MCL (a, b)      => VmOp::MCL (a.to_register_id(), b.to_register_id()),
            MCLI(a, b)      => VmOp::MCLI(a.to_register_id(), b.value),
            MCP (a, b, c)   => VmOp::MCP (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MCPI(a, b, c)   => VmOp::MCPI(a.to_register_id(), b.to_register_id(), c.value),
            MEQ (a, b, c, d)=> VmOp::MEQ (a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            SB  (a, b, c)   => VmOp::SB  (a.to_register_id(), b.to_register_id(), c.value),
            SW  (a, b, c)   => VmOp::SW  (a.to_register_id(), b.to_register_id(), c.value),
            BAL (a, b, c)   => VmOp::BAL (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            BHSH(a, b)      => VmOp::BHSH(a.to_register_id(), b.to_register_id()),
            BHEI(a)         => VmOp::BHEI(a.to_register_id()),
            BURN(a)         => VmOp::BURN(a.to_register_id()),
            CALL(a, b, c, d)=> VmOp::CALL(a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            CCP (a, b, c, d)=> VmOp::CCP (a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            CROO(a, b)      => VmOp::CROO(a.to_register_id(), b.to_register_id()),
            CSIZ(a, b)      => VmOp::CSIZ(a.to_register_id(), b.to_register_id()),
            CB  (a)         => VmOp::CB  (a.to_register_id()),
            LDC (a, b, c)   => VmOp::LDC (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            LOG (a, b, c, d)=> VmOp::LOG (a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            LOGD(a, b, c, d)=> VmOp::LOGD(a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            MINT(a)         => VmOp::MINT(a.to_register_id()),
            RVRT(a)         => VmOp::RVRT(a.to_register_id()),
            SRW (a, b)      => VmOp::SRW (a.to_register_id(), b.to_register_id()),
            SRWQ(a, b)      => VmOp::SRWQ(a.to_register_id(), b.to_register_id()),
            SWW (a, b)      => VmOp::SWW (a.to_register_id(), b.to_register_id()),
            SWWQ(a, b)      => VmOp::SWWQ(a.to_register_id(), b.to_register_id()),
            TIME(a, b)      => VmOp::TIME(a.to_register_id(), b.to_register_id()),
            TR  (a, b, c)   => VmOp::TR  (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            TRO (a, b, c, d)=> VmOp::TRO (a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            ECR (a, b, c)   => VmOp::ECR (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            K256(a, b, c)   => VmOp::K256(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            S256(a, b, c)   => VmOp::S256(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            NOOP            => VmOp::NOOP,
            BLOB(a)         => return Either::Left(std::iter::repeat(VmOp::NOOP).take(a.value as usize).collect()),
            FLAG(a)         => VmOp::FLAG(a.to_register_id()),
            GM  (a, b)      => VmOp::GM(a.to_register_id(), b.value),
            Undefined       => VmOp::Undefined,
            DataSectionOffsetPlaceholder => return Either::Right(offset_to_data_section.to_be_bytes()),
            DataSectionRegisterLoadPlaceholder => VmOp::LW(crate::asm_generation::compiler_constants::DATA_SECTION_REGISTER as fuel_asm::RegisterId, ConstantRegister::InstructionStart.to_register_id(), 1),
         }]);
        fuel_op
    }
}

/// Converts a virtual load word instruction which uses data labels into one which uses
/// actual bytewise offsets for use in bytecode.
/// Returns one op if the type is less than one word big, but two ops if it has to construct
/// a pointer and add it to $is.
fn realize_lw(
    dest: &AllocatedRegister,
    data_id: &DataId,
    data_section: &mut DataSection,
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
        let data_id_for_pointer =
            data_section.append_pointer(pointer_offset_from_instruction_start);
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
            crate::asm_generation::compiler_constants::DATA_SECTION_REGISTER as usize,
            offset.value,
        )]
    }
}
