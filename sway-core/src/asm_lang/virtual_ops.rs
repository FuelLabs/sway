//! This module contains abstracted versions of bytecode primitives that the compiler uses to
//! ensure correctness and safety.
//!
//! The immediate types are used to safely construct numbers that are within their bounds, and the
//! ops are clones of the actual opcodes, but with the safe primitives as arguments.

use super::{
    allocated_ops::{AllocatedOpcode, AllocatedRegister},
    virtual_immediate::*,
    virtual_register::*,
    DataId, Op,
};
use crate::asm_generation::RegisterPool;

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

    /* Control Flow Instructions */
    JMP(VirtualRegister),
    JI(VirtualImmediate24),
    JNE(VirtualRegister, VirtualRegister, VirtualRegister),
    JNEI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    JNZI(VirtualRegister, VirtualImmediate18),
    RET(VirtualRegister),

    /* Memory Instructions */
    ALOC(VirtualRegister),
    CFEI(VirtualImmediate24),
    CFSI(VirtualImmediate24),
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
    BURN(VirtualRegister),
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
    LDC(VirtualRegister, VirtualRegister, VirtualRegister),
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
    MINT(VirtualRegister),
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
    ECR(VirtualRegister, VirtualRegister, VirtualRegister),
    K256(VirtualRegister, VirtualRegister, VirtualRegister),
    S256(VirtualRegister, VirtualRegister, VirtualRegister),

    /* Other Instructions */
    FLAG(VirtualRegister),
    GM(VirtualRegister, VirtualImmediate18),
    GTF(VirtualRegister, VirtualRegister, VirtualImmediate12),

    /* Non-VM Instructions */
    BLOB(VirtualImmediate24),
    DataSectionOffsetPlaceholder,
    DataSectionRegisterLoadPlaceholder,
    // LWDataId takes a virtual register and a DataId, which points to a labeled piece
    // of data in the data section. Note that the ASM op corresponding to a LW is
    // subtly complex: $rB is in bytes and points to some mem address. The immediate
    // third argument is a _word_ offset from that byte address.
    LWDataId(VirtualRegister, DataId),
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

            /* Control Flow Instructions */
            JMP(r1) => vec![r1],
            JI(_im) => vec![],
            JNE(r1, r2, r3) => vec![r1, r2, r3],
            JNEI(r1, r2, _i) => vec![r1, r2],
            JNZI(r1, _i) => vec![r1],
            RET(r1) => vec![r1],

            /* Memory Instructions */
            ALOC(r1) => vec![r1],
            CFEI(_imm) => vec![],
            CFSI(_imm) => vec![],
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
            BURN(r1) => vec![r1],
            CALL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CB(r1) => vec![r1],
            CCP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CROO(r1, r2) => vec![r1, r2],
            CSIZ(r1, r2) => vec![r1, r2],
            LDC(r1, r2, r3) => vec![r1, r2, r3],
            LOG(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            LOGD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            MINT(r1) => vec![r1],
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
            ECR(r1, r2, r3) => vec![r1, r2, r3],
            K256(r1, r2, r3) => vec![r1, r2, r3],
            S256(r1, r2, r3) => vec![r1, r2, r3],

            /* Other Instructions */
            FLAG(r1) => vec![r1],
            GM(r1, _imm) => vec![r1],
            GTF(r1, r2, _i) => vec![r1, r2],

            /* Non-VM Instructions */
            BLOB(_imm) => vec![],
            DataSectionOffsetPlaceholder => vec![],
            DataSectionRegisterLoadPlaceholder => vec![
                &VirtualRegister::Constant(ConstantRegister::DataSectionStart),
                &VirtualRegister::Constant(ConstantRegister::InstructionStart),
            ],
            LWDataId(r1, _i) => vec![r1],
            Undefined => vec![],
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
            MODI(r1, r2, _i) => vec![r1, r2],
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

            /* Control Flow Instructions */
            JMP(r1) => vec![r1],
            JI(_im) => vec![],
            JNE(r1, r2, r3) => vec![r1, r2, r3],
            JNEI(r1, r2, _i) => vec![r1, r2],
            JNZI(r1, _i) => vec![r1],
            RET(r1) => vec![r1],

            /* Memory Instructions */
            ALOC(r1) => vec![r1],
            CFEI(_imm) => vec![],
            CFSI(_imm) => vec![],
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
            BURN(r1) => vec![r1],
            CALL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CB(r1) => vec![r1],
            CCP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CROO(r1, r2) => vec![r1, r2],
            CSIZ(_r1, r2) => vec![r2],
            LDC(r1, r2, r3) => vec![r1, r2, r3],
            LOG(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            LOGD(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            MINT(r1) => vec![r1],
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
            ECR(r1, r2, r3) => vec![r1, r2, r3],
            K256(r1, r2, r3) => vec![r1, r2, r3],
            S256(r1, r2, r3) => vec![r1, r2, r3],

            /* Other Instructions */
            FLAG(r1) => vec![r1],
            GM(_r1, _imm) => vec![],
            GTF(_r1, r2, _i) => vec![r2],

            /* Non-VM Instructions */
            BLOB(_imm) => vec![],
            DataSectionOffsetPlaceholder => vec![],
            DataSectionRegisterLoadPlaceholder => vec![&VirtualRegister::Constant(
                ConstantRegister::InstructionStart,
            )],
            LWDataId(_r1, _i) => vec![],
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
            LWDataId(r1, _i) => vec![r1],
            DataSectionOffsetPlaceholder => vec![],
            DataSectionRegisterLoadPlaceholder => vec![&VirtualRegister::Constant(
                ConstantRegister::DataSectionStart,
            )],
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
        reg_to_reg_map: &HashMap<VirtualRegister, VirtualRegister>,
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
            ALOC(r1) => Self::ALOC(update_reg(reg_to_reg_map, r1)),
            CFEI(i) => Self::CFEI(i.clone()),
            CFSI(i) => Self::CFSI(i.clone()),
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
            BURN(r1) => Self::BURN(update_reg(reg_to_reg_map, r1)),
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
            LDC(r1, r2, r3) => Self::LDC(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
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
            MINT(r1) => Self::MINT(update_reg(reg_to_reg_map, r1)),
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
            ECR(r1, r2, r3) => Self::ECR(
                update_reg(reg_to_reg_map, r1),
                update_reg(reg_to_reg_map, r2),
                update_reg(reg_to_reg_map, r3),
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

            /* Other Instructions */
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
            DataSectionRegisterLoadPlaceholder => Self::DataSectionRegisterLoadPlaceholder,
            LWDataId(r1, i) => Self::LWDataId(update_reg(reg_to_reg_map, r1), i.clone()),
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
                        .get(&(i.value as u64))
                        .expect("new offset should be valid") as u64,
                    crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                )
                .unwrap(),
            ),
            JNEI(r1, r2, i) => Self::JNEI(
                r1.clone(),
                r2.clone(),
                VirtualImmediate12::new(
                    *offset_map
                        .get(&(i.value as u64))
                        .expect("new offset should be valid") as u64,
                    crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                )
                .unwrap(),
            ),
            JNZI(r1, i) => Self::JNZI(
                r1.clone(),
                VirtualImmediate18::new(
                    *offset_map
                        .get(&(i.value as u64))
                        .expect("new offset should be valid") as u64,
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
            .clone()
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
            ALOC(reg) => AllocatedOpcode::ALOC(map_reg(&mapping, reg)),
            CFEI(imm) => AllocatedOpcode::CFEI(imm.clone()),
            CFSI(imm) => AllocatedOpcode::CFSI(imm.clone()),
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
            BURN(reg1) => AllocatedOpcode::BURN(map_reg(&mapping, reg1)),
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
            LDC(reg1, reg2, reg3) => AllocatedOpcode::LDC(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
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
            MINT(reg1) => AllocatedOpcode::MINT(map_reg(&mapping, reg1)),
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
            ECR(reg1, reg2, reg3) => AllocatedOpcode::ECR(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
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

            /* Other Instructions */
            FLAG(reg) => AllocatedOpcode::FLAG(map_reg(&mapping, reg)),
            GM(reg, imm) => AllocatedOpcode::GM(map_reg(&mapping, reg), imm.clone()),
            GTF(reg1, reg2, imm) => AllocatedOpcode::GTF(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),

            /* Non-VM Instructions */
            BLOB(imm) => AllocatedOpcode::BLOB(imm.clone()),
            DataSectionOffsetPlaceholder => AllocatedOpcode::DataSectionOffsetPlaceholder(0),
            DataSectionRegisterLoadPlaceholder => {
                AllocatedOpcode::DataSectionRegisterLoadPlaceholder
            }
            LWDataId(reg1, label) => {
                AllocatedOpcode::LWDataId(map_reg(&mapping, reg1), label.clone())
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
    reg_to_reg_map: &HashMap<VirtualRegister, VirtualRegister>,
    reg: &VirtualRegister,
) -> VirtualRegister {
    if let Some(r) = reg_to_reg_map.get(reg) {
        r.clone()
    } else {
        reg.clone()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
/// A label for a spot in the bytecode, to be later compiled to an offset.
pub(crate) struct Label(pub(crate) usize);
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}", self.0)
    }
}
