script;

trait TypeTrait1 {
    type T;
}

trait TypeTrait2 {
    type T;
}

struct Struct {}

impl TypeTrait1 for Struct {
    type T = u32;
}


impl TypeTrait2 for Struct {
    type T = u64;
}

fn main() -> u32 {
    let _i : Struct::T = 1;

    0
}
