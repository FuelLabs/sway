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
            VirtualRegister::Virtual(name) => write!(f, "$r{}", name),
            VirtualRegister::Constant(name) => {
                write!(f, "{}", name)
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

    // Registers for the first NUM_ARG_REGISTERS function arguments.
    FuncArg0,
    FuncArg1,
    FuncArg2,
    FuncArg3,
    FuncArg4,
    FuncArg5,
}

use crate::asm_generation::compiler_constants;

impl ConstantRegister {
    pub(crate) fn to_register_id(self) -> fuel_asm::RegisterId {
        use fuel_vm::consts::*;
        use ConstantRegister::*;
        match self {
            Zero => REG_ZERO,
            One => REG_ONE,
            Overflow => REG_OF,
            ProgramCounter => REG_PC,
            StackStartPointer => REG_SSP,
            StackPointer => REG_SP,
            FramePointer => REG_FP,
            HeapPointer => REG_HP,
            Error => REG_ERR,
            GlobalGas => REG_GGAS,
            ContextGas => REG_CGAS,
            Balance => REG_BAL,
            InstructionStart => REG_IS,
            ReturnValue => REG_RET,
            ReturnLength => REG_RETL,
            Flags => REG_FLAG,

            DataSectionStart => (compiler_constants::DATA_SECTION_REGISTER) as fuel_asm::RegisterId,
            CallReturnAddress => {
                (compiler_constants::RETURN_ADDRESS_REGISTER) as fuel_asm::RegisterId
            }
            CallReturnValue => (compiler_constants::RETURN_VALUE_REGISTER) as fuel_asm::RegisterId,
            Scratch => (compiler_constants::SCRATCH_REGISTER) as fuel_asm::RegisterId,

            FuncArg0 => compiler_constants::ARG_REG0 as fuel_asm::RegisterId,
            FuncArg1 => compiler_constants::ARG_REG1 as fuel_asm::RegisterId,
            FuncArg2 => compiler_constants::ARG_REG2 as fuel_asm::RegisterId,
            FuncArg3 => compiler_constants::ARG_REG3 as fuel_asm::RegisterId,
            FuncArg4 => compiler_constants::ARG_REG4 as fuel_asm::RegisterId,
            FuncArg5 => compiler_constants::ARG_REG5 as fuel_asm::RegisterId,
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
