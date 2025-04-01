library;

pub use ::items_1::*;

struct Items1_Struct {
    a: u64,
}

enum Items1_Enum {
    A: u64,
    B: u64,
}

const ITEMS_1_FUNCTION_RES: u64 = 654;

fn items_1_function() -> u64 {
    ITEMS_1_FUNCTION_RES
}

trait Items1Trait<T> {
    fn items_1_trait_function(self, x: T) -> u64;
}
