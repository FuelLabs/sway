library;

pub enum SingleU8 {
    A: u8,
}

pub enum SingleU64 {
    A: u64,
}

pub enum SingleBool {
    A: bool,
}

pub enum MultiUnits {
    A: (),
    B: (),
    C: (),
}

pub enum MultiOneByte {
    A: bool,
    B: u8,
    C: (),
}

pub enum U8AndU64 {
    A: u8,
    B: u64,
}

// Three words wide enum. The first word in the slot is reserved for the tag, the rest fills the enum.
pub enum SlotSize {
    A: u8,
    B: (u64, u64, u64),
}

// Three words wide enum. The first word is reserved for the tag.
pub enum LargerThanSlot {
    A: u8,
    B: (u64, u64, u64, u64, u64),
}

abi StorageEnum {
    // Setters
    #[storage(read, write)]
    fn read_write_enums() -> u64;
}
