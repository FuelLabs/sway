script;

mod impls;
use impls::*;

#[inline(always)]
fn assign_built_in_value_u8() {
    let mut x = 11u8;

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    *r_mut_x = 22u8;
    assert(x == 22u8);

    **r_mut_r_mut_x = 33u8;
    assert(x == 33u8);

    ***r_mut_r_mut_r_mut_x = 44u8;
    assert(x == 44u8);

    let r = & &mut & &mut x;
    ****r = 11;
    assert(x == 11);
}

#[inline(never)]
fn assign_built_in_value_u8_not_inlined() {
    assign_built_in_value_u8()
}

struct S {
    x: bool,
    y: u64,
}

impl PartialEq for S {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for S {}

#[inline(always)]
fn assign_struct_value() {
    let mut x = S {
        x: true,
        y: 111,
    };

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    *r_mut_x = S {
        x: false,
        y: 222,
    };
    assert(x == S {
        x: false,
        y: 222,
    });

    **r_mut_r_mut_x = S {
        x: true,
        y: 333,
    };
    assert(x == S {
        x: true,
        y: 333,
    });

    ***r_mut_r_mut_r_mut_x = S {
        x: false,
        y: 444,
    };
    assert(x == S {
        x: false,
        y: 444,
    });

    let r = & &mut & &mut x;
    ****r = S {
        x: true,
        y: 111,
    };
    assert(x == S {
        x: true,
        y: 111,
    });
}

#[inline(never)]
fn assign_struct_value_not_inlined() {
    assign_struct_value()
}

#[inline(always)]
fn assign_tuple_value() {
    let mut x = (true, 111);

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    *r_mut_x = (false, 222);
    assert(x == (false, 222));

    **r_mut_r_mut_x = (true, 333);
    assert(x == (true, 333));

    ***r_mut_r_mut_r_mut_x = (false, 444);
    assert(x == (false, 444));

    let r = & &mut & &mut x;
    ****r = (true, 111);
    assert(x == (true, 111));
}

#[inline(never)]
fn assign_tuple_value_not_inlined() {
    assign_tuple_value()
}

enum E {
    A: u64,
    B: bool,
    C: u8,
    D: u32,
}

impl PartialEq for E {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (E::A(l), E::A(r)) => l == r,
            (E::B(l), E::B(r)) => l == r,
            (E::C(l), E::C(r)) => l == r,
            (E::D(l), E::D(r)) => l == r,
            _ => false,
        }
    }
}
impl Eq for E {}

#[inline(always)]
fn assign_enum_value() {
    let mut x = E::A(111);

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    *r_mut_x = E::B(true);
    assert(x == E::B(true));

    **r_mut_r_mut_x = E::C(222);
    assert(x == E::C(222));

    ***r_mut_r_mut_r_mut_x = E::D(333);
    assert(x == E::D(333));

    let r = & &mut & &mut x;
    ****r = E::A(111);
    assert(x == E::A(111));
}

#[inline(never)]
fn assign_enum_value_not_inlined() {
    assign_enum_value()
}

#[inline(always)]
fn assign_array_value() {
    let mut x = [111, 222];

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    *r_mut_x = [333, 444];
    assert(x == [333, 444]);

    **r_mut_r_mut_x = [555, 666];
    assert(x == [555, 666]);

    ***r_mut_r_mut_r_mut_x = [777, 888];
    assert(x == [777, 888]);

    let r = & &mut & &mut x;
    ****r = [111, 222];
    assert(x == [111, 222]);
}

#[inline(never)]
fn assign_array_value_not_inlined() {
    assign_array_value()
}

#[inline(always)]
fn assign_value<T>()
where
    T: TestInstance + Eq,
{
    let mut x = T::new();

    let mut r_mut_x = &mut x;
    let mut r_mut_r_mut_x = &mut r_mut_x;
    let r_mut_r_mut_r_mut_x = &mut r_mut_r_mut_x;

    *r_mut_x = T::different();
    assert(x == T::different());

    **r_mut_r_mut_x = T::new();
    assert(x == T::new());

    ***r_mut_r_mut_r_mut_x = T::different();
    assert(x == T::different());

    let r = & &mut & &mut x;
    ****r = T::new();
    assert(x == T::new());
}

#[inline(never)]
fn assign_value_not_inlined<T>()
where
    T: TestInstance + Eq,
{
    assign_value::<T>()
}

#[inline(never)]
fn test_all_inlined() {
    assign_built_in_value_u8();
    assign_struct_value();
    assign_array_value();
    assign_tuple_value();
    assign_enum_value();

    assign_value::<()>();
    assign_value::<bool>();
    assign_value::<u8>();
    assign_value::<u16>();
    assign_value::<u32>();
    assign_value::<u64>();
    // TODO: Enable once https://github.com/FuelLabs/sway/issues/5833 get solved.
    // assign_value::<u256>();
    assign_value::<[u64; 2]>();
    assign_value::<[u64; 0]>();
    assign_value::<Struct>();
    assign_value::<EmptyStruct>();
    assign_value::<str>();
    assign_value::<str[6]>();
    assign_value::<Enum>();
    assign_value::<(u8, u32)>();
    // TODO: Enable once https://github.com/FuelLabs/sway/issues/5833 get solved.
    // assign_value::<b256>();
    assign_value::<raw_ptr>();
    assign_value::<raw_slice>();
}

#[inline(never)]
fn test_not_inlined() {
    assign_built_in_value_u8_not_inlined();
    assign_struct_value_not_inlined();
    assign_array_value_not_inlined();
    assign_tuple_value_not_inlined();
    assign_enum_value_not_inlined();

    assign_value_not_inlined::<()>();
    assign_value_not_inlined::<bool>();
    assign_value_not_inlined::<u8>();
    assign_value_not_inlined::<u16>();
    assign_value_not_inlined::<u32>();
    assign_value_not_inlined::<u64>();
    // TODO: Enable once https://github.com/FuelLabs/sway/issues/5833 get solved.
    // assign_value_not_inlined::<u256>();
    assign_value_not_inlined::<[u64; 2]>();
    assign_value_not_inlined::<[u64; 0]>();
    assign_value_not_inlined::<Struct>();
    assign_value_not_inlined::<EmptyStruct>();
    assign_value_not_inlined::<str>();
    assign_value_not_inlined::<str[6]>();
    assign_value_not_inlined::<Enum>();
    assign_value_not_inlined::<(u8, u32)>();
    // TODO: Enable once https://github.com/FuelLabs/sway/issues/5833 get solved.
    // assign_value_not_inlined::<b256>();
    assign_value_not_inlined::<raw_ptr>();
    assign_value_not_inlined::<raw_slice>();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    42
}
