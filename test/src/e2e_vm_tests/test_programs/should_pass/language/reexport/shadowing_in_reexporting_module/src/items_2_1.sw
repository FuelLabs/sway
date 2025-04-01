library;

pub struct Items2_Struct {
    pub b: bool,
}

pub enum Items2_Enum {
    C: bool,
    D: bool,
}

pub const ITEMS_2_FUNCTION_RES: u64 = 789;

pub fn items_2_function() -> bool {
    ITEMS_2_FUNCTION_RES == 789
}

pub trait Items2Trait<T> {
    fn items_2_trait_function(self, x: T) -> bool;
}
