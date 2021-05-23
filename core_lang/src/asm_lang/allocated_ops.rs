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

use super::virtual_ops::*;
use pest::Span;
use std::fmt;

const COMMENT_START_COLUMN: usize = 30;

/// Represents registers that have gone through register allocation. The value in the [Allocated]
/// variant is guaranteed to be between 0 and [compiler_constants::NUM_FREE_REGISTERS].
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum AllocatedRegister {
    Allocated(u8),
    Constant(super::virtual_ops::ConstantRegister),
}

impl fmt::Display for AllocatedRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocatedRegister::Allocated(name) => write!(f, "$r{}", name),
            AllocatedRegister::Constant(name) => {
                write!(f, "{}", name)
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
#[derive(Clone)]
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
    MLOG(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MROO(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MOD(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MODI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    MOVE(AllocatedRegister, AllocatedRegister),
    MUL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MULI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
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
    CIMV(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    CTMV(AllocatedRegister, AllocatedRegister),
    JI(VirtualImmediate24),
    JNEI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    RET(AllocatedRegister),
    CFEI(VirtualImmediate24),
    CFSI(VirtualImmediate24),
    LB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    LW(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    ALOC(AllocatedRegister),
    MCL(AllocatedRegister, AllocatedRegister),
    MCLI(AllocatedRegister, VirtualImmediate18),
    MCP(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    MEQ(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    SB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SW(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
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
    MINT(AllocatedRegister),
    RVRT(AllocatedRegister),
    SLDC(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    SRW(AllocatedRegister, AllocatedRegister),
    SRWQ(AllocatedRegister, AllocatedRegister),
    SWW(AllocatedRegister, AllocatedRegister),
    SWWQ(AllocatedRegister, AllocatedRegister),
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
    FLAG(AllocatedRegister),
    Undefined,
}

#[derive(Clone)]
pub(crate) struct AllocatedOp<'sc> {
    pub(crate) opcode: AllocatedOpcode,
    /// A descriptive comment for ASM readability
    pub(crate) comment: String,
    pub(crate) owning_span: Option<Span<'sc>>,
}

impl<'sc> fmt::Display for AllocatedOp<'sc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AllocatedOpcode::*;
        #[rustfmt::skip]
        let string = match &self.opcode {
            ADD(a, b, c)    => format!("add  {} {} {}", a, b, c),
            ADDI(a, b, c)   => format!("addi {} {} {}", a, b, c),
            AND(a, b, c)    => format!("and  {} {} {}", a, b, c),
            ANDI(a, b, c)   => format!("andi {} {} {}", a, b, c),
            DIV(a, b, c)    => format!("div  {} {} {}", a, b, c),
            DIVI(a, b, c)   => format!("divi {} {} {}", a, b, c),
            EQ(a, b, c)     => format!("eq   {} {} {}", a, b, c),
            EXP(a, b, c)    => format!("exp  {} {} {}", a, b, c),
            EXPI(a, b, c)   => format!("expi {} {} {}", a, b, c),
            GT(a, b, c)     => format!("gt   {} {} {}", a, b, c),
            MLOG(a, b, c)   => format!("mlog {} {} {}", a, b, c),
            MROO(a, b, c)   => format!("mroo {} {} {}", a, b, c),
            MOD(a, b, c)    => format!("mod  {} {} {}", a, b, c),
            MODI(a, b, c)   => format!("modi {} {} {}", a, b, c),
            MOVE(a, b)      => format!("move {} {}", a, b),
            MUL(a, b, c)    => format!("mul  {} {} {}", a, b, c),
            MULI(a, b, c)   => format!("muli {} {} {}", a, b, c),
            NOT(a, b)       => format!("not  {} {}", a, b),
            OR(a, b, c)     => format!("or   {} {} {}", a, b, c),
            ORI(a, b, c)    => format!("ori  {} {} {}", a, b, c),
            SLL(a, b, c)    => format!("sll  {} {} {}", a, b, c),
            SLLI(a, b, c)   => format!("slli {} {} {}", a, b, c),
            SRL(a, b, c)    => format!("srl  {} {} {}", a, b, c),
            SRLI(a, b, c)   => format!("srli {} {} {}", a, b, c),
            SUB(a, b, c)    => format!("sub  {} {} {}", a, b, c),
            SUBI(a, b, c)   => format!("subi {} {} {}", a, b, c),
            XOR(a, b, c)    => format!("xor  {} {} {}", a, b, c),
            XORI(a, b, c)   => format!("xori {} {} {}", a, b, c),
            CIMV(a, b, c)   => format!("cimv {} {} {}", a, b, c),
            CTMV(a, b)      => format!("ctmv {} {}", a, b),
            JI(a)           => format!("ji   {}", a),
            JNEI(a, b, c)   => format!("jnei {} {} {}", a, b, c),
            RET(a)          => format!("ret  {}", a),
            CFEI(a)         => format!("cfei {}", a),
            CFSI(a)         => format!("cfsi {}", a),
            LB(a, b, c)     => format!("lb   {} {} {}", a, b, c),
            LW(a, b, c)     => format!("lw   {} {} {}", a, b, c),
            ALOC(a)         => format!("aloc {}", a),
            MCL(a, b)       => format!("mcl  {} {}", a, b),
            MCLI(a, b)      => format!("mcli {} {}", a, b),
            MCP(a, b, c)    => format!("mcp  {} {} {}", a, b, c),
            MEQ(a, b, c, d) => format!("meq  {} {} {} {}", a, b, c, d),
            SB(a, b, c)     => format!("sb   {} {} {}", a, b, c),
            SW(a, b, c)     => format!("sw   {} {} {}", a, b, c),
            BHSH(a, b)      => format!("bhsh {} {}", a, b),
            BHEI(a)         => format!("bhei {}", a),
            BURN(a)         => format!("burn {}", a),
            CALL(a, b, c, d)=> format!("call {} {} {} {}", a, b, c, d),
            CCP(a, b, c, d) => format!("ccp  {} {} {} {}", a, b, c, d),
            CROO(a, b)      => format!("croo {} {}", a, b),
            CSIZ(a, b)      => format!("csiz {} {}", a, b),
            CB(a)           => format!("cb   {}", a),
            LDC(a, b, c)    => format!("ldc  {} {} {}", a, b, c),
            LOG(a, b, c, d) => format!("log  {} {} {} {}", a, b, c, d),
            MINT(a)         => format!("mint {}", a),
            RVRT(a)         => format!("rvrt {}", a),
            SLDC(a, b, c)   => format!("sldc {} {} {}", a, b, c),
            SRW(a, b)       => format!("srw  {} {}", a, b),
            SRWQ(a, b)      => format!("srwq {} {}", a, b),
            SWW(a, b)       => format!("sww  {} {}", a, b),
            SWWQ(a, b)      => format!("swwq {} {}", a, b),
            TR(a, b, c)     => format!("tr   {} {} {}", a, b, c),
            TRO(a, b, c, d) => format!("tro  {} {} {} {}", a, b, c, d),
            ECR(a, b, c)    => format!("ecr  {} {} {}", a, b, c),
            K256(a, b, c)   => format!("k256 {} {} {}", a, b, c),
            S256(a, b, c)   => format!("s256 {} {} {}", a, b, c),
            NOOP            => "noop".to_string(),
            FLAG(a)         => format!("flag {}", a),
            Undefined       => format!("undefined op"),
        };
        // we want the comment to always be COMMENT_START_COLUMN characters offset to the right
        // to not interfere with the ASM but to be aligned
        let mut op_and_comment = string;
        if self.comment.len() > 0 {
            while op_and_comment.len() < COMMENT_START_COLUMN {
                op_and_comment.push_str(" ");
            }
            op_and_comment.push_str(&format!("; {}", self.comment))
        }

        write!(f, "{}", op_and_comment)
    }
}

impl<'sc> AllocatedOp<'sc> {
    fn to_fuel_asm(&self) -> fuel_asm::Opcode {
        use fuel_asm::Opcode as VmOp;
        use AllocatedOpcode::*;
        #[rustfmt::skip]
         let fuel_op = match &self.opcode {
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
            MLOG(a, b, c)   => VmOp::MLOG(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MROO(a, b, c)   => VmOp::MROO(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MOD (a, b, c)   => VmOp::MOD (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MODI(a, b, c)   => VmOp::MODI(a.to_register_id(), b.to_register_id(), c.value),
            MOVE(a, b)      => VmOp::MOVE(a.to_register_id(), b.to_register_id()),
            MUL (a, b, c)   => VmOp::MUL (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MULI(a, b, c)   => VmOp::MULI(a.to_register_id(), b.to_register_id(), c.value),
            NOT (a, b)      => VmOp::NOT (a.to_register_id(), b.to_register_id()),
            OR  (a, b, c)   => VmOp::OR  (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            ORI (a, b, c)   => VmOp::ORI (a.to_register_id(), b.to_register_id(), c.value),
            SLL (a, b, c)   => VmOp::SLL (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SLLI(a, b, c)   => VmOp::SLLI(a.to_register_id(), b.to_register_id(), c.value),
            SRL (a, b, c)   => VmOp::SRL (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SRLI(a, b, c)   => VmOp::SRLI(a.to_register_id(), b.to_register_id(), c.value),
            SUB (a, b, c)   => VmOp::SUB (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SUBI(a, b, c)   => VmOp::SUBI(a.to_register_id(), b.to_register_id(), c.value),
            XOR (a, b, c)   => VmOp::XOR (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            XORI(a, b, c)   => VmOp::XORI(a.to_register_id(), b.to_register_id(), c.value),
            CIMV(a, b, c)   => VmOp::CIMV(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            CTMV(a, b)      => VmOp::CTMV(a.to_register_id(), b.to_register_id()),
            JI  (a)         => VmOp::JI  (a.value),
            JNEI(a, b, c)   => VmOp::JNEI(a.to_register_id(), b.to_register_id(), c.value),
            RET (a)         => VmOp::RET (a.to_register_id()),
            CFEI(a)         => VmOp::CFEI(a.value),
            CFSI(a)         => VmOp::CFSI(a.value),
            LB  (a, b, c)   => VmOp::LB  (a.to_register_id(), b.to_register_id(), c.value),
            LW  (a, b, c)   => VmOp::LW  (a.to_register_id(), b.to_register_id(), c.value),
            ALOC(a)         => VmOp::ALOC(a.to_register_id()),
            MCL (a, b)      => VmOp::MCL (a.to_register_id(), b.to_register_id()),
            MCLI(a, b)      => VmOp::MCLI(a.to_register_id(), b.value),
            MCP (a, b, c)   => VmOp::MCP (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            MEQ (a, b, c, d)=> VmOp::MEQ (a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            SB  (a, b, c)   => VmOp::SB  (a.to_register_id(), b.to_register_id(), c.value),
            SW  (a, b, c)   => VmOp::SW  (a.to_register_id(), b.to_register_id(), c.value),
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
            MINT(a)         => VmOp::MINT(a.to_register_id()),
            RVRT(a)         => VmOp::RVRT(a.to_register_id()),
            SLDC(a, b, c)   => VmOp::SLDC(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            SRW (a, b)      => VmOp::SRW (a.to_register_id(), b.to_register_id()),
            SRWQ(a, b)      => VmOp::SRWQ(a.to_register_id(), b.to_register_id()),
            SWW (a, b)      => VmOp::SWW (a.to_register_id(), b.to_register_id()),
            SWWQ(a, b)      => VmOp::SWWQ(a.to_register_id(), b.to_register_id()),
            TR  (a, b, c)   => VmOp::TR  (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            TRO (a, b, c, d)=> VmOp::TRO (a.to_register_id(), b.to_register_id(), c.to_register_id(), d.to_register_id()),
            ECR (a, b, c)   => VmOp::ECR (a.to_register_id(), b.to_register_id(), c.to_register_id()),
            K256(a, b, c)   => VmOp::K256(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            S256(a, b, c)   => VmOp::S256(a.to_register_id(), b.to_register_id(), c.to_register_id()),
            NOOP            => VmOp::NOOP,
            FLAG(a)         => VmOp::FLAG(a.to_register_id()),
            Undefined       => VmOp::Undefined,
         };
        fuel_op
    }
}
