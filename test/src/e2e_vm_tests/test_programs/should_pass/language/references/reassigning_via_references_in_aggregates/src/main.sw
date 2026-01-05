script;

struct A {
    r_u8: &mut u8,
    r_array: &mut [u64; 3],
}

impl A {
    fn new() -> Self {
        Self {
            r_u8: &mut 0,
            r_array: &mut [0, 0, 0],
        }
    }

    fn use_me(self) {
        poke(self.r_u8);
        poke(self.r_array);
    }
}

struct B {
    r_a: &A,
    r_array: &[&A; 3],
}

impl B {
    fn new() -> Self {
        let r_a = &A::new();
        Self {
            r_a: r_a,
            r_array: &[r_a, r_a, r_a],
        }
    }

    fn use_me(self) {
        poke(self.r_a);
        poke(self.r_array);
    }
}

#[inline(always)]
fn in_structs() {
    let mut x = 11u8;
    let mut array: [u64; 3] = [11, 11, 11];

    let a = A {
        r_u8: &mut x,
        r_array: &mut array,
    };
    let b = B {
        r_a: &a,
        r_array: &[&a, &a, &a],
    };

    *a.r_u8 = 22;
    assert_eq(x, 22);

    *a.r_array = [22, 22, 22];
    assert_eq(array, [22, 22, 22]);

    *b.r_a.r_u8 = 33;
    assert_eq(x, 33);

    *b.r_array[0].r_u8 = 44;
    assert_eq(x, 44);

    *b.r_array[0].r_array = [33, 33, 33];
    assert_eq(array, [33, 33, 33]);
}

#[inline(never)]
fn in_structs_not_inlined() {
    in_structs()
}

enum E {
    R_A: &A,
    R_B: &B,
}

#[inline(always)]
fn in_enums() {
    let mut x = 11u8;
    let mut array: [u64; 3] = [11, 11, 11];

    let a = A {
        r_u8: &mut x,
        r_array: &mut array,
    };
    let b = B {
        r_a: &a,
        r_array: &[&a, &a, &a],
    };

    let e_r_a = E::R_A(&a);
    let e_r_b = E::R_B(&b);

    match e_r_a {
        E::R_A(r_a) => {
            *r_a.r_u8 = 22;
            assert_eq(x, 22);

            *r_a.r_array = [22, 22, 22];
            assert_eq(array, [22, 22, 22]);
        }
        _ => assert(false),
    }

    match e_r_b {
        E::R_B(r_b) => {
            *r_b.r_a.r_u8 = 33;
            assert_eq(x, 33);

            *r_b.r_array[0].r_u8 = 44;
            assert_eq(x, 44);

            *r_b.r_array[0].r_array = [33, 33, 33];
            assert_eq(array, [33, 33, 33]);
        }
        _ => assert(false),
    }
}

#[inline(never)]
fn in_enums_not_inlined() {
    in_enums()
}

#[inline(always)]
fn in_arrays() {
    let mut x = 11u8;
    let mut array: [u64; 3] = [11, 11, 11];

    let a = A {
        r_u8: &mut x,
        r_array: &mut array,
    };
    let b = B {
        r_a: &a,
        r_array: &[&a, &a, &a],
    };

    let arr_a = [&a, &a, &a];
    let arr_b = [&b, &b, &b];

    *arr_a[0].r_u8 = 22;
    assert_eq(x, 22);

    *arr_a[1].r_array = [22, 22, 22];
    assert_eq(array, [22, 22, 22]);

    *arr_b[0].r_a.r_u8 = 33;
    assert_eq(x, 33);

    *arr_b[1].r_array[0].r_u8 = 44;
    assert_eq(x, 44);

    *arr_b[2].r_array[0].r_array = [33, 33, 33];
    assert_eq(array, [33, 33, 33]);
}

#[inline(never)]
fn in_arrays_not_inlined() {
    in_arrays()
}

#[inline(always)]
fn in_tuples() {
    let mut x = 11u8;
    let mut array: [u64; 3] = [11, 11, 11];

    let a = A {
        r_u8: &mut x,
        r_array: &mut array,
    };
    let b = B {
        r_a: &a,
        r_array: &[&a, &a, &a],
    };

    let tuple_a = (&a, &a, &a);
    let tuple_b = (&b, &b, &b);

    *tuple_a.0.r_u8 = 22;
    assert_eq(x, 22);

    *tuple_a.1.r_array = [22, 22, 22];
    assert_eq(array, [22, 22, 22]);

    *tuple_b.0.r_a.r_u8 = 33;
    assert_eq(x, 33);

    *tuple_b.1.r_array[0].r_u8 = 44;
    assert_eq(x, 44);

    *tuple_b.2.r_array[0].r_array = [33, 33, 33];
    assert_eq(array, [33, 33, 33]);
}

#[inline(never)]
fn in_tuples_not_inlined() {
    in_tuples()
}

#[inline(never)]
fn test_all_inlined() {
    in_structs();
    in_enums();
    in_arrays();
    in_tuples();
}

#[inline(never)]
fn test_not_inlined() {
    in_structs_not_inlined();
    in_enums_not_inlined();
    in_arrays_not_inlined();
    in_tuples_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    A::new().use_me();
    B::new().use_me();

    42
}

fn poke<T>(_x: T) {}
