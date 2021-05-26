//! This module contains abstracted versions of bytecode primitives that the compiler uses to
//! ensure correctness and safety.
//!
//! The immediate types are used to safely construct numbers that are within their bounds, and the
//! ops are clones of the actual opcodes, but with the safe primitives as arguments.

use super::{
    allocated_ops::{AllocatedOpcode, AllocatedRegister},
    DataId, RealizedOp,
};
use crate::asm_generation::RegisterPool;
use crate::error::*;
use pest::Span;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fmt;

/// Represents virtual registers that have yet to be allocated.
/// Note that only the Virtual variant will be allocated, and the Constant variant refers to
/// reserved registers.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum VirtualRegister {
    Virtual(String),
    Constant(ConstantRegister),
}

impl Into<VirtualRegister> for &VirtualRegister {
    fn into(self) -> VirtualRegister {
        self.clone()
    }
}

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirtualRegister::Virtual(name) => write!(f, "$r{}", name),
            VirtualRegister::Constant(name) => {
                write!(f, "{}", name)
            }
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
/// These are the special registers defined in the spec
pub enum ConstantRegister {
    // Below are VM-reserved registers
    Zero,
    One,
    Overflow,
    ProgramCounter,
    StackStartPointer,
    StackPointer,
    FramePointer,
    HeapPointer,
    Error,
    GlobalGas,
    ContextGas,
    Balance,
    InstructionStart,
    Flags,
    // Below are compiler-reserved registers
    DataSectionStart,
}

impl ConstantRegister {
    pub(crate) fn to_register_id(&self) -> fuel_asm::RegisterId {
        use ConstantRegister::*;
        match self {
            Zero => 0,
            One => 1,
            Overflow => 2,
            ProgramCounter => 3,
            StackStartPointer => 4,
            StackPointer => 5,
            FramePointer => 6,
            HeapPointer => 7,
            Error => 8,
            GlobalGas => 9,
            ContextGas => 10,
            Balance => 11,
            InstructionStart => 12,
            Flags => 13,
            DataSectionStart => {
                (crate::asm_generation::compiler_constants::DATA_SECTION_REGISTER())
                    as fuel_asm::RegisterId
            }
        }
    }
}

impl fmt::Display for ConstantRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ConstantRegister::*;
        let text = match self {
            Zero => "$zero",
            One => "$one",
            Overflow => "$of",
            ProgramCounter => "$pc",
            StackStartPointer => "$ssp",
            StackPointer => "$sp",
            FramePointer => "$fp",
            HeapPointer => "$hp",
            Error => "$err",
            GlobalGas => "$ggas",
            ContextGas => "$cgas",
            Balance => "$bal",
            InstructionStart => "$is",
            Flags => "$flag",
            DataSectionStart => "$ds",
        };
        write!(f, "{}", text)
    }
}

/// 6-bits immediate value type
#[derive(Clone)]
pub struct VirtualImmediate06 {
    pub(crate) value: u8,
}

impl VirtualImmediate06 {
    pub(crate) fn new<'sc>(raw: u64, err_msg_span: Span<'sc>) -> Result<Self, CompileError<'sc>> {
        if raw > 0b111_111 {
            return Err(CompileError::Immediate06TooLarge {
                val: raw,
                span: err_msg_span,
            });
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
}
impl fmt::Display for VirtualImmediate06 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.value)
    }
}

/// 12-bits immediate value type
#[derive(Clone)]
pub struct VirtualImmediate12 {
    pub(crate) value: u16,
}

impl VirtualImmediate12 {
    pub(crate) fn new<'sc>(raw: u64, err_msg_span: Span<'sc>) -> Result<Self, CompileError<'sc>> {
        if raw > 0b111_111_111_111 {
            return Err(CompileError::Immediate12TooLarge {
                val: raw,
                span: err_msg_span,
            });
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().expect(&(msg.into())),
        }
    }
}
impl fmt::Display for VirtualImmediate12 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.value)
    }
}

/// 18-bits immediate value type
#[derive(Clone)]
pub struct VirtualImmediate18 {
    pub(crate) value: u32,
}
impl VirtualImmediate18 {
    pub(crate) fn new<'sc>(raw: u64, err_msg_span: Span<'sc>) -> Result<Self, CompileError<'sc>> {
        if raw > crate::asm_generation::compiler_constants::EIGHTEEN_BITS {
            return Err(CompileError::Immediate18TooLarge {
                val: raw,
                span: err_msg_span,
            });
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().expect(&(msg.into())),
        }
    }
}
impl fmt::Display for VirtualImmediate18 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.value)
    }
}

/// 24-bits immediate value type
#[derive(Clone)]
pub struct VirtualImmediate24 {
    pub(crate) value: u32,
}
impl VirtualImmediate24 {
    pub(crate) fn new<'sc>(raw: u64, err_msg_span: Span<'sc>) -> Result<Self, CompileError<'sc>> {
        if raw > crate::asm_generation::compiler_constants::TWENTY_FOUR_BITS {
            return Err(CompileError::Immediate24TooLarge {
                val: raw,
                span: err_msg_span,
            });
        } else {
            Ok(Self {
                value: raw.try_into().unwrap(),
            })
        }
    }
    /// This method should only be used if the size of the raw value has already been manually
    /// checked.
    /// This is valuable when you don't necessarily have exact [Span] info and want to handle the
    /// error at a higher level, probably via an internal compiler error or similar.
    /// A panic message is still required, just in case the programmer has made an error.
    pub(crate) fn new_unchecked(raw: u64, msg: impl Into<String>) -> Self {
        Self {
            value: raw.try_into().expect(&(msg.into())),
        }
    }
}
impl fmt::Display for VirtualImmediate24 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i{}", self.value)
    }
}

/// This enum is unfortunately a redundancy of the [fuel_asm::Opcode] enum. This variant, however,
/// allows me to use the compiler's internal [VirtualRegister] types and maintain type safety
/// between virtual ops and the real opcodes. A bit of copy/paste seemed worth it for that safety,
/// so here it is.
#[derive(Clone)]
pub(crate) enum VirtualOp {
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
    MLOG(VirtualRegister, VirtualRegister, VirtualRegister),
    MROO(VirtualRegister, VirtualRegister, VirtualRegister),
    MOD(VirtualRegister, VirtualRegister, VirtualRegister),
    MODI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    MOVE(VirtualRegister, VirtualRegister),
    MUL(VirtualRegister, VirtualRegister, VirtualRegister),
    MULI(VirtualRegister, VirtualRegister, VirtualImmediate12),
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
    CIMV(VirtualRegister, VirtualRegister, VirtualRegister),
    CTMV(VirtualRegister, VirtualRegister),
    JI(VirtualImmediate24),
    JNEI(VirtualRegister, VirtualRegister, VirtualImmediate12),
    RET(VirtualRegister),
    CFEI(VirtualImmediate24),
    CFSI(VirtualImmediate24),
    LB(VirtualRegister, VirtualRegister, VirtualImmediate12),
    // LW takes a virtual register and a DataId, which points to a labeled piece
    // of data in the data section. Note that the ASM op corresponding to a LW is
    // subtly complex: $rB is in bytes and points to some mem address. The immediate
    // third argument is a _word_ offset from that byte address.
    LW(VirtualRegister, DataId),
    ALOC(VirtualRegister),
    MCL(VirtualRegister, VirtualRegister),
    MCLI(VirtualRegister, VirtualImmediate18),
    MCP(VirtualRegister, VirtualRegister, VirtualRegister),
    MEQ(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    SB(VirtualRegister, VirtualRegister, VirtualImmediate12),
    SW(VirtualRegister, VirtualRegister, VirtualImmediate12),
    BHSH(VirtualRegister, VirtualRegister),
    BHEI(VirtualRegister),
    BURN(VirtualRegister),
    CALL(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    CCP(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    CROO(VirtualRegister, VirtualRegister),
    CSIZ(VirtualRegister, VirtualRegister),
    CB(VirtualRegister),
    LDC(VirtualRegister, VirtualRegister, VirtualRegister),
    LOG(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    MINT(VirtualRegister),
    RVRT(VirtualRegister),
    SLDC(VirtualRegister, VirtualRegister, VirtualRegister),
    SRW(VirtualRegister, VirtualRegister),
    SRWQ(VirtualRegister, VirtualRegister),
    SWW(VirtualRegister, VirtualRegister),
    SWWQ(VirtualRegister, VirtualRegister),
    TR(VirtualRegister, VirtualRegister, VirtualRegister),
    TRO(
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
        VirtualRegister,
    ),
    ECR(VirtualRegister, VirtualRegister, VirtualRegister),
    K256(VirtualRegister, VirtualRegister, VirtualRegister),
    S256(VirtualRegister, VirtualRegister, VirtualRegister),
    NOOP,
    FLAG(VirtualRegister),
    Undefined,
    DataSectionOffsetPlaceholder,
    DataSectionRegisterLoadPlaceholder,
}

impl VirtualOp {
    pub(crate) fn registers(&self) -> HashSet<&VirtualRegister> {
        use VirtualOp::*;
        (match self {
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
            MLOG(r1, r2, r3) => vec![r1, r2, r3],
            MROO(r1, r2, r3) => vec![r1, r2, r3],
            MOD(r1, r2, r3) => vec![r1, r2, r3],
            MODI(r1, r2, _i) => vec![r1, r2],
            MOVE(r1, r2) => vec![r1, r2],
            MUL(r1, r2, r3) => vec![r1, r2, r3],
            MULI(r1, r2, _i) => vec![r1, r2],
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
            CIMV(r1, r2, r3) => vec![r1, r2, r3],
            CTMV(r1, r2) => vec![r1, r2],
            JI(_im) => vec![],
            JNEI(r1, r2, _i) => vec![r1, r2],
            RET(r1) => vec![r1],
            CFEI(_imm) => vec![],
            CFSI(_imm) => vec![],
            LB(r1, r2, _i) => vec![r1, r2],
            LW(r1, _i) => vec![r1],
            ALOC(_imm) => vec![],
            MCL(r1, r2) => vec![r1, r2],
            MCLI(r1, _imm) => vec![r1],
            MCP(r1, r2, r3) => vec![r1, r2, r3],
            MEQ(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            SB(r1, r2, _i) => vec![r1, r2],
            SW(r1, r2, _i) => vec![r1, r2],
            BHSH(r1, r2) => vec![r1, r2],
            BHEI(r1) => vec![r1],
            BURN(r1) => vec![r1],
            CALL(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CCP(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            CROO(r1, r2) => vec![r1, r2],
            CSIZ(r1, r2) => vec![r1, r2],
            CB(r1) => vec![r1],
            LDC(r1, r2, r3) => vec![r1, r2, r3],
            LOG(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            MINT(r1) => vec![r1],
            RVRT(r1) => vec![r1],
            SLDC(r1, r2, r3) => vec![r1, r2, r3],
            SRW(r1, r2) => vec![r1, r2],
            SRWQ(r1, r2) => vec![r1, r2],
            SWW(r1, r2) => vec![r1, r2],
            SWWQ(r1, r2) => vec![r1, r2],
            TR(r1, r2, r3) => vec![r1, r2, r3],
            TRO(r1, r2, r3, r4) => vec![r1, r2, r3, r4],
            ECR(r1, r2, r3) => vec![r1, r2, r3],
            K256(r1, r2, r3) => vec![r1, r2, r3],
            S256(r1, r2, r3) => vec![r1, r2, r3],
            NOOP => vec![],
            FLAG(r1) => vec![r1],
            Undefined | DataSectionOffsetPlaceholder => vec![],
            DataSectionRegisterLoadPlaceholder => vec![
                &VirtualRegister::Constant(ConstantRegister::DataSectionStart),
                &VirtualRegister::Constant(ConstantRegister::InstructionStart),
            ],
        })
        .into_iter()
        .collect()
    }

    pub(crate) fn allocate_registers(
        &self,
        pool: &mut RegisterPool,
        op_register_mapping: &[(RealizedOp, HashSet<VirtualRegister>)],
        ix: usize,
    ) -> AllocatedOpcode {
        let virtual_registers = self.registers();
        let register_allocation_result = virtual_registers
            .clone()
            .into_iter()
            .map(|x| (x, pool.get_register(x, &op_register_mapping[ix + 1..])))
            .map(|(x, res)| match res {
                Some(res) => Some((x, res)),
                None => None,
            })
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
                unimplemented!("The allocator cannot resolve a register mapping for this program. This is a temporary artifact of the extremely early stage version of this language. Try to lower the number of variables you use.")
            }
        };

        use VirtualOp::*;
        match self {
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
            MLOG(reg1, reg2, reg3) => AllocatedOpcode::MLOG(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            MROO(reg1, reg2, reg3) => AllocatedOpcode::MROO(
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
            CIMV(reg1, reg2, reg3) => AllocatedOpcode::CIMV(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            CTMV(reg1, reg2) => {
                AllocatedOpcode::CTMV(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            JI(imm) => AllocatedOpcode::JI(imm.clone()),
            JNEI(reg1, reg2, imm) => AllocatedOpcode::JNEI(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            RET(reg) => AllocatedOpcode::RET(map_reg(&mapping, reg)),
            CFEI(imm) => AllocatedOpcode::CFEI(imm.clone()),
            CFSI(imm) => AllocatedOpcode::CFSI(imm.clone()),
            LB(reg1, reg2, imm) => AllocatedOpcode::LB(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                imm.clone(),
            ),
            LW(reg1, imm) => AllocatedOpcode::LW(map_reg(&mapping, reg1), imm.clone()),
            ALOC(reg) => AllocatedOpcode::ALOC(map_reg(&mapping, reg)),
            MCL(reg1, reg2) => {
                AllocatedOpcode::MCL(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            MCLI(reg1, imm) => AllocatedOpcode::MCLI(map_reg(&mapping, reg1), imm.clone()),
            MCP(reg1, reg2, reg3) => AllocatedOpcode::MCP(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
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
            BHSH(reg1, reg2) => {
                AllocatedOpcode::BHSH(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            BHEI(reg1) => AllocatedOpcode::BHEI(map_reg(&mapping, reg1)),
            BURN(reg1) => AllocatedOpcode::BURN(map_reg(&mapping, reg1)),
            CALL(reg1, reg2, reg3, reg4) => AllocatedOpcode::CALL(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
                map_reg(&mapping, reg4),
            ),
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
            CB(reg1) => AllocatedOpcode::CB(map_reg(&mapping, reg1)),
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
            MINT(reg1) => AllocatedOpcode::MINT(map_reg(&mapping, reg1)),
            RVRT(reg1) => AllocatedOpcode::RVRT(map_reg(&mapping, reg1)),
            SLDC(reg1, reg2, reg3) => AllocatedOpcode::SLDC(
                map_reg(&mapping, reg1),
                map_reg(&mapping, reg2),
                map_reg(&mapping, reg3),
            ),
            SRW(reg1, reg2) => {
                AllocatedOpcode::SRW(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            SRWQ(reg1, reg2) => {
                AllocatedOpcode::SRWQ(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            SWW(reg1, reg2) => {
                AllocatedOpcode::SWW(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
            }
            SWWQ(reg1, reg2) => {
                AllocatedOpcode::SWWQ(map_reg(&mapping, reg1), map_reg(&mapping, reg2))
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
            NOOP => AllocatedOpcode::NOOP,
            FLAG(reg) => AllocatedOpcode::FLAG(map_reg(&mapping, reg)),
            Undefined => AllocatedOpcode::Undefined,
            DataSectionOffsetPlaceholder => AllocatedOpcode::DataSectionOffsetPlaceholder,
            DataSectionRegisterLoadPlaceholder => {
                AllocatedOpcode::DataSectionRegisterLoadPlaceholder
            }
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

#[derive(Clone, Eq, PartialEq, Hash)]
/// A label for a spot in the bytecode, to be later compiled to an offset.
pub(crate) struct Label(pub(crate) usize);
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ".{}", self.0)
    }
}
