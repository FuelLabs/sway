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

use super::*;
use crate::{
    asm_generation::fuel::{
        compiler_constants::{DATA_SECTION_REGISTER, FIRST_ALLOCATED_REGISTER},
        data_section::{DataId, DataSection},
    },
    fuel_prelude::fuel_asm::{self, op},
};
use fuel_vm::fuel_asm::{
    op::{ADD, MOVI},
    Imm18,
};
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
            AllocatedRegister::Allocated(name) => write!(fmtr, "$r{name}"),
            AllocatedRegister::Constant(name) => {
                write!(fmtr, "{name}")
            }
        }
    }
}

impl AllocatedRegister {
    /// First allocated register startd at `FIRST_ALLOCATED_REGISTER` (52) and goes
    /// down until 17.
    pub(crate) fn to_reg_id(&self) -> fuel_asm::RegId {
        match self {
            AllocatedRegister::Allocated(id) => {
                let id = FIRST_ALLOCATED_REGISTER.checked_sub(*id).unwrap();
                if id <= 16 {
                    panic!("invalid register id")
                }
                fuel_asm::RegId::new(id)
            },
            AllocatedRegister::Constant(constant) => constant.to_reg_id(),
        }
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, Self::Constant(ConstantRegister::Zero))
    }
}

/// This enum is unfortunately a redundancy of the [fuel_asm::Opcode] and [crate::VirtualOp] enums. This variant, however,
/// allows me to use the compiler's internal [AllocatedRegister] types and maintain type safety
/// between virtual ops and those which have gone through register allocation.
/// A bit of copy/paste seemed worth it for that safety,
/// so here it is.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub(crate) enum AllocatedInstruction {
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
    WQOP(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        VirtualImmediate06,
    ),
    WQML(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        VirtualImmediate06,
    ),
    WQDV(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        VirtualImmediate06,
    ),
    WQMD(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    WQCM(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        VirtualImmediate06,
    ),
    WQAM(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    WQMM(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),

    /* Control Flow Instructions */
    JMP(AllocatedRegister),
    JI(VirtualImmediate24),
    JNE(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    JNEI(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    JNZI(AllocatedRegister, VirtualImmediate18),
    JMPB(AllocatedRegister, VirtualImmediate18),
    JMPF(AllocatedRegister, VirtualImmediate18),
    JNZB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    JNZF(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    JNEB(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        VirtualImmediate06,
    ),
    JNEF(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        VirtualImmediate06,
    ),
    JAL(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    RET(AllocatedRegister),

    /* Memory Instructions */
    ALOC(AllocatedRegister),
    CFEI(VirtualImmediate24),
    CFSI(VirtualImmediate24),
    CFE(AllocatedRegister),
    CFS(AllocatedRegister),
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
    PSHH(VirtualImmediate24),
    PSHL(VirtualImmediate24),
    POPH(VirtualImmediate24),
    POPL(VirtualImmediate24),
    SB(AllocatedRegister, AllocatedRegister, VirtualImmediate12),
    SW(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /* Contract Instructions */
    BAL(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    BHEI(AllocatedRegister),
    BHSH(AllocatedRegister, AllocatedRegister),
    BURN(AllocatedRegister, AllocatedRegister),
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
    BSIZ(AllocatedRegister, AllocatedRegister),
    LDC(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        VirtualImmediate06,
    ),
    BLDD(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
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
    MINT(AllocatedRegister, AllocatedRegister),
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
    ECK1(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    ECR1(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    ED19(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    K256(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    S256(AllocatedRegister, AllocatedRegister, AllocatedRegister),
    ECOP(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    EPAR(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),

    /* Other Instructions */
    ECAL(
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
        AllocatedRegister,
    ),
    FLAG(AllocatedRegister),
    GM(AllocatedRegister, VirtualImmediate18),
    GTF(AllocatedRegister, AllocatedRegister, VirtualImmediate12),

    /* Non-VM Instructions */
    BLOB(VirtualImmediate24),
    ConfigurablesOffsetPlaceholder,
    DataSectionOffsetPlaceholder,
    LoadDataId(AllocatedRegister, DataId),
    AddrDataId(AllocatedRegister, DataId),
    Undefined,
}

impl AllocatedInstruction {
    /// Returns a list of all registers *written* by instruction `self`.
    pub(crate) fn def_registers(&self) -> BTreeSet<&AllocatedRegister> {
        use AllocatedInstruction::*;
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
            WQOP(_, _, _, _) => vec![],
            WQML(_, _, _, _) => vec![],
            WQDV(_, _, _, _) => vec![],
            WQMD(_, _, _, _) => vec![],
            WQCM(r1, _, _, _) => vec![r1],
            WQAM(_, _, _, _) => vec![],
            WQMM(_, _, _, _) => vec![],

            /* Control Flow Instructions */
            JMP(_r1) => vec![],
            JI(_im) => vec![],
            JNE(_r1, _r2, _r3) => vec![],
            JNEI(_r1, _r2, _i) => vec![],
            JNZI(_r1, _i) => vec![],
            JMPB(_r1, _i) => vec![],
            JMPF(_r1, _i) => vec![],
            JNZB(_r1, _r2, _i) => vec![],
            JNZF(_r1, _r2, _i) => vec![],
            JNEB(_r1, _r2, _r3, _i) => vec![],
            JNEF(_r1, _r2, _r3, _i) => vec![],
            JAL(r1, _r2, _i) => vec![r1],
            RET(_r1) => vec![],

            /* Memory Instructions */
            ALOC(_r1) => vec![],
            CFEI(_imm) => vec![],
            CFSI(_imm) => vec![],
            CFE(_r1) => vec![],
            CFS(_r1) => vec![],
            LB(r1, _r2, _i) => vec![r1],
            LW(r1, _r2, _i) => vec![r1],
            MCL(_r1, _r2) => vec![],
            MCLI(_r1, _imm) => vec![],
            MCP(_r1, _r2, _r3) => vec![],
            MCPI(_r1, _r2, _imm) => vec![],
            MEQ(r1, _r2, _r3, _r4) => vec![r1],
            PSHH(_mask) | PSHL(_mask) | POPH(_mask) | POPL(_mask) => {
                panic!("Cannot determine defined registers for register PUSH/POP instructions")
            }
            SB(_r1, _r2, _i) => vec![],
            SW(_r1, _r2, _i) => vec![],

            /* Contract Instructions */
            BAL(r1, _r2, _r3) => vec![r1],
            BHEI(r1) => vec![r1],
            BHSH(_r1, _r2) => vec![],
            BURN(_r1, _r2) => vec![],
            CALL(_r1, _r2, _r3, _r4) => vec![],
            CB(_r1) => vec![],
            CCP(_r1, _r2, _r3, _r4) => vec![],
            CROO(_r1, _r2) => vec![],
            CSIZ(r1, _r2) => vec![r1],
            BSIZ(r1, _r2) => vec![r1],
            LDC(_r1, _r2, _r3, _i0) => vec![],
            BLDD(_r1, _r2, _r3, _r4) => vec![],
            LOG(_r1, _r2, _r3, _r4) => vec![],
            LOGD(_r1, _r2, _r3, _r4) => vec![],
            MINT(_r1, _r2) => vec![],
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
            ECK1(_r1, _r2, _r3) => vec![],
            ECR1(_r1, _r2, _r3) => vec![],
            ED19(_r1, _r2, _r3, _r4) => vec![],
            K256(_r1, _r2, _r3) => vec![],
            S256(_r1, _r2, _r3) => vec![],
            ECOP(_r1, _r2, _r3, _r4) => vec![],
            EPAR(r1, _r2, _r3, _r4) => vec![r1],

            /* Other Instructions */
            ECAL(_r1, _r2, _r3, _r4) => vec![],
            FLAG(_r1) => vec![],
            GM(r1, _imm) => vec![r1],
            GTF(r1, _r2, _i) => vec![r1],

            /* Non-VM Instructions */
            BLOB(_imm) => vec![],
            ConfigurablesOffsetPlaceholder => vec![],
            DataSectionOffsetPlaceholder => vec![],
            LoadDataId(r1, _i) => vec![r1],
            AddrDataId(r1, _i) => vec![r1],
            Undefined => vec![],
        })
        .into_iter()
        .collect()
    }
}

impl fmt::Display for AllocatedInstruction {
    fn fmt(&self, fmtr: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AllocatedInstruction::*;
        match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(a, b, c) => write!(fmtr, "add  {a} {b} {c}"),
            ADDI(a, b, c) => write!(fmtr, "addi {a} {b} {c}"),
            AND(a, b, c) => write!(fmtr, "and  {a} {b} {c}"),
            ANDI(a, b, c) => write!(fmtr, "andi {a} {b} {c}"),
            DIV(a, b, c) => write!(fmtr, "div  {a} {b} {c}"),
            DIVI(a, b, c) => write!(fmtr, "divi {a} {b} {c}"),
            EQ(a, b, c) => write!(fmtr, "eq   {a} {b} {c}"),
            EXP(a, b, c) => write!(fmtr, "exp  {a} {b} {c}"),
            EXPI(a, b, c) => write!(fmtr, "expi {a} {b} {c}"),
            GT(a, b, c) => write!(fmtr, "gt   {a} {b} {c}"),
            LT(a, b, c) => write!(fmtr, "lt   {a} {b} {c}"),
            MLOG(a, b, c) => write!(fmtr, "mlog {a} {b} {c}"),
            MOD(a, b, c) => write!(fmtr, "mod  {a} {b} {c}"),
            MODI(a, b, c) => write!(fmtr, "modi {a} {b} {c}"),
            MOVE(a, b) => write!(fmtr, "move {a} {b}"),
            MOVI(a, b) => write!(fmtr, "movi {a} {b}"),
            MROO(a, b, c) => write!(fmtr, "mroo {a} {b} {c}"),
            MUL(a, b, c) => write!(fmtr, "mul  {a} {b} {c}"),
            MULI(a, b, c) => write!(fmtr, "muli {a} {b} {c}"),
            NOOP => write!(fmtr, "noop"),
            NOT(a, b) => write!(fmtr, "not  {a} {b}"),
            OR(a, b, c) => write!(fmtr, "or   {a} {b} {c}"),
            ORI(a, b, c) => write!(fmtr, "ori  {a} {b} {c}"),
            SLL(a, b, c) => write!(fmtr, "sll  {a} {b} {c}"),
            SLLI(a, b, c) => write!(fmtr, "slli {a} {b} {c}"),
            SRL(a, b, c) => write!(fmtr, "srl  {a} {b} {c}"),
            SRLI(a, b, c) => write!(fmtr, "srli {a} {b} {c}"),
            SUB(a, b, c) => write!(fmtr, "sub  {a} {b} {c}"),
            SUBI(a, b, c) => write!(fmtr, "subi {a} {b} {c}"),
            XOR(a, b, c) => write!(fmtr, "xor  {a} {b} {c}"),
            XORI(a, b, c) => write!(fmtr, "xori {a} {b} {c}"),
            WQOP(a, b, c, d) => write!(fmtr, "wqop {a} {b} {c} {d}"),
            WQML(a, b, c, d) => write!(fmtr, "wqml {a} {b} {c} {d}"),
            WQDV(a, b, c, d) => write!(fmtr, "wqdv {a} {b} {c} {d}"),
            WQMD(a, b, c, d) => write!(fmtr, "wqmd {a} {b} {c} {d}"),
            WQCM(a, b, c, d) => write!(fmtr, "wqcm {a} {b} {c} {d}"),
            WQAM(a, b, c, d) => write!(fmtr, "wqam {a} {b} {c} {d}"),
            WQMM(a, b, c, d) => write!(fmtr, "wqmm {a} {b} {c} {d}"),

            /* Control Flow Instructions */
            JMP(a) => write!(fmtr, "jmp  {a}"),
            JI(a) => write!(fmtr, "ji   {a}"),
            JNE(a, b, c) => write!(fmtr, "jne  {a} {b} {c}"),
            JNEI(a, b, c) => write!(fmtr, "jnei {a} {b} {c}"),
            JNZI(a, b) => write!(fmtr, "jnzi {a} {b}"),
            JMPB(a, b) => write!(fmtr, "jmpb {a} {b}"),
            JMPF(a, b) => write!(fmtr, "jmpf {a} {b}"),
            JNZB(a, b, c) => write!(fmtr, "jnzb {a} {b} {c}"),
            JNZF(a, b, c) => write!(fmtr, "jnzf {a} {b} {c}"),
            JNEB(a, b, c, d) => write!(fmtr, "jneb {a} {b} {c} {d}"),
            JNEF(a, b, c, d) => write!(fmtr, "jnef {a} {b} {c} {d}"),
            JAL(a, b, c) => write!(fmtr, "jal  {a} {b} {c}"),
            RET(a) => write!(fmtr, "ret  {a}"),

            /* Memory Instructions */
            ALOC(a) => write!(fmtr, "aloc {a}"),
            CFEI(a) => write!(fmtr, "cfei {a}"),
            CFSI(a) => write!(fmtr, "cfsi {a}"),
            CFE(a) => write!(fmtr, "cfe {a}"),
            CFS(a) => write!(fmtr, "cfs {a}"),
            LB(a, b, c) => write!(fmtr, "lb   {a} {b} {c}"),
            LW(a, b, c) => write!(fmtr, "lw   {a} {b} {c}"),
            MCL(a, b) => write!(fmtr, "mcl  {a} {b}"),
            MCLI(a, b) => write!(fmtr, "mcli {a} {b}"),
            MCP(a, b, c) => write!(fmtr, "mcp  {a} {b} {c}"),
            MCPI(a, b, c) => write!(fmtr, "mcpi {a} {b} {c}"),
            MEQ(a, b, c, d) => write!(fmtr, "meq  {a} {b} {c} {d}"),
            PSHH(mask) => write!(fmtr, "pshh {mask}"),
            PSHL(mask) => write!(fmtr, "pshl {mask}"),
            POPH(mask) => write!(fmtr, "poph {mask}"),
            POPL(mask) => write!(fmtr, "popl {mask}"),
            SB(a, b, c) => write!(fmtr, "sb   {a} {b} {c}"),
            SW(a, b, c) => write!(fmtr, "sw   {a} {b} {c}"),

            /* Contract Instructions */
            BAL(a, b, c) => write!(fmtr, "bal  {a} {b} {c}"),
            BHEI(a) => write!(fmtr, "bhei {a}"),
            BHSH(a, b) => write!(fmtr, "bhsh {a} {b}"),
            BURN(a, b) => write!(fmtr, "burn {a} {b}"),
            CALL(a, b, c, d) => write!(fmtr, "call {a} {b} {c} {d}"),
            CB(a) => write!(fmtr, "cb   {a}"),
            CCP(a, b, c, d) => write!(fmtr, "ccp  {a} {b} {c} {d}"),
            CROO(a, b) => write!(fmtr, "croo {a} {b}"),
            CSIZ(a, b) => write!(fmtr, "csiz {a} {b}"),
            BSIZ(a, b) => write!(fmtr, "bsiz {a} {b}"),
            LDC(a, b, c, d) => write!(fmtr, "ldc  {a} {b} {c} {d}"),
            BLDD(a, b, c, d) => write!(fmtr, "bldd {a} {b} {c} {d}"),
            LOG(a, b, c, d) => write!(fmtr, "log  {a} {b} {c} {d}"),
            LOGD(a, b, c, d) => write!(fmtr, "logd {a} {b} {c} {d}"),
            MINT(a, b) => write!(fmtr, "mint {a} {b}"),
            RETD(a, b) => write!(fmtr, "retd  {a} {b}"),
            RVRT(a) => write!(fmtr, "rvrt {a}"),
            SMO(a, b, c, d) => write!(fmtr, "smo  {a} {b} {c} {d}"),
            SCWQ(a, b, c) => write!(fmtr, "scwq {a} {b} {c}"),
            SRW(a, b, c) => write!(fmtr, "srw  {a} {b} {c}"),
            SRWQ(a, b, c, d) => write!(fmtr, "srwq {a} {b} {c} {d}"),
            SWW(a, b, c) => write!(fmtr, "sww  {a} {b} {c}"),
            SWWQ(a, b, c, d) => write!(fmtr, "swwq {a} {b} {c} {d}"),
            TIME(a, b) => write!(fmtr, "time {a} {b}"),
            TR(a, b, c) => write!(fmtr, "tr   {a} {b} {c}"),
            TRO(a, b, c, d) => write!(fmtr, "tro  {a} {b} {c} {d}"),

            /* Cryptographic Instructions */
            ECK1(a, b, c) => write!(fmtr, "eck1  {a} {b} {c}"),
            ECR1(a, b, c) => write!(fmtr, "ecr1  {a} {b} {c}"),
            ED19(a, b, c, d) => write!(fmtr, "ed19  {a} {b} {c} {d}"),
            K256(a, b, c) => write!(fmtr, "k256 {a} {b} {c}"),
            S256(a, b, c) => write!(fmtr, "s256 {a} {b} {c}"),
            ECOP(a, b, c, d) => write!(fmtr, "ecop {a} {b} {c} {d}"),
            EPAR(a, b, c, d) => write!(fmtr, "epar {a} {b} {c} {d}"),

            /* Other Instructions */
            ECAL(a, b, c, d) => write!(fmtr, "ecal {a} {b} {c} {d}"),
            FLAG(a) => write!(fmtr, "flag {a}"),
            GM(a, b) => write!(fmtr, "gm   {a} {b}"),
            GTF(a, b, c) => write!(fmtr, "gtf  {a} {b} {c}"),

            /* Non-VM Instructions */
            BLOB(a) => write!(fmtr, "blob {a}"),
            ConfigurablesOffsetPlaceholder => write!(
                fmtr,
                "CONFIGURABLES_OFFSET[0..32]\nCONFIGURABLES_OFFSET[32..64]"
            ),
            DataSectionOffsetPlaceholder => {
                write!(
                    fmtr,
                    "DATA_SECTION_OFFSET[0..32]\nDATA_SECTION_OFFSET[32..64]"
                )
            }
            LoadDataId(a, b) => write!(fmtr, "load {a} {b}"),
            AddrDataId(a, b) => write!(fmtr, "addr {a} {b}"),
            Undefined => write!(fmtr, "undefined op"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AllocatedOp {
    pub(crate) opcode: AllocatedInstruction,
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

        write!(fmtr, "{op_and_comment}")
    }
}

pub(crate) enum FuelAsmData {
    ConfigurablesOffset([u8; 8]),
    DatasectionOffset([u8; 8]),
    Instructions(Vec<fuel_asm::Instruction>),
}

impl AllocatedOp {
    pub(crate) fn to_fuel_asm(
        &self,
        offset_to_data_section: u64,
        offset_from_instr_start: u64,
        data_section: &DataSection,
    ) -> FuelAsmData {
        use AllocatedInstruction::*;
        FuelAsmData::Instructions(vec![match &self.opcode {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(a, b, c) => op::ADD::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            ADDI(a, b, c) => op::ADDI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            AND(a, b, c) => op::AND::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            ANDI(a, b, c) => op::ANDI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            DIV(a, b, c) => op::DIV::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            DIVI(a, b, c) => op::DIVI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            EQ(a, b, c) => op::EQ::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            EXP(a, b, c) => op::EXP::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            EXPI(a, b, c) => op::EXPI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            GT(a, b, c) => op::GT::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            LT(a, b, c) => op::LT::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            MLOG(a, b, c) => op::MLOG::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            MOD(a, b, c) => op::MOD::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            MODI(a, b, c) => op::MODI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            MOVE(a, b) => op::MOVE::new(a.to_reg_id(), b.to_reg_id()).into(),
            MOVI(a, b) => op::MOVI::new(a.to_reg_id(), b.as_imm18().unwrap()).into(),
            MROO(a, b, c) => op::MROO::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            MUL(a, b, c) => op::MUL::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            MULI(a, b, c) => op::MULI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            NOOP => op::NOOP::new().into(),
            NOT(a, b) => op::NOT::new(a.to_reg_id(), b.to_reg_id()).into(),
            OR(a, b, c) => op::OR::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            ORI(a, b, c) => op::ORI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            SLL(a, b, c) => op::SLL::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            SLLI(a, b, c) => op::SLLI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            SRL(a, b, c) => op::SRL::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            SRLI(a, b, c) => op::SRLI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            SUB(a, b, c) => op::SUB::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            SUBI(a, b, c) => op::SUBI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            XOR(a, b, c) => op::XOR::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            XORI(a, b, c) => op::XORI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            WQOP(a, b, c, d) => op::WQOP::new(
                a.to_reg_id(),
                b.to_reg_id(),
                c.to_reg_id(),
                d.value().into(),
            )
            .into(),
            WQML(a, b, c, d) => op::WQML::new(
                a.to_reg_id(),
                b.to_reg_id(),
                c.to_reg_id(),
                d.value().into(),
            )
            .into(),
            WQDV(a, b, c, d) => op::WQDV::new(
                a.to_reg_id(),
                b.to_reg_id(),
                c.to_reg_id(),
                d.value().into(),
            )
            .into(),
            WQMD(a, b, c, d) => {
                op::WQMD::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            WQCM(a, b, c, d) => op::WQCM::new(
                a.to_reg_id(),
                b.to_reg_id(),
                c.to_reg_id(),
                d.value().into(),
            )
            .into(),
            WQAM(a, b, c, d) => {
                op::WQAM::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            WQMM(a, b, c, d) => {
                op::WQMM::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }

            /* Control Flow Instructions */
            JMP(a) => op::JMP::new(a.to_reg_id()).into(),
            JI(a) => op::JI::new(a.value().into()).into(),
            JNE(a, b, c) => op::JNE::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            JNEI(a, b, c) => op::JNEI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            JNZI(a, b) => op::JNZI::new(a.to_reg_id(), b.as_imm18().unwrap()).into(),
            JMPB(a, b) => op::JMPB::new(a.to_reg_id(), b.as_imm18().unwrap()).into(),
            JMPF(a, b) => op::JMPF::new(a.to_reg_id(), b.as_imm18().unwrap()).into(),
            JNZB(a, b, c) => op::JNZB::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            JNZF(a, b, c) => op::JNZF::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            JNEB(a, b, c, d) => op::JNEB::new(
                a.to_reg_id(),
                b.to_reg_id(),
                c.to_reg_id(),
                d.value().into(),
            )
            .into(),
            JNEF(a, b, c, d) => op::JNEF::new(
                a.to_reg_id(),
                b.to_reg_id(),
                c.to_reg_id(),
                d.value().into(),
            )
            .into(),
            JAL(a, b, c) => op::JAL::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            RET(a) => op::RET::new(a.to_reg_id()).into(),

            /* Memory Instructions */
            ALOC(a) => op::ALOC::new(a.to_reg_id()).into(),
            CFEI(a) if a.value() == 0 => return FuelAsmData::Instructions(vec![]),
            CFEI(a) => op::CFEI::new(a.value().into()).into(),
            CFSI(a) if a.value() == 0 => return FuelAsmData::Instructions(vec![]),
            CFSI(a) => op::CFSI::new(a.value().into()).into(),
            CFE(a) => op::CFE::new(a.to_reg_id()).into(),
            CFS(a) => op::CFS::new(a.to_reg_id()).into(),
            LB(a, b, c) => op::LB::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            LW(a, b, c) => op::LW::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            MCL(a, b) => op::MCL::new(a.to_reg_id(), b.to_reg_id()).into(),
            MCLI(a, b) => op::MCLI::new(a.to_reg_id(), b.as_imm18().unwrap()).into(),
            MCP(a, b, c) => op::MCP::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            MCPI(a, b, c) => op::MCPI::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            MEQ(a, b, c, d) => {
                op::MEQ::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            PSHH(mask) => op::PSHH::new(mask.value().into()).into(),
            PSHL(mask) => op::PSHL::new(mask.value().into()).into(),
            POPH(mask) => op::POPH::new(mask.value().into()).into(),
            POPL(mask) => op::POPL::new(mask.value().into()).into(),
            SB(a, b, c) => op::SB::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),
            SW(a, b, c) => op::SW::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),

            /* Contract Instructions */
            BAL(a, b, c) => op::BAL::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            BHEI(a) => op::BHEI::new(a.to_reg_id()).into(),
            BHSH(a, b) => op::BHSH::new(a.to_reg_id(), b.to_reg_id()).into(),
            BURN(a, b) => op::BURN::new(a.to_reg_id(), b.to_reg_id()).into(),
            CALL(a, b, c, d) => {
                op::CALL::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            CB(a) => op::CB::new(a.to_reg_id()).into(),
            CCP(a, b, c, d) => {
                op::CCP::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            CROO(a, b) => op::CROO::new(a.to_reg_id(), b.to_reg_id()).into(),
            CSIZ(a, b) => op::CSIZ::new(a.to_reg_id(), b.to_reg_id()).into(),
            BSIZ(a, b) => op::BSIZ::new(a.to_reg_id(), b.to_reg_id()).into(),
            LDC(a, b, c, d) => op::LDC::new(
                a.to_reg_id(),
                b.to_reg_id(),
                c.to_reg_id(),
                d.value().into(),
            )
            .into(),
            BLDD(a, b, c, d) => {
                op::BLDD::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            LOG(a, b, c, d) => {
                op::LOG::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            LOGD(a, b, c, d) => {
                op::LOGD::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            MINT(a, b) => op::MINT::new(a.to_reg_id(), b.to_reg_id()).into(),
            RETD(a, b) => op::RETD::new(a.to_reg_id(), b.to_reg_id()).into(),
            RVRT(a) => op::RVRT::new(a.to_reg_id()).into(),
            SMO(a, b, c, d) => {
                op::SMO::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            SCWQ(a, b, c) => op::SCWQ::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            SRW(a, b, c) => op::SRW::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            SRWQ(a, b, c, d) => {
                op::SRWQ::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            SWW(a, b, c) => op::SWW::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            SWWQ(a, b, c, d) => {
                op::SWWQ::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            TIME(a, b) => op::TIME::new(a.to_reg_id(), b.to_reg_id()).into(),
            TR(a, b, c) => op::TR::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            TRO(a, b, c, d) => {
                op::TRO::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }

            /* Cryptographic Instructions */
            ECK1(a, b, c) => op::ECK1::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            ECR1(a, b, c) => op::ECR1::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            ED19(a, b, c, d) => {
                op::ED19::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            K256(a, b, c) => op::K256::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            S256(a, b, c) => op::S256::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id()).into(),
            ECOP(a, b, c, d) => {
                op::ECOP::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            EPAR(a, b, c, d) => {
                op::EPAR::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }

            /* Other Instructions */
            ECAL(a, b, c, d) => {
                op::ECAL::new(a.to_reg_id(), b.to_reg_id(), c.to_reg_id(), d.to_reg_id()).into()
            }
            FLAG(a) => op::FLAG::new(a.to_reg_id()).into(),
            GM(a, b) => op::GM::new(a.to_reg_id(), b.as_imm18().unwrap()).into(),
            GTF(a, b, c) => op::GTF::new(a.to_reg_id(), b.to_reg_id(), c.value().into()).into(),

            /* Non-VM Instructions */
            BLOB(a) => {
                return FuelAsmData::Instructions(
                    std::iter::repeat_n(op::NOOP::new().into(), a.value() as usize).collect(),
                )
            }
            ConfigurablesOffsetPlaceholder => {
                return FuelAsmData::ConfigurablesOffset([0, 0, 0, 0, 0, 0, 0, 0])
            }
            DataSectionOffsetPlaceholder => {
                return FuelAsmData::DatasectionOffset(offset_to_data_section.to_be_bytes())
            }
            LoadDataId(a, b) => {
                return FuelAsmData::Instructions(realize_load(
                    a,
                    b,
                    data_section,
                    offset_to_data_section,
                    offset_from_instr_start,
                ))
            }
            AddrDataId(a, b) => return FuelAsmData::Instructions(addr_of(a, b, data_section)),
            Undefined => unreachable!("Sway cannot generate undefined ASM opcodes"),
        }])
    }
}

/// Address of a data section item
fn addr_of(
    dest: &AllocatedRegister,
    data_id: &DataId,
    data_section: &DataSection,
) -> Vec<fuel_asm::Instruction> {
    let offset_bytes = data_section.data_id_to_offset(data_id) as u64;
    vec![
        fuel_asm::Instruction::MOVI(MOVI::new(
            dest.to_reg_id(),
            Imm18::new(offset_bytes.try_into().unwrap()),
        )),
        fuel_asm::Instruction::ADD(ADD::new(
            dest.to_reg_id(),
            dest.to_reg_id(),
            fuel_asm::RegId::new(DATA_SECTION_REGISTER),
        )),
    ]
}

/// Converts a virtual load word instruction which uses data labels into one which uses
/// actual bytewise offsets for use in bytecode.
/// Returns one op if the type is less than one word big, but two ops if it has to construct
/// a pointer and add it to $is.
fn realize_load(
    dest: &AllocatedRegister,
    data_id: &DataId,
    data_section: &DataSection,
    offset_to_data_section: u64,
    offset_from_instr_start: u64,
) -> Vec<fuel_asm::Instruction> {
    // if this data is larger than a word, instead of loading the data directly
    // into the register, we want to load a pointer to the data into the register
    // this appends onto the data section and mutates it by adding the pointer as a literal
    let has_copy_type = data_section.has_copy_type(data_id).expect(
        "Internal miscalculation in data section -- data id did not match up to any actual data",
    );

    let is_byte = data_section.is_byte(data_id).expect(
        "Internal miscalculation in data section -- data id did not match up to any actual data",
    );

    // all data is word-aligned right now, and `offset_to_id` returns the offset in bytes
    let offset_bytes = data_section.data_id_to_offset(data_id) as u64;
    assert!(
        offset_bytes.is_multiple_of(8),
        "Internal miscalculation in data section -- data offset is not aligned to a word",
    );
    let offset_words = offset_bytes / 8;

    let imm = VirtualImmediate12::try_new(
        if is_byte { offset_bytes } else { offset_words },
        Span::new(" ".into(), 0, 0, None).unwrap(),
    );
    let offset = match imm {
        Ok(value) => value,
        Err(_) => panic!(
            "Unable to offset into the data section more than 2^12 bits. \
                                Unsupported data section length: {offset_words} words."
        ),
    };

    if !has_copy_type {
        // load the pointer itself into the register. `offset_to_data_section` is in bytes.
        // The -4 is because $pc is added in the *next* instruction.
        let pointer_offset_from_current_instr =
            offset_to_data_section - offset_from_instr_start + offset_bytes - 4;

        // insert the pointer as bytes as a new data section entry at the end of the data
        let data_id_for_pointer = data_section
            .data_id_of_pointer(pointer_offset_from_current_instr)
            .expect("Pointer offset must be in data_section");

        // now load the pointer we just created into the `dest`ination
        let mut buf = Vec::with_capacity(2);
        buf.append(&mut realize_load(
            dest,
            &data_id_for_pointer,
            data_section,
            offset_to_data_section,
            offset_from_instr_start,
        ));
        // add $pc to the pointer since it is relative to the current instruction.
        buf.push(
            fuel_asm::op::ADD::new(
                dest.to_reg_id(),
                dest.to_reg_id(),
                ConstantRegister::ProgramCounter.to_reg_id(),
            )
            .into(),
        );
        buf
    } else if is_byte {
        vec![fuel_asm::op::LB::new(
            dest.to_reg_id(),
            fuel_asm::RegId::new(DATA_SECTION_REGISTER),
            offset.value().into(),
        )
        .into()]
    } else {
        vec![fuel_asm::op::LW::new(
            dest.to_reg_id(),
            fuel_asm::RegId::new(DATA_SECTION_REGISTER),
            offset.value().into(),
        )
        .into()]
    }
}
