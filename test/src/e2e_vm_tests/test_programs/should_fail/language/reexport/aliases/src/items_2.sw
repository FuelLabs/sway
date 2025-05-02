library;

pub struct Items2_Struct {
    pub b: u64,
}

pub enum Items2_Enum {
    C: u64,
    D: u64,
}

pub enum Items2_Variants {
    Z: u64,
    W: u64,
}

pub const ITEMS_2_FUNCTION_RES: u64 = 987;

pub fn items_2_function() -> u64 {
    ITEMS_2_FUNCTION_RES
}

pub trait Items2Trait<T> {
    fn items_2_trait_function(self, x: T) -> bool;
}
