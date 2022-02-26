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

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
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
}

impl ConstantRegister {
    pub(crate) fn to_register_id(&self) -> fuel_asm::RegisterId {
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
            DataSectionStart => {
                (crate::asm_generation::compiler_constants::DATA_SECTION_REGISTER)
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
            ReturnValue => "$ret",
            ReturnLength => "$retl",
            Flags => "$flag",
            // two `$` signs denotes this is a compiler-reserved register and not a
            // VM-reserved register
            DataSectionStart => "$$ds",
        };
        write!(f, "{}", text)
    }
}
