/// The total number of registers available and the number of registers available for the compiler
/// to use. Registers reserved by the compiler are contained within these.
const NUM_TOTAL_REGISTERS: u8 = 64;
const NUM_FREE_REGISTERS: u8 = 48;

/// This is the number of registers reserved by the compiler. Adjust this number if a new
/// reservation must be made.
/// So far, the compiler-reserved registers are:
/// 1. DATA_SECTION_BEGIN - the offset to the read only data section.
/// 2. RETURN_ADDRESS - where a function must return to.
/// 3. RETURN_VALUE - the value returned by a _function_ call.
/// 4. SCRATCH - used for certain operations which need a register temporarily, such as JMP.
/// 5. LOCALS_BASE - base register for stack locals.
/// 6. ARGS - for passing arguments to function calls.
const NUM_COMPILER_RESERVED_REGISTERS: u8 = 5 + NUM_ARG_REGISTERS;

pub(crate) const DATA_SECTION_REGISTER: u8 = NUM_TOTAL_REGISTERS - 1;
pub(crate) const RETURN_ADDRESS_REGISTER: u8 = NUM_TOTAL_REGISTERS - 2;
pub(crate) const RETURN_VALUE_REGISTER: u8 = NUM_TOTAL_REGISTERS - 3;
pub(crate) const SCRATCH_REGISTER: u8 = NUM_TOTAL_REGISTERS - 4;
pub(crate) const LOCALS_BASE: u8 = NUM_TOTAL_REGISTERS - 5;

pub(crate) const NUM_ARG_REGISTERS: u8 = 6;
pub(crate) const ARG_REG0: u8 = NUM_TOTAL_REGISTERS - 6;
pub(crate) const ARG_REG1: u8 = NUM_TOTAL_REGISTERS - 7;
pub(crate) const ARG_REG2: u8 = NUM_TOTAL_REGISTERS - 8;
pub(crate) const ARG_REG3: u8 = NUM_TOTAL_REGISTERS - 9;
pub(crate) const ARG_REG4: u8 = NUM_TOTAL_REGISTERS - 10;
pub(crate) const ARG_REG5: u8 = NUM_TOTAL_REGISTERS - 11;

pub(crate) const FIRST_ALLOCATED_REGISTER: u8 = ARG_REG5 - 1;

pub(crate) const NUM_ALLOCATABLE_REGISTERS: u8 =
    NUM_FREE_REGISTERS - NUM_COMPILER_RESERVED_REGISTERS;

pub(crate) const TWENTY_FOUR_BITS: u64 = 0b1111_1111_1111_1111_1111_1111;
pub(crate) const EIGHTEEN_BITS: u64 = 0b11_1111_1111_1111_1111;
pub(crate) const TWELVE_BITS: u64 = 0b1111_1111_1111;
pub(crate) const SIX_BITS: u64 = 0b11_1111;

/// Some arbitrary values used for error codes.
pub(crate) const MISMATCHED_SELECTOR_REVERT_CODE: u32 = 123;
