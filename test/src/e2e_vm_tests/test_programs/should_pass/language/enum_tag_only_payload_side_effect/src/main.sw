library;

struct EmptyStruct {}

enum SomeEnum {
    A: EmptyStruct,
    B: (),
}

fn side_effect(ref mut x: u64) -> EmptyStruct {
    x += 1;
    EmptyStruct {}
}

#[test]
fn tag_only_enum_payload_side_effect_is_executed() {
    let mut i = 0;
    let e = SomeEnum::A(side_effect(i));

    // Even though `SomeEnum` is tag-only and the payload carries no data, the side effect
    // of the payload expression must still be executed at runtime.
    assert_eq(i, 1);

    poke(e);
}

#[inline(never)]
fn poke<T>(_t: T) {}
