use crate::fuel_prelude::fuel_asm;
use std::fmt;

/// Represents virtual registers that have yet to be allocated.
/// Note that only the Virtual variant will be allocated, and the Constant variant refers to
/// reserved registers.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum VirtualRegister {
    Virtual(String),
    Constant(ConstantRegister),
}

impl From<&str> for VirtualRegister {
    fn from(value: &str) -> Self {
        VirtualRegister::Virtual(value.to_string())
    }
}

impl VirtualRegister {
    pub fn is_virtual(&self) -> bool {
        matches!(self, Self::Virtual(_))
    }
}

impl From<&VirtualRegister> for VirtualRegister {
    fn from(register: &VirtualRegister) -> VirtualRegister {
        register.clone()
    }
}

impl From<ConstantRegister> for VirtualRegister {
    fn from(constant_register: ConstantRegister) -> Self {
        VirtualRegister::Constant(constant_register)
    }
}

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VirtualRegister::Virtual(name) => write!(f, "$r{name}"),
            VirtualRegister::Constant(name) => {
                write!(f, "{name}")
            }
        }
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
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
    ReturnValue,
    ReturnLength,
    Flags,

    // Below are compiler-reserved registers
    DataSectionStart,
    CallReturnAddress,
    CallReturnValue,
    Scratch,
    LocalsBase,

    // Registers for the first NUM_ARG_REGISTERS function arguments.
    FuncArg0,
    FuncArg1,
    FuncArg2,
    FuncArg3,
    FuncArg4,
    FuncArg5,
}

impl ConstantRegister {
    pub(crate) fn parse_register_name(raw: &str) -> Option<ConstantRegister> {
        use ConstantRegister::*;
        Some(match raw {
            "zero" => Zero,
            "one" => One,
            "of" => Overflow,
            "pc" => ProgramCounter,
            "ssp" => StackStartPointer,
            "sp" => StackPointer,
            "fp" => FramePointer,
            "hp" => HeapPointer,
            "err" => Error,
            "ggas" => GlobalGas,
            "cgas" => ContextGas,
            "bal" => Balance,
            "is" => InstructionStart,
            "flag" => Flags,
            "retl" => ReturnLength,
            "ret" => ReturnValue,
            "ds" => DataSectionStart,
            _ => return None,
        })
    }
}

use crate::asm_generation::fuel::compiler_constants;

impl ConstantRegister {
    pub(crate) fn to_reg_id(self) -> fuel_asm::RegId {
        use ConstantRegister::*;
        match self {
            Zero => fuel_asm::RegId::ZERO,
            One => fuel_asm::RegId::ONE,
            Overflow => fuel_asm::RegId::OF,
            ProgramCounter => fuel_asm::RegId::PC,
            StackStartPointer => fuel_asm::RegId::SSP,
            StackPointer => fuel_asm::RegId::SP,
            FramePointer => fuel_asm::RegId::FP,
            HeapPointer => fuel_asm::RegId::HP,
            Error => fuel_asm::RegId::ERR,
            GlobalGas => fuel_asm::RegId::GGAS,
            ContextGas => fuel_asm::RegId::CGAS,
            Balance => fuel_asm::RegId::BAL,
            InstructionStart => fuel_asm::RegId::IS,
            ReturnValue => fuel_asm::RegId::RET,
            ReturnLength => fuel_asm::RegId::RETL,
            Flags => fuel_asm::RegId::FLAG,

            DataSectionStart => fuel_asm::RegId::new(compiler_constants::DATA_SECTION_REGISTER),
            CallReturnAddress => fuel_asm::RegId::new(compiler_constants::RETURN_ADDRESS_REGISTER),
            CallReturnValue => fuel_asm::RegId::new(compiler_constants::RETURN_VALUE_REGISTER),
            Scratch => fuel_asm::RegId::new(compiler_constants::SCRATCH_REGISTER),
            LocalsBase => fuel_asm::RegId::new(compiler_constants::LOCALS_BASE),

            FuncArg0 => fuel_asm::RegId::new(compiler_constants::ARG_REG0),
            FuncArg1 => fuel_asm::RegId::new(compiler_constants::ARG_REG1),
            FuncArg2 => fuel_asm::RegId::new(compiler_constants::ARG_REG2),
            FuncArg3 => fuel_asm::RegId::new(compiler_constants::ARG_REG3),
            FuncArg4 => fuel_asm::RegId::new(compiler_constants::ARG_REG4),
            FuncArg5 => fuel_asm::RegId::new(compiler_constants::ARG_REG5),
        }
    }

    pub(crate) const ARG_REGS: [ConstantRegister; compiler_constants::NUM_ARG_REGISTERS as usize] = [
        ConstantRegister::FuncArg0,
        ConstantRegister::FuncArg1,
        ConstantRegister::FuncArg2,
        ConstantRegister::FuncArg3,
        ConstantRegister::FuncArg4,
        ConstantRegister::FuncArg5,
    ];
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
            ReturnValue => "$ret",
            ReturnLength => "$retl",
            Flags => "$flag",
            // two `$` signs denotes this is a compiler-reserved register and not a
            // VM-reserved register
            DataSectionStart => "$$ds",
            CallReturnAddress => "$$reta",
            CallReturnValue => "$$retv",
            Scratch => "$$tmp",
            LocalsBase => "$$locbase",
            FuncArg0 => "$$arg0",
            FuncArg1 => "$$arg1",
            FuncArg2 => "$$arg2",
            FuncArg3 => "$$arg3",
            FuncArg4 => "$$arg4",
            FuncArg5 => "$$arg5",
        };
        write!(f, "{text}")
    }
}
