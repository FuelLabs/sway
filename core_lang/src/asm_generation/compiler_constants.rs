pub(crate) const NUM_FREE_REGISTERS: u8 = 48;
pub(crate) const TWENTY_FOUR_BITS: u64 = 0b111_111_111_111_111_111_111_111;
pub(crate) const EIGHTEEN_BITS: u64 = 0b111_111_111_111_111_111;
pub(crate) const TWELVE_BITS: u64 = 0b111_111_111_111;

#[allow(non_snake_case)]
pub(crate) const fn DATA_SECTION_REGISTER() -> u8 {
    NUM_FREE_REGISTERS - 2
}
