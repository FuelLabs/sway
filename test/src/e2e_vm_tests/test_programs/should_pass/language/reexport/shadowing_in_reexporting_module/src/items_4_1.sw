library;

pub struct Items4_Struct {
    pub d: bool,
}

pub enum Items4_Enum {
    I: bool,
    J: bool,
}

pub enum Items4_Variants {
    K: bool,
    L: bool,
}

pub const ITEMS_4_FUNCTION_RES: u64 = 5678;

pub fn items_4_function() -> bool {
    ITEMS_4_FUNCTION_RES == 5678
}

pub trait Items4Trait<T> {
    fn items_4_trait_function(self, x: T) -> bool;
}
