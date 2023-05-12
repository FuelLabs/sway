script;

mod data_structures;
mod eq_impls;

use eq_impls::*;
use data_structures::*;

fn main() {
    let mut expected = Vec::new();
    expected.push(SomeEnum::a(0u32));
    expected.push(SomeEnum::a(1u32));

    assert(expected == expected);

    let mut expected = Vec::new();
    expected.push(SomeStruct { a: 0u32 });
    expected.push(SomeStruct { a: 1u32 });

    assert(expected == expected);
}
