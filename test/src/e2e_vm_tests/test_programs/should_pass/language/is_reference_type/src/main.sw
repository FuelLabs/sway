script;

use std::assert::assert;

struct S {
    a: u64,
}

enum E {
    Variant: (),
}

fn arg_is_reference<T>(a: T) -> bool {
    is_reference_type::<T>()
}

fn main() -> bool {
    assert(!is_reference_type::<()>());        // Is Unit ref or not?
    assert(!is_reference_type::<bool>());
    assert(!is_reference_type::<byte>());
    assert(!is_reference_type::<u64>());

    assert(is_reference_type::<str[1]>());
    assert(is_reference_type::<b256>());
    assert(is_reference_type::<S>());
    assert(is_reference_type::<E>());
    assert(is_reference_type::<(bool, bool)>());
    assert(is_reference_type::<[u64; 2]>());

    assert(!arg_is_reference(()));
    assert(!arg_is_reference(false));
    assert(!arg_is_reference(0x2b));
    assert(!arg_is_reference(0));

    assert(arg_is_reference("breakfast"));
    assert(arg_is_reference(0xfefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefe));
    assert(arg_is_reference(S { a: 42 }));
    assert(arg_is_reference(E::Variant));
    assert(arg_is_reference((true, true)));
    assert(arg_is_reference([5, 4, 3, 2, 1]));

    true
}
