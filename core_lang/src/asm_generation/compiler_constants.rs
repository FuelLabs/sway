/// The number of registers available for the compiler to use. Registers reserved by the
/// compiler are contained within these.
const NUM_FREE_REGISTERS: u8 = 48;
pub(crate) const TWENTY_FOUR_BITS: u64 = 0b111_111_111_111_111_111_111_111;
pub(crate) const EIGHTEEN_BITS: u64 = 0b111_111_111_111_111_111;
pub(crate) const TWELVE_BITS: u64 = 0b111_111_111_111;

/// This is the number of registers reserved by the compiler. Adjust this number if a new
/// reservation must be made.
/// So far, the compiler-reserved registers are:
/// 1. DATA_SECTION_BEGIN
const NUM_COMPILER_RESERVED_REGISTERS: u8 = 1;
pub(crate) const DATA_SECTION_REGISTER: u8 = NUM_FREE_REGISTERS - 2;
pub(crate) const NUM_ALLOCATABLE_REGISTERS: u8 =
    NUM_FREE_REGISTERS - NUM_COMPILER_RESERVED_REGISTERS;
