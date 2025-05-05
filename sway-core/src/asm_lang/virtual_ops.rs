//! This module contains abstracted versions of bytecode primitives that the compiler uses to
//! ensure correctness and safety.
//!
//! The immediate types are used to safely construct numbers that are within their bounds, and the
//! ops are clones of the actual opcodes, but with the safe primitives as arguments.

use indexmap::IndexMap;

use super::{
    allocated_ops::{AllocatedOpcode, AllocatedRegister},
    virtual_immediate::*,
    virtual_register::*,
    Op,
};
use crate::asm_generation::fuel::{data_section::DataId, register_allocator::RegisterPool};

use std::collections::{BTreeSet, HashMap};

use std::fmt;

/// This enum is unfortunately a redundancy of the [fuel_asm::Opcode] enum. This variant, however,
/// allows me to use the compiler's internal [VirtualRegister] types and maintain type safety
/// between virtual ops and the real opcodes. A bit of copy/paste seemed worth it for that safety,
/// so here it is.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub(crate) enum VirtualOp {
    /* Arithmetic/Logic (ALU) Instructions */
    ADD(VirtualRegister, VirtualRegister, VirtualRegister),
    ADDI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    AND(VirtualRegister, VirtualRegister, VirtualRegister),
    ANDI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    DIV(VirtualRegister, VirtualRegister, VirtualRegister),
    DIVI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    EQ(VirtualRegister, VirtualRegister, VirtualRegister),
    EXP(VirtualRegister, VirtualRegister, VirtualRegister),
    EXPI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    GT(VirtualRegister, VirtualRegister, VirtualRegister),
    LT(VirtualRegister, VirtualRegister, VirtualRegister),
    MLOG(VirtualRegister, VirtualRegister, VirtualRegister),
    MOD(VirtualRegister, VirtualRegister, VirtualRegister),
    MODI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    MOVE(VirtualRegister, VirtualRegister),
    MOVI(VirtualRegister, VirtualImmediate18),
    MROO(VirtualRegister, VirtualRegister, VirtualRegister),
    MUL(VirtualRegister, VirtualRegister, VirtualRegister),
    MULI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    NOOP,
    NOT(VirtualRegister, VirtualRegister),
    OR(VirtualRegister, VirtualRegister, VirtualRegister),
    ORI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    SLL(VirtualRegister, VirtualRegister, VirtualRegister),
    SLLI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    SRL(VirtualRegister, VirtualRegister, VirtualRegister),
    SRLI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    SUB(VirtualRegister, VirtualRegister, VirtualRegister),
    SUBI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    XOR(VirtualRegister, VirtualRegister, VirtualRegister),
    XORI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    WQOP(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualImmediate06,
    ),
    WQML(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualImmediate06,
    ),
    WQDV(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualImmediate06,
    ),
    WQMD(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    WQCM(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualImmediate06,
    ),
    WQAM(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    WQMM(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),

    /* Control Flow Instructions */
    JMP(VirtualRegister),
    JI(VirtualImmediate24),
    JNE(VirtualRegister, VirtualRegister, VirtualRegister),
    JNEI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    JNZI(VirtualRegister, VirtualImmediate18),
    RET(VirtualRegister),

    /* Memory Instructions */
    ALOC(VirtualRegister, VirtualRegister),
    CFEI(VirtualRegister, VirtualImmediate24),
    CFSI(VirtualRegister, VirtualImmediate24),
    CFE(VirtualRegister, VirtualRegister),
    CFS(VirtualRegister, VirtualRegister),
    LB(VirtualRegister, VirtualRegister, VirtualImmediate12),
    LW(VirtualRegister, VirtualRegister, VirtualImmediate12),
    MCL(VirtualRegister, VirtualRegister),
    MCLI(VirtualRegister, VirtualImmediate18),
    MCP(VirtualRegister, VirtualRegister, VirtualRegister),
    MCPI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    MEQ(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    SB(VirtualRegister, VirtualRegister, VirtualImmediate12),
    SW(VirtualRegister, VirtualRegister, VirtualImmediate12),

    /* Contract Instructions */
    BAL(VirtualRegister, VirtualRegister, VirtualRegister),
    BHEI(VirtualRegister),
    BHSH(VirtualRegister, VirtualRegister),
    BURN(VirtualRegister, VirtualRegister),
    CALL(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    CB(VirtualRegister),
    CCP(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    CROO(VirtualRegister, VirtualRegister),
    CSIZ(VirtualRegister, VirtualRegister),
    BSIZ(VirtualRegister, VirtualRegister),
    LDC(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualImmediate06,
    ),
    BLDD(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    LOG(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    LOGD(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    MINT(VirtualRegister, VirtualRegister),
    RETD(VirtualRegister, VirtualRegister),
    RVRT(VirtualRegister),
    SMO(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    SCWQ(VirtualRegister, VirtualRegister, VirtualRegister),
    SRW(VirtualRegister, VirtualRegister, VirtualRegister),
    SRWQ(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    SWW(VirtualRegister, VirtualRegister, VirtualRegister),
    SWWQ(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    TIME(VirtualRegister, VirtualRegister),
    TR(VirtualRegister, VirtualRegister, VirtualRegister),
    TRO(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),

    /* Cryptographic Instructions */
    ECK1(VirtualRegister, VirtualRegister, VirtualRegister),
    ECR1(VirtualRegister, VirtualRegister, VirtualRegister),
    ED19(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    K256(VirtualRegister, VirtualRegister, VirtualRegister),
    S256(VirtualRegister, VirtualRegister, VirtualRegister),
    ECOP(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    EPAR(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),

    /* Other Instructions */
    ECAL(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    FLAG(VirtualRegister),
    GM(VirtualRegister, VirtualImmediate18),
    GTF(VirtualRegister, VirtualRegister, VirtualImmediate12),

    /* Non-VM Instructions */
    BLOB(VirtualImmediate24),
    ConfigurablesOffsetPlaceholder,
    DataSectionOffsetPlaceholder,
    // LoadDataId takes a virtual register and a DataId, which points to a labeled piece
    // of data in the data section. Note that the ASM op corresponding to a LW is
    // subtly complex: $rB is in bytes and points to some mem address. The immediate
    // third argument is a _word_ offset from that byte address.
    LoadDataId(VirtualRegister, DataId),
    AddrDataId(VirtualRegister, DataId),
    Undefined,
}

impl VirtualOp {
    pub(crate) fn registers(&self) -> BTreeSet<&VirtualRegister> {
        use VirtualOp::*;
        (match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(r1, r2, r3) => vec![r1, r2, r3],
            ADDI(r1, r2, _i) => vec![r1, r2],
            AND(r1, r2, r3) => vec![r1, r2, r3],
            ANDI(r1, r2, _i) => vec![r1, r2],
            DIV(r1, r2, r3) => vec![r1, r2, r3],
            DIVI(r1, r2, _i) => vec![r1, r2],
            EQ(r1, r2, r3) => vec![r1, r2, r3],
            EXP(r1, r2, r3) => vec![r1, r2, r3],
            EXPI(r1, r2, _i) => vec![r1, r2],
            GT(r1, r2, r3) => vec![r1, r2, r3],
            LT(r1, r2, r3) => vec![r1, r2, r3],
            MLOG(r1, r2, r3) => vec![r1, r2, r3],
            MOD(r1, r2, r3) => vec![r1, r2, r3],
            MODI(r1, r2, _i) => vec![r1, r2],
            MOVE(r1, r2) => vec![r1, r2],
            MOVI(r1, _i) => vec![r1],
            MROO(r1, r2, r3) => vec![r1, r2, r3],
            MUL(r1, r2, r3) => vec![r1, r2, r3],
            MULI(r1, r2, _i) => vec![r1, r2],
            NOOP => vec![],
            NOT(r1, r2) => vec![r1, r2],
            OR(r1, r2, r3) => vec![r1, r2, r3],
            ORI(r1, r2, _i) => vec![r1, r2],
            SLL(r1, r2, r3) => vec![r1, r2, r3],
            SLLI(r1, r2, _i) => vec![r1, r2],
            SRL(r1, r2, r3) => vec![r1, r2, r3],
            SRLI(r1, r2, _i) => vec![r1, r2],
            SUB(r1, r2, r3) => vec![r1, r2, r3],
            SUBI(r1, r2, _i) => vec![r1, r2],
            XOR(r1, r2, r3) => vec![r1, r2, r3],
            XORI(r1, r2, _i) => vec![r1, r2],
            WQOP(r1, r2, r3, _) => vec![r1, r2, r3],
            WQML(r1, r2, r3, _) => vec![r1, r2, r3],
            WQDV(r1, r2, r3, _) => vec![r1, r2, r3],
            WQMD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            WQCM(r1, r2, r3, _) => vec![r1, r2, r3],
            WQAM(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            WQMM(r1, r2, r3, r4) => vec![r1, r2, r3, r4],

            /* Control Flow Instructions */
            JMP(r1) => vec![r1],
            JI(_im) => vec![],
            JNE(r1, r2, r3) => vec![r1, r2, r3],
            JNEI(r1, r2, _i) => vec![r1, r2],
            JNZI(r1, _i) => vec![r1],
            RET(r1) => vec![r1],

            /* Memory Instructions */
            ALOC(hp, r1) => vec![hp, r1],
            CFEI(sp, _imm) => vec![sp],
            CFSI(sp, _imm) => vec![sp],
            CFE(sp, r1) => vec![sp, r1],
            CFS(sp, r1) => vec![sp, r1],
            LB(r1, r2, _i) => vec![r1, r2],
            LW(r1, r2, _i) => vec![r1, r2],
            MCL(r1, r2) => vec![r1, r2],
            MCLI(r1, _imm) => vec![r1],
            MCP(r1, r2, r3) => vec![r1, r2, r3],
            MCPI(r1, r2, _imm) => vec![r1, r2],
            MEQ(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            SB(r1, r2, _i) => vec![r1, r2],
            SW(r1, r2, _i) => vec![r1, r2],

            /* Contract Instructions */
            BAL(r1, r2, r3) => vec![r1, r2, r3],
            BHEI(r1) => vec![r1],
            BHSH(r1, r2) => vec![r1, r2],
            BURN(r1, r2) => vec![r1, r2],
            CALL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CB(r1) => vec![r1],
            CCP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CROO(r1, r2) => vec![r1, r2],
            CSIZ(r1, r2) => vec![r1, r2],
            BSIZ(r1, r2) => vec![r1, r2],
            LDC(r1, r2, r3, _i0) => vec![r1, r2, r3],
            BLDD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            LOG(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            LOGD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            MINT(r1, r2) => vec![r1, r2],
            RETD(r1, r2) => vec![r1, r2],
            RVRT(r1) => vec![r1],
            SMO(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            SCWQ(r1, r2, r3) => vec![r1, r2, r3],
            SRW(r1, r2, r3) => vec![r1, r2, r3],
            SRWQ(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            SWW(r1, r2, r3) => vec![r1, r2, r3],
            SWWQ(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            TIME(r1, r2) => vec![r1, r2],
            TR(r1, r2, r3) => vec![r1, r2, r3],
            TRO(r1, r2, r3, r4) => vec![r1, r2, r3, r4],

            /* Cryptographic Instructions */
            ECK1(r1, r2, r3) => vec![r1, r2, r3],
            ECR1(r1, r2, r3) => vec![r1, r2, r3],
            ED19(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            K256(r1, r2, r3) => vec![r1, r2, r3],
            S256(r1, r2, r3) => vec![r1, r2, r3],
            ECOP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            EPAR(r1, r2, r3, r4) => vec![r1, r2, r3, r4],

            /* Other Instructions */
            ECAL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            FLAG(r1) => vec![r1],
            GM(r1, _imm) => vec![r1],
            GTF(r1, r2, _i) => vec![r1, r2],

            /* Non-VM Instructions */
            BLOB(_imm) => vec![],
            DataSectionOffsetPlaceholder => vec![],
            ConfigurablesOffsetPlaceholder => vec![],
            LoadDataId(r1, _i) => vec![r1],
            AddrDataId(r1, _) => vec![r1],

            Undefined => vec![],
        })
        .into_iter()
        .collect()
    }

    /// Does this op do anything other than just compute something?
    /// (and hence if the result is dead, the OP can safely be deleted).
    pub(crate) fn has_side_effect(&self) -> bool {
        use VirtualOp::*;
        match self {
            // Arithmetic and logical
            ADD(_, _, _)
            | ADDI(_, _, _)
            | AND(_, _, _)
            | ANDI(_, _, _)
            | DIV(_, _, _)
            | DIVI(_, _, _)
            | EQ(_, _, _)
            | EXP(_, _, _)
            | EXPI(_, _, _)
            | GT(_, _, _)
            | LT(_, _, _)
            | MLOG(_, _, _)
            | MOD(_, _, _)
            | MODI(_, _, _)
            | MOVE(_, _)
            | MOVI(_, _)
            | MROO(_, _, _)
            | MUL(_, _, _)
            | MULI(_, _, _)
            | NOOP
            | NOT(_, _)
            | OR(_, _, _)
            | ORI(_, _, _)
            | SLL(_, _, _)
            | SLLI(_, _, _)
            | SRL(_, _, _)
            | SRLI(_, _, _)
            | SUB(_, _, _)
            | SUBI(_, _, _)
            | XOR(_, _, _)
            | XORI(_, _, _)
            // Memory load
            | LB(_, _, _)
            | LW(_, _, _)
            // Blockchain read
            |  BAL(_, _, _)
            |  BHEI(_)
            | CSIZ(_, _)
            | BSIZ(_, _)
            | SRW(_, _, _)
            | TIME(_, _)
            |  GM(_, _)
            | GTF(_, _, _)
            | EPAR(_, _, _, _)
            // Virtual OPs
            | LoadDataId(_, _)
            | AddrDataId(_, _)
             => self.def_registers().iter().any(|vreg| matches!(vreg, VirtualRegister::Constant(_))),
            // Memory write and jump
            WQOP(_, _, _, _)
            | WQML(_, _, _, _)
            | WQDV(_, _, _, _)
            | WQMD(_, _, _, _)
            | WQCM(_, _, _, _)
            | WQAM(_, _, _, _)
            | WQMM(_, _, _, _)
            | JMP(_)
            | JI(_)
            | JNE(_, _, _)
            | JNEI(_, _, _)
            | JNZI(_, _)
            | RET(_)
            | ALOC(..)
            | CFEI(..)
            | CFSI(..)
            | CFE(..)
            | CFS(..)
            | MCL(_, _)
            | MCLI(_, _)
            | MCP(_, _, _)
            | MCPI(_, _, _)
            | MEQ(_, _, _, _)
            | SB(_, _, _)
            | SW(_, _, _)
            // Other blockchain etc ...
            | BHSH(_, _)
            | BURN(_, _)
            | CALL(_, _, _, _)
            | CB(_)
            | CCP(_, _, _, _)
            | CROO(_, _)
            | LDC(_, _, _, _)
            | BLDD(_, _, _, _)
            | LOG(_, _, _, _)
            | LOGD(_, _, _, _)
            | MINT(_, _)
            | RETD(_, _)
            | RVRT(_)
            | SMO(_, _, _, _)
            | SCWQ(_, _, _)
            | SRWQ(_, _, _, _)
            | SWW(_, _, _)
            | SWWQ(_, _, _, _)
            | TR(_, _, _)
            | TRO(_, _, _, _)
            | ECK1(_, _, _)
            | ECR1(_, _, _)
            | ED19(_, _, _, _)
            | K256(_, _, _)
            | S256(_, _, _)
            | ECOP(_, _, _, _)
            // Other instructions
            | ECAL(_, _, _, _)
            | FLAG(_)
            // Virtual OPs
            | BLOB(_)
            | DataSectionOffsetPlaceholder
            | ConfigurablesOffsetPlaceholder
            | Undefined => true
        }
    }

    // What are the special registers that an OP may set.
    pub(crate) fn def_const_registers(&self) -> BTreeSet<&VirtualRegister> {
        use ConstantRegister::*;
        use VirtualOp::*;
        (match self {
            // Arithmetic and logical
            ADD(_, _, _)
            | ADDI(_, _, _)
            | AND(_, _, _)
            | ANDI(_, _, _)
            | DIV(_, _, _)
            | DIVI(_, _, _)
            | EQ(_, _, _)
            | EXP(_, _, _)
            | EXPI(_, _, _)
            | GT(_, _, _)
            | LT(_, _, _)
            | MLOG(_, _, _)
            | MOD(_, _, _)
            | MODI(_, _, _)
            | MOVE(_, _)
            | MOVI(_, _)
            | MROO(_, _, _)
            | MUL(_, _, _)
            | MULI(_, _, _)
            | NOOP
            | NOT(_, _)
            | OR(_, _, _)
            | ORI(_, _, _)
            | SLL(_, _, _)
            | SLLI(_, _, _)
            | SRL(_, _, _)
            | SRLI(_, _, _)
            | SUB(_, _, _)
            | SUBI(_, _, _)
            | XOR(_, _, _)
            | XORI(_, _, _)
            | WQOP(_, _, _, _)
            | WQML(_, _, _, _)
            | WQDV(_, _, _, _)
            | WQMD(_, _, _, _)
            | WQCM(_, _, _, _)
            | WQAM(_, _, _, _)
            | WQMM(_, _, _, _)
            // Cryptographic
            | ECK1(_, _, _)
            | ECR1(_, _, _)
            | ED19(_, _, _, _)
             => vec![&VirtualRegister::Constant(Overflow), &VirtualRegister::Constant(Error)],
            FLAG(_) => vec![&VirtualRegister::Constant(Flags)],
            | ALOC(hp, _) => vec![hp],
            | CFEI(sp, _)
            | CFSI(sp, _)
            | CFE(sp, _)
            | CFS(sp, _) => vec![sp],
            JMP(_)
            | JI(_)
            | JNE(_, _, _)
            | JNEI(_, _, _)
            | JNZI(_, _)
            | RET(_)
            | LB(_, _, _)
            | LW(_, _, _)
            | MCL(_, _)
            | MCLI(_, _)
            | MCP(_, _, _)
            | MCPI(_, _, _)
            | MEQ(_, _, _, _)
            | SB(_, _, _)
            | SW(_, _, _)
            | BAL(_, _, _)
            | BHEI(_)
            | BHSH(_, _)
            | BURN(_, _)
            | CALL(_, _, _, _)
            | CB(_)
            | CCP(_, _, _, _)
            | CROO(_, _)
            | CSIZ(_, _)
            | BSIZ(_, _)
            | LDC(_, _, _, _)
            | BLDD(_, _, _, _)
            | LOG(_, _, _, _)
            | LOGD(_, _, _, _)
            | MINT(_, _)
            | RETD(_, _)
            | RVRT(_)
            | SMO(_, _, _, _)
            | SCWQ(_, _, _)
            | SRW(_, _, _)
            | SRWQ(_, _, _, _)
            | SWW(_, _, _)
            | SWWQ(_, _, _, _)
            | TIME(_, _)
            | TR(_, _, _)
            | TRO(_, _, _, _)
            | K256(_, _, _)
            | S256(_, _, _)
            | ECOP(_, _, _, _)
            | EPAR(_, _, _, _)
            | ECAL(_, _, _, _)
            | GM(_, _)
            | GTF(_, _, _)
            | BLOB(_)
            | DataSectionOffsetPlaceholder
            | ConfigurablesOffsetPlaceholder
            | LoadDataId(_, _)
            | AddrDataId(_, _)
            | Undefined => vec![],
        })
        .into_iter()
        .collect()
    }

    /// Returns a list of all registers *read* by instruction `self`.
    pub(crate) fn use_registers(&self) -> BTreeSet<&VirtualRegister> {
        use VirtualOp::*;
        (match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(_r1, r2, r3) => vec![r2, r3],
            ADDI(_r1, r2, _i) => vec![r2],
            AND(_r1, r2, r3) => vec![r2, r3],
            ANDI(_r1, r2, _i) => vec![r2],
            DIV(_r1, r2, r3) => vec![r2, r3],
            DIVI(_r1, r2, _i) => vec![r2],
            EQ(_r1, r2, r3) => vec![r2, r3],
            EXP(_r1, r2, r3) => vec![r2, r3],
            EXPI(_r1, r2, _i) => vec![r2],
            GT(_r1, r2, r3) => vec![r2, r3],
            LT(_r1, r2, r3) => vec![r2, r3],
            MLOG(_r1, r2, r3) => vec![r2, r3],
            MOD(_r1, r2, r3) => vec![r2, r3],
            MODI(_r1, r2, _i) => vec![r2],
            MOVE(_r1, r2) => vec![r2],
            MOVI(_r1, _i) => vec![],
            MROO(_r1, r2, r3) => vec![r2, r3],
            MUL(_r1, r2, r3) => vec![r2, r3],
            MULI(_r1, r2, _i) => vec![r2],
            NOOP => vec![],
            NOT(_r1, r2) => vec![r2],
            OR(_r1, r2, r3) => vec![r2, r3],
            ORI(_r1, r2, _i) => vec![r2],
            SLL(_r1, r2, r3) => vec![r2, r3],
            SLLI(_r1, r2, _i) => vec![r2],
            SRL(_r1, r2, r3) => vec![r2, r3],
            SRLI(_r1, r2, _i) => vec![r2],
            SUB(_r1, r2, r3) => vec![r2, r3],
            SUBI(_r1, r2, _i) => vec![r2],
            XOR(_r1, r2, r3) => vec![r2, r3],
            XORI(_r1, r2, _i) => vec![r2],
            // Note that most of the `WQ..` instructions *read* from the `r1` result register,
            // because the register itself does not contain the result, but provides the
            // memory address at which the result will be stored.
            WQOP(r1, r2, r3, _) => vec![r1, r2, r3],
            WQML(r1, r2, r3, _) => vec![r1, r2, r3],
            WQDV(r1, r2, r3, _) => vec![r1, r2, r3],
            WQMD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            WQCM(_, r2, r3, _) => vec![r2, r3],
            WQAM(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            WQMM(r1, r2, r3, r4) => vec![r1, r2, r3, r4],

            /* Control Flow Instructions */
            JMP(r1) => vec![r1],
            JI(_im) => vec![],
            JNE(r1, r2, r3) => vec![r1, r2, r3],
            JNEI(r1, r2, _i) => vec![r1, r2],
            JNZI(r1, _i) => vec![r1],
            RET(r1) => vec![r1],

            /* Memory Instructions */
            ALOC(hp, r1) => vec![hp, r1],
            CFEI(sp, _imm) => vec![sp],
            CFSI(sp, _imm) => vec![sp],
            CFE(sp, r1) => vec![sp, r1],
            CFS(sp, r1) => vec![sp, r1],
            LB(_r1, r2, _i) => vec![r2],
            LW(_r1, r2, _i) => vec![r2],
            MCL(r1, r2) => vec![r1, r2],
            MCLI(r1, _imm) => vec![r1],
            MCP(r1, r2, r3) => vec![r1, r2, r3],
            MCPI(r1, r2, _imm) => vec![r1, r2],
            MEQ(_r1, r2, r3, r4) => vec![r2, r3, r4],
            SB(r1, r2, _i) => vec![r1, r2],
            SW(r1, r2, _i) => vec![r1, r2],

            /* Contract Instructions */
            BAL(_r1, r2, r3) => vec![r2, r3],
            BHEI(_r1) => vec![],
            BHSH(r1, r2) => vec![r1, r2],
            BURN(r1, r2) => vec![r1, r2],
            CALL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CB(r1) => vec![r1],
            CCP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CROO(r1, r2) => vec![r1, r2],
            CSIZ(_r1, r2) => vec![r2],
            BSIZ(_r1, r2) => vec![r2],
            LDC(r1, r2, r3, _i0) => vec![r1, r2, r3],
            BLDD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            LOG(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            LOGD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            MINT(r1, r2) => vec![r1, r2],
            RETD(r1, r2) => vec![r1, r2],
            RVRT(r1) => vec![r1],
            SMO(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            SCWQ(r1, _r2, r3) => vec![r1, r3],
            SRW(_r1, _r2, r3) => vec![r3],
            SRWQ(r1, _r2, r3, r4) => vec![r1, r3, r4],
            SWW(r1, _r2, r3) => vec![r1, r3],
            SWWQ(r1, _r2, r3, r4) => vec![r1, r3, r4],
            TIME(_r1, r2) => vec![r2],
            TR(r1, r2, r3) => vec![r1, r2, r3],
            TRO(r1, r2, r3, r4) => vec![r1, r2, r3, r4],

            /* Cryptographic Instructions */
            ECK1(r1, r2, r3) => vec![r1, r2, r3],
            ECR1(r1, r2, r3) => vec![r1, r2, r3],
            ED19(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            K256(r1, r2, r3) => vec![r1, r2, r3],
            S256(r1, r2, r3) => vec![r1, r2, r3],
            ECOP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            EPAR(_r1, r2, r3, r4) => vec![r2, r3, r4],

            /* Other Instructions */
            ECAL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            FLAG(r1) => vec![r1],
            GM(_r1, _imm) => vec![],
            GTF(_r1, r2, _i) => vec![r2],

            /* Non-VM Instructions */
            BLOB(_imm) => vec![],
            DataSectionOffsetPlaceholder => vec![],
            ConfigurablesOffsetPlaceholder => vec![],
            LoadDataId(_r1, _i) => vec![],
            AddrDataId(_r1, _i) => vec![],

            Undefined => vec![],
        })
        .into_iter()
        .collect()
    }

    /// Returns a list of all registers *read* by instruction `self`.
    pub(crate) fn use_registers_mut(&mut self) -> BTreeSet<&mut VirtualRegister> {
        use VirtualOp::*;
        (match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(_r1, r2, r3) => vec![r2, r3],
            ADDI(_r1, r2, _i) => vec![r2],
            AND(_r1, r2, r3) => vec![r2, r3],
            ANDI(_r1, r2, _i) => vec![r2],
            DIV(_r1, r2, r3) => vec![r2, r3],
            DIVI(_r1, r2, _i) => vec![r2],
            EQ(_r1, r2, r3) => vec![r2, r3],
            EXP(_r1, r2, r3) => vec![r2, r3],
            EXPI(_r1, r2, _i) => vec![r2],
            GT(_r1, r2, r3) => vec![r2, r3],
            LT(_r1, r2, r3) => vec![r2, r3],
            MLOG(_r1, r2, r3) => vec![r2, r3],
            MOD(_r1, r2, r3) => vec![r2, r3],
            MODI(_r1, r2, _i) => vec![r2],
            MOVE(_r1, r2) => vec![r2],
            MOVI(_r1, _i) => vec![],
            MROO(_r1, r2, r3) => vec![r2, r3],
            MUL(_r1, r2, r3) => vec![r2, r3],
            MULI(_r1, r2, _i) => vec![r2],
            NOOP => vec![],
            NOT(_r1, r2) => vec![r2],
            OR(_r1, r2, r3) => vec![r2, r3],
            ORI(_r1, r2, _i) => vec![r2],
            SLL(_r1, r2, r3) => vec![r2, r3],
            SLLI(_r1, r2, _i) => vec![r2],
            SRL(_r1, r2, r3) => vec![r2, r3],
            SRLI(_r1, r2, _i) => vec![r2],
            SUB(_r1, r2, r3) => vec![r2, r3],
            SUBI(_r1, r2, _i) => vec![r2],
            XOR(_r1, r2, r3) => vec![r2, r3],
            XORI(_r1, r2, _i) => vec![r2],
            // Note that most of the `WQ..` instructions *read* from the `r1` result register,
            // because the register itself does not contain the result, but provides the
            // memory address at which the result will be stored.
            WQOP(r1, r2, r3, _) => vec![r1, r2, r3],
            WQML(r1, r2, r3, _) => vec![r1, r2, r3],
            WQDV(r1, r2, r3, _) => vec![r1, r2, r3],
            WQMD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            WQCM(_, r2, r3, _) => vec![r2, r3],
            WQAM(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            WQMM(r1, r2, r3, r4) => vec![r1, r2, r3, r4],

            /* Control Flow Instructions */
            JMP(r1) => vec![r1],
            JI(_im) => vec![],
            JNE(r1, r2, r3) => vec![r1, r2, r3],
            JNEI(r1, r2, _i) => vec![r1, r2],
            JNZI(r1, _i) => vec![r1],
            RET(r1) => vec![r1],

            /* Memory Instructions */
            ALOC(hp, r1) => vec![hp, r1],
            CFEI(sp, _imm) => vec![sp],
            CFSI(sp, _imm) => vec![sp],
            CFE(sp, r1) => vec![sp, r1],
            CFS(sp, r1) => vec![sp, r1],
            LB(_r1, r2, _i) => vec![r2],
            LW(_r1, r2, _i) => vec![r2],
            MCL(r1, r2) => vec![r1, r2],
            MCLI(r1, _imm) => vec![r1],
            MCP(r1, r2, r3) => vec![r1, r2, r3],
            MCPI(r1, r2, _imm) => vec![r1, r2],
            MEQ(_r1, r2, r3, r4) => vec![r2, r3, r4],
            SB(r1, r2, _i) => vec![r1, r2],
            SW(r1, r2, _i) => vec![r1, r2],

            /* Contract Instructions */
            BAL(_r1, r2, r3) => vec![r2, r3],
            BHEI(_r1) => vec![],
            BHSH(r1, r2) => vec![r1, r2],
            BURN(r1, r2) => vec![r1, r2],
            CALL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CB(r1) => vec![r1],
            CCP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CROO(r1, r2) => vec![r1, r2],
            CSIZ(_r1, r2) => vec![r2],
            BSIZ(_r1, r2) => vec![r2],
            LDC(r1, r2, r3, _i0) => vec![r1, r2, r3],
            BLDD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            LOG(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            LOGD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            MINT(r1, r2) => vec![r1, r2],
            RETD(r1, r2) => vec![r1, r2],
            RVRT(r1) => vec![r1],
            SMO(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            SCWQ(r1, _r2, r3) => vec![r1, r3],
            SRW(_r1, _r2, r3) => vec![r3],
            SRWQ(r1, _r2, r3, r4) => vec![r1, r3, r4],
            SWW(r1, _r2, r3) => vec![r1, r3],
            SWWQ(r1, _r2, r3, r4) => vec![r1, r3, r4],
            TIME(_r1, r2) => vec![r2],
            TR(r1, r2, r3) => vec![r1, r2, r3],
            TRO(r1, r2, r3, r4) => vec![r1, r2, r3, r4],

            /* Cryptographic Instructions */
            ECK1(r1, r2, r3) => vec![r1, r2, r3],
            ECR1(r1, r2, r3) => vec![r1, r2, r3],
            ED19(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            K256(r1, r2, r3) => vec![r1, r2, r3],
            S256(r1, r2, r3) => vec![r1, r2, r3],
            ECOP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            EPAR(_r1, r2, r3, r4) => vec![r2, r3, r4],

            /* Other Instructions */
            ECAL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            FLAG(r1) => vec![r1],
            GM(_r1, _imm) => vec![],
            GTF(_r1, r2, _i) => vec![r2],

            /* Non-VM Instructions */
            BLOB(_imm) => vec![],
            DataSectionOffsetPlaceholder => vec![],
            ConfigurablesOffsetPlaceholder => vec![],
            LoadDataId(_r1, _i) => vec![],
            AddrDataId(_r1, _i) => vec![],

            Undefined => vec![],
        })
        .into_iter()
        .collect()
    }

    /// Returns a list of all registers *written* by instruction `self`. All of our opcodes define
    /// exactly 0 or 1 register, so the size of this returned vector should always be at most 1.
    pub(crate) fn def_registers(&self) -> BTreeSet<&VirtualRegister> {
        use VirtualOp::*;
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
            RET(_r1) => vec![],

            /* Memory Instructions */
            ALOC(hp, _r1) => vec![hp],
            CFEI(sp, _imm) => vec![sp],
            CFSI(sp, _imm) => vec![sp],
            CFE(sp, _r1) => vec![sp],
            CFS(sp, _r1) => vec![sp],
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
            BURN(_r1, _r2) => vec![],
            CALL(_r1, _r2, _r3, _r4) => vec![],
            CB(_r1) => vec![],
            CCP(_r1, _r2, _r3, _r4) => vec![],
            CROO(_r1, _r2) => vec![],
            CSIZ(r1, _r2) => vec![r1],
            BSIZ(r1, _r2) => vec![r1],
            LDC(_r1, _r2, _r3, _i0) => vec![],
            BLDD(_r1, _r2, _r3, _i0) => vec![],
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
            LoadDataId(r1, _i) => vec![r1],
            AddrDataId(r1, _i) => vec![r1],
            DataSectionOffsetPlaceholder => vec![],
            ConfigurablesOffsetPlaceholder => vec![],
            Undefined => vec![],
        })
        .into_iter()
        .collect()
    }

    /// Returns a list of indices that represent the successors of `self` in the list of
    /// instructions `ops`. For most instructions, the successor is simply the next instruction in
    /// `ops`. The exceptions are jump instructions that can have arbitrary successors and RVRT
    /// which does not have any successors.
    pub(crate) fn successors(&self, index: usize, ops: &[Op]) -> Vec<usize> {
        use VirtualOp::*;
        let next_op = if index >= ops.len() - 1 {
            vec![]
        } else {
            vec![index + 1]
        };
        match self {
            RVRT(_) => vec![],
            JI(_) | JNEI(..) | JNZI(..) => {
                unreachable!("At this stage we shouldn't have jumps in the code.")
            }

            _ => next_op,
        }
    }

    pub(crate) fn update_register(
        &self,
        reg_to_reg_map: &IndexMap<&VirtualRegister, &VirtualRegister>,
    ) -> Self {
        use VirtualOp::*;
        match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(r1, r2, r3) => Self::ADD(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            ADDI(r1, r2, i) => Self::ADDI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            AND(r1, r2, r3) => Self::AND(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            ANDI(r1, r2, i) => Self::ANDI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            DIV(r1, r2, r3) => Self::DIV(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            DIVI(r1, r2, i) => Self::DIVI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            EQ(r1, r2, r3) => Self::EQ(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            EXP(r1, r2, r3) => Self::EXP(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            EXPI(r1, r2, i) => Self::EXPI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            GT(r1, r2, r3) => Self::GT(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            LT(r1, r2, r3) => Self::LT(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            MLOG(r1, r2, r3) => Self::MLOG(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            MOD(r1, r2, r3) => Self::MOD(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            MODI(r1, r2, i) => Self::MODI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            MOVE(r1, r2) => Self::MOVE(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            MOVI(r1, i) => Self::MOVI(update_reg(reg_to_reg_map, r1), i.clone()),
            MROO(r1, r2, r3) => Self::MROO(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            MUL(r1, r2, r3) => Self::MUL(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            MULI(r1, r2, i) => Self::MULI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            NOOP => Self::NOOP,
            NOT(r1, r2) => Self::NOT(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            OR(r1, r2, r3) => Self::OR(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            ORI(r1, r2, i) => Self::ORI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            SLL(r1, r2, r3) => Self::SLL(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            SLLI(r1, r2, i) => Self::SLLI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            SRL(r1, r2, r3) => Self::SRL(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            SRLI(r1, r2, i) => Self::SRLI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            SUB(r1, r2, r3) => Self::SUB(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            SUBI(r1, r2, i) => Self::SUBI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            XOR(r1, r2, r3) => Self::XOR(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            XORI(r1, r2, i) => Self::XORI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            WQOP(r1, r2, r3, i) => Self::WQOP(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                i.clone(),
            ),
            WQML(r1, r2, r3, i) => Self::WQML(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                i.clone(),
            ),
            WQDV(r1, r2, r3, i) => Self::WQDV(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                i.clone(),
            ),
            WQMD(r1, r2, r3, r4) => Self::WQMD(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            WQCM(r1, r2, r3, i) => Self::WQCM(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                i.clone(),
            ),
            WQAM(r1, r2, r3, r4) => Self::WQAM(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            WQMM(r1, r2, r3, r4) => Self::WQMM(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),

            /* Control Flow Instructions */
            JMP(r1) => Self::JMP(update_reg(reg_to_reg_map, r1)),
            JI(_) => self.clone(),
            JNE(r1, r2, r3) => Self::JNE(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            JNEI(r1, r2, i) => Self::JNEI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            JNZI(r1, i) => Self::JNZI(update_reg(reg_to_reg_map, r1), i.clone()),
            RET(r1) => Self::RET(update_reg(reg_to_reg_map, r1)),

            /* Memory Instructions */
            ALOC(hp, r1) => Self::ALOC(hp.clone(), update_reg(reg_to_reg_map, r1)),
            CFEI(sp, i) => Self::CFEI(sp.clone(), i.clone()),
            CFSI(sp, i) => Self::CFSI(sp.clone(), i.clone()),
            CFE(sp, r1) => Self::CFE(sp.clone(), update_reg(reg_to_reg_map, r1)),
            CFS(sp, r1) => Self::CFS(sp.clone(), update_reg(reg_to_reg_map, r1)),
            LB(r1, r2, i) => Self::LB(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            LW(r1, r2, i) => Self::LW(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            MCL(r1, r2) => Self::MCL(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            MCLI(r1, i) => Self::MCLI(update_reg(reg_to_reg_map, r1), i.clone()),
            MCP(r1, r2, r3) => Self::MCP(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            MEQ(r1, r2, r3, r4) => Self::MEQ(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            MCPI(r1, r2, i) => Self::MCPI(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            SB(r1, r2, i) => Self::SB(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),
            SW(r1, r2, i) => Self::SW(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),

            /* Contract Instructions */
            BAL(r1, r2, r3) => Self::BAL(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            BHEI(r1) => Self::BHEI(update_reg(reg_to_reg_map, r1)),
            BHSH(r1, r2) => Self::BHSH(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            BURN(r1, r2) => Self::BURN(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            CALL(r1, r2, r3, r4) => Self::CALL(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            CB(r1) => Self::CB(update_reg(reg_to_reg_map, r1)),
            CCP(r1, r2, r3, r4) => Self::CCP(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            CROO(r1, r2) => Self::CROO(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            CSIZ(r1, r2) => Self::CSIZ(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            BSIZ(r1, r2) => Self::BSIZ(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            LDC(r1, r2, r3, i0) => Self::LDC(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                i0.clone(),
            ),
            BLDD(r1, r2, r3, r4) => Self::BLDD(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            LOG(r1, r2, r3, r4) => Self::LOG(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            LOGD(r1, r2, r3, r4) => Self::LOGD(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            MINT(r1, r2) => Self::MINT(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            RETD(r1, r2) => Self::RETD(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            RVRT(reg1) => Self::RVRT(update_reg(reg_to_reg_map, reg1)),
            SMO(r1, r2, r3, r4) => Self::SMO(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            SCWQ(r1, r2, r3) => Self::SCWQ(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            SRW(r1, r2, r3) => Self::SRW(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            SRWQ(r1, r2, r3, r4) => Self::SRWQ(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            SWW(r1, r2, r3) => Self::SWW(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            SWWQ(r1, r2, r3, r4) => Self::SWWQ(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            TIME(r1, r2) => Self::TIME(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
            ),
            TR(r1, r2, r3) => Self::TR(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            TRO(r1, r2, r3, r4) => Self::TRO(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),

            /* Cryptographic Instructions */
            ECK1(r1, r2, r3) => Self::ECK1(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            ECR1(r1, r2, r3) => Self::ECR1(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            ED19(r1, r2, r3, r4) => Self::ED19(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            K256(r1, r2, r3) => Self::K256(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            S256(r1, r2, r3) => Self::S256(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
            ),
            ECOP(r1, r2, r3, r4) => Self::ECOP(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            EPAR(r1, r2, r3, r4) => Self::EPAR(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),

            /* Other Instructions */
            ECAL(r1, r2, r3, r4) => Self::ECAL(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
                update_reg(reg_to_reg_map, r4),
            ),
            FLAG(r1) => Self::FLAG(update_reg(reg_to_reg_map, r1)),
            GM(r1, i) => Self::GM(update_reg(reg_to_reg_map, r1), i.clone()),
            GTF(r1, r2, i) => Self::GTF(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                i.clone(),
            ),

            /* Non-VM Instructions */
            BLOB(i) => Self::BLOB(i.clone()),
            DataSectionOffsetPlaceholder => Self::DataSectionOffsetPlaceholder,
            ConfigurablesOffsetPlaceholder => Self::ConfigurablesOffsetPlaceholder,
            LoadDataId(r1, i) => Self::LoadDataId(update_reg(reg_to_reg_map, r1), i.clone()),
            AddrDataId(r1, i) => Self::AddrDataId(update_reg(reg_to_reg_map, r1), i.clone()),
            Undefined => Self::Undefined,
        }
    }

    /// Use `offset_map` to update the immediate value of a jump instruction. The map simply tells
    /// us what the new offset should be given the existing offset.
    pub(crate) fn update_jump_immediate_values(&mut self, offset_map: &HashMap<u64, u64>) -> Self {
        use VirtualOp::*;
        match self {
            JI(i) => Self::JI(
                VirtualImmediate24::new(
                    *offset_map
                        .get(&(i.value() as u64))
                        .expect("new offset should be valid"),
                    crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                )
                .unwrap(),
            ),
            JNEI(r1, r2, i) => Self::JNEI(
                r1.clone(),
                r2.clone(),
                VirtualImmediate12::new(
                    *offset_map
                        .get(&(i.value() as u64))
                        .expect("new offset should be valid"),
                    crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                )
                .unwrap(),
            ),
            JNZI(r1, i) => Self::JNZI(
                r1.clone(),
                VirtualImmediate18::new(
                    *offset_map
                        .get(&(i.value() as u64))
                        .expect("new offset should be valid"),
                    crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                )
                .unwrap(),
            ),

            _ => self.clone(),
        }
    }

    pub(crate) fn allocate_registers(&self, pool: &RegisterPool) -> AllocatedOpcode {
        let virtual_registers = self.registers();
        let register_allocation_result = virtual_registers
            .into_iter()
            .map(|x| match x {
                VirtualRegister::Constant(c) => (x, Some(AllocatedRegister::Constant(*c))),
                VirtualRegister::Virtual(_) => (x, pool.get_register(x)),
            })
            .map(|(x, register_opt)| register_opt.map(|register| (x, register)))
            .collect::<Option<Vec<_>>>();

        // Maps virtual registers to their allocated equivalent
        let mut mapping = HashMap::default();
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

        use VirtualOp::*;
        match self {
            /* Arithmetic/Logic (ALU) Instructions */
            ADD(reg1, reg2, reg3) => AllocatedOpcode::ADD(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ADDI(reg1, reg2, imm) => AllocatedOpcode::ADDI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            AND(reg1, reg2, reg3) => AllocatedOpcode::AND(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ANDI(reg1, reg2, imm) => AllocatedOpcode::ANDI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            DIV(reg1, reg2, reg3) => AllocatedOpcode::DIV(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            DIVI(reg1, reg2, imm) => AllocatedOpcode::DIVI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            EQ(reg1, reg2, reg3) => AllocatedOpcode::EQ(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            EXP(reg1, reg2, reg3) => AllocatedOpcode::EXP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            EXPI(reg1, reg2, imm) => AllocatedOpcode::EXPI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            GT(reg1, reg2, reg3) => AllocatedOpcode::GT(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            LT(reg1, reg2, reg3) => AllocatedOpcode::LT(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MLOG(reg1, reg2, reg3) => AllocatedOpcode::MLOG(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MOD(reg1, reg2, reg3) => AllocatedOpcode::MOD(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MODI(reg1, reg2, imm) => AllocatedOpcode::MODI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            MOVE(reg1, reg2) => {
                AllocatedOpcode::MOVE(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            MOVI(reg1, imm) => AllocatedOpcode::MOVI(map_reg(&mapping, reg1), imm.clone()),
            MROO(reg1, reg2, reg3) => AllocatedOpcode::MROO(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MUL(reg1, reg2, reg3) => AllocatedOpcode::MUL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MULI(reg1, reg2, imm) => AllocatedOpcode::MULI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            NOOP => AllocatedOpcode::NOOP,
            NOT(reg1, reg2) => {
                AllocatedOpcode::NOT(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            OR(reg1, reg2, reg3) => AllocatedOpcode::OR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ORI(reg1, reg2, imm) => AllocatedOpcode::ORI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            SLL(reg1, reg2, reg3) => AllocatedOpcode::SLL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SLLI(reg1, reg2, imm) => AllocatedOpcode::SLLI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            SRL(reg1, reg2, reg3) => AllocatedOpcode::SRL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SRLI(reg1, reg2, imm) => AllocatedOpcode::SRLI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            SUB(reg1, reg2, reg3) => AllocatedOpcode::SUB(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SUBI(reg1, reg2, imm) => AllocatedOpcode::SUBI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            XOR(reg1, reg2, reg3) => AllocatedOpcode::XOR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            XORI(reg1, reg2, imm) => AllocatedOpcode::XORI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            WQOP(reg1, reg2, reg3, imm) => AllocatedOpcode::WQOP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                imm.clone(),
            ),
            WQML(reg1, reg2, reg3, imm) => AllocatedOpcode::WQML(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                imm.clone(),
            ),
            WQDV(reg1, reg2, reg3, imm) => AllocatedOpcode::WQDV(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                imm.clone(),
            ),
            WQMD(reg1, reg2, reg3, reg4) => AllocatedOpcode::WQMD(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            WQCM(reg1, reg2, reg3, imm) => AllocatedOpcode::WQCM(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                imm.clone(),
            ),
            WQAM(reg1, reg2, reg3, reg4) => AllocatedOpcode::WQAM(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            WQMM(reg1, reg2, reg3, reg4) => AllocatedOpcode::WQMM(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),

            /* Control Flow Instructions */
            JMP(reg1) => AllocatedOpcode::JMP(map_reg(&mapping, reg1)),
            JI(imm) => AllocatedOpcode::JI(imm.clone()),
            JNE(reg1, reg2, reg3) => AllocatedOpcode::JNE(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            JNEI(reg1, reg2, imm) => AllocatedOpcode::JNEI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            JNZI(reg1, imm) => AllocatedOpcode::JNZI(map_reg(&mapping, reg1), imm.clone()),
            RET(reg) => AllocatedOpcode::RET(map_reg(&mapping, reg)),

            /* Memory Instructions */
            ALOC(_hp, reg) => AllocatedOpcode::ALOC(map_reg(&mapping, reg)),
            CFEI(_sp, imm) => AllocatedOpcode::CFEI(imm.clone()),
            CFSI(_sp, imm) => AllocatedOpcode::CFSI(imm.clone()),
            CFE(_sp, reg) => AllocatedOpcode::CFE(map_reg(&mapping, reg)),
            CFS(_sp, reg) => AllocatedOpcode::CFS(map_reg(&mapping, reg)),
            LB(reg1, reg2, imm) => AllocatedOpcode::LB(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            LW(reg1, reg2, imm) => AllocatedOpcode::LW(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            MCL(reg1, reg2) => {
                AllocatedOpcode::MCL(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            MCLI(reg1, imm) => AllocatedOpcode::MCLI(map_reg(&mapping, reg1), imm.clone()),
            MCP(reg1, reg2, reg3) => AllocatedOpcode::MCP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MCPI(reg1, reg2, imm) => AllocatedOpcode::MCPI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            MEQ(reg1, reg2, reg3, reg4) => AllocatedOpcode::MEQ(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            SB(reg1, reg2, imm) => AllocatedOpcode::SB(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            SW(reg1, reg2, imm) => AllocatedOpcode::SW(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),

            /* Contract Instructions */
            BAL(reg1, reg2, reg3) => AllocatedOpcode::BAL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            BHEI(reg1) => AllocatedOpcode::BHEI(map_reg(&mapping, reg1)),
            BHSH(reg1, reg2) => {
                AllocatedOpcode::BHSH(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            BURN(reg1, reg2) => {
                AllocatedOpcode::BURN(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            CALL(reg1, reg2, reg3, reg4) => AllocatedOpcode::CALL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            CB(reg1) => AllocatedOpcode::CB(map_reg(&mapping, reg1)),
            CCP(reg1, reg2, reg3, reg4) => AllocatedOpcode::CCP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            CROO(reg1, reg2) => {
                AllocatedOpcode::CROO(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            CSIZ(reg1, reg2) => {
                AllocatedOpcode::CSIZ(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            BSIZ(reg1, reg2) => {
                AllocatedOpcode::BSIZ(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            LDC(reg1, reg2, reg3, imm0) => AllocatedOpcode::LDC(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                imm0.clone(),
            ),
            BLDD(reg1, reg2, reg3, reg4) => AllocatedOpcode::BLDD(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            LOG(reg1, reg2, reg3, reg4) => AllocatedOpcode::LOG(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            LOGD(reg1, reg2, reg3, reg4) => AllocatedOpcode::LOGD(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            MINT(reg1, reg2) => {
                AllocatedOpcode::MINT(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            RETD(reg1, reg2) => {
                AllocatedOpcode::RETD(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            RVRT(reg1) => AllocatedOpcode::RVRT(map_reg(&mapping, reg1)),
            SMO(reg1, reg2, reg3, reg4) => AllocatedOpcode::SMO(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            SCWQ(reg1, reg2, reg3) => AllocatedOpcode::SCWQ(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SRW(reg1, reg2, reg3) => AllocatedOpcode::SRW(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SRWQ(reg1, reg2, reg3, reg4) => AllocatedOpcode::SRWQ(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            SWW(reg1, reg2, reg3) => AllocatedOpcode::SWW(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SWWQ(reg1, reg2, reg3, reg4) => AllocatedOpcode::SWWQ(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            TIME(reg1, reg2) => {
                AllocatedOpcode::TIME(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            TR(reg1, reg2, reg3) => AllocatedOpcode::TR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            TRO(reg1, reg2, reg3, reg4) => AllocatedOpcode::TRO(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),

            /* Cryptographic Instructions */
            ECK1(reg1, reg2, reg3) => AllocatedOpcode::ECK1(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ECR1(reg1, reg2, reg3) => AllocatedOpcode::ECR1(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ED19(reg1, reg2, reg3, reg4) => AllocatedOpcode::ED19(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            K256(reg1, reg2, reg3) => AllocatedOpcode::K256(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            S256(reg1, reg2, reg3) => AllocatedOpcode::S256(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            ECOP(reg1, reg2, reg3, reg4) => AllocatedOpcode::ECOP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            EPAR(reg1, reg2, reg3, reg4) => AllocatedOpcode::EPAR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),

            /* Other Instructions */
            ECAL(reg1, reg2, reg3, reg4) => AllocatedOpcode::ECAL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
            FLAG(reg) => AllocatedOpcode::FLAG(map_reg(&mapping, reg)),
            GM(reg, imm) => AllocatedOpcode::GM(map_reg(&mapping, reg), imm.clone()),
            GTF(reg1, reg2, imm) => AllocatedOpcode::GTF(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),

            /* Non-VM Instructions */
            BLOB(imm) => AllocatedOpcode::BLOB(imm.clone()),
            DataSectionOffsetPlaceholder => AllocatedOpcode::DataSectionOffsetPlaceholder,
            ConfigurablesOffsetPlaceholder => AllocatedOpcode::ConfigurablesOffsetPlaceholder,
            LoadDataId(reg1, label) => {
                AllocatedOpcode::LoadDataId(map_reg(&mapping, reg1), label.clone())
            }
            AddrDataId(reg1, label) => {
                AllocatedOpcode::AddrDataId(map_reg(&mapping, reg1), label.clone())
            }
            Undefined => AllocatedOpcode::Undefined,
        }
    }
}

/// An unchecked function which serves as a convenience for looking up register mappings
fn map_reg(
    mapping: &HashMap<&VirtualRegister, AllocatedRegister>,
    reg: &VirtualRegister,
) -> AllocatedRegister {
    mapping.get(reg).unwrap().clone()
}

fn update_reg(
    reg_to_reg_map: &IndexMap<&VirtualRegister, &VirtualRegister>,
    reg: &VirtualRegister,
) -> VirtualRegister {
    if let Some(r) = reg_to_reg_map.get(reg) {
        assert!(reg.is_virtual(), "Only virtual registers should be updated");
        (*r).into()
    } else {
        reg.clone()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
/// A label for a spot in the bytecode, to be later compiled to an offset.
pub struct Label(pub(crate) usize);
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}", self.0)
    }
}
