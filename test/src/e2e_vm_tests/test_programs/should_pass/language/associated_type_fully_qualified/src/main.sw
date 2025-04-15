script;

trait TypeTrait1 {
    type T;
}

trait TypeTrait2 {
    type T;
}

trait ConstTrait {
    const C: u64;
}

struct Struct {}

struct Struct2 {}

impl TypeTrait1 for Struct {
    type T = u32;
}

impl TypeTrait2 for Struct {
    type T = Struct2;
}

impl ConstTrait for Struct2 {
    const C: u64 = 42u64;
}

fn main() -> u32 {
    let _i1: <Struct as TypeTrait1>::T = 1u32;

    assert_eq(<Struct as TypeTrait1>::T::max(), u32::max());

    assert_eq(<Struct as TypeTrait2>::T::C, 42);

    1
}
