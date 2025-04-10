library;

pub struct Items4_Struct {
    pub d: u64,
}

pub enum Items4_Enum {
    I: u64,
    J: u64,
}

pub enum Items4_Variants {
    K: u64,
    L: u64,
}

pub const ITEMS_4_FUNCTION_RES: u64 = 8765;

pub fn items_4_function() -> u64 {
    ITEMS_4_FUNCTION_RES
}

pub trait Items4Trait<T> {
    fn items_4_trait_function(self, x: T) -> u64;
}
