library;

pub struct Items1_Struct {
    pub a: u64,
}

pub enum Items1_Enum {
    A: u64,
    B: u64,
}

pub enum Items1_Variants {
    X: u64,
    Y: u64,
}

pub const ITEMS_1_FUNCTION_RES: u64 = 456;

pub fn items_1_function() -> u64 {
    ITEMS_1_FUNCTION_RES
}

pub trait Items1Trait<T> {
    fn items_1_trait_function(self, x: T) -> bool;
}
