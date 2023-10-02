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
    let _i1: <Struct as TypeTrait1>::T = 1u32;
    let _i2 : <Struct as TypeTrait2>::T = 1u64;

    // TODO
    //assert_eq(<Struct as TypeTrait1>::T::max(), u32::max());

    1
}
