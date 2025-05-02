library;

pub use ::items_3_1::*;
pub use ::items_3_1::Items3_Variants::*;

pub struct Items3_Struct {
    pub c: u64,
}

pub enum Items3_Enum {
    E: u64,
    F: u64,
}

pub enum Items3_Variants {
    G: u64,
    H: u64,
}

pub const ITEMS_3_FUNCTION_RES: u64 = 4321;

pub fn items_3_function() -> u64 {
    ITEMS_3_FUNCTION_RES
}

pub trait Items3Trait<T> {
    fn items_3_trait_function(self, x: T) -> u64;
}
