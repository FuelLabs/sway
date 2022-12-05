script;

use std::assert::assert;

struct S {
    a: u64,
}

enum E {
    Variant: (),
}

fn arg_is_reference<T>(a: T) -> bool {
    __is_reference_type::<T>()
}

fn main() -> bool {
    assert(!__is_reference_type::<()>());        // Is Unit ref or not?
    assert(!__is_reference_type::<bool>());
    assert(!__is_reference_type::<u64>());

    assert(__is_reference_type::<str[1]>());
    assert(__is_reference_type::<b256>());
    assert(__is_reference_type::<S>());
    assert(__is_reference_type::<E>());
    assert(__is_reference_type::<(bool, bool)>());
    assert(__is_reference_type::<[u64; 2]>());

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
