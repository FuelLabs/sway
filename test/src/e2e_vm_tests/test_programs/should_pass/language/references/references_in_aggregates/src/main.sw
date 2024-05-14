script;

struct A {
    r_u8: &u8,
    r_array: &[u64;3],
}

impl A {
    fn new() -> Self {
        Self { r_u8: &0, r_array: &[0, 0, 0] }
    }
    
    fn use_me(self) {
        poke(self.r_u8);
        poke(self.r_array);
    }
}

struct B {
    r_a: &A,
    r_array: &[&A;3],
}

impl B {
    fn new() -> Self {
        let r_a = &A::new();
        Self { r_a: r_a, r_array: &[r_a, r_a, r_a] }
    }
    
    fn use_me(self) {
        poke(self.r_a);
        poke(self.r_array);
    }
}

#[inline(always)]
fn in_structs() {
    assert(__size_of::<A>() == 2 * 8);
    assert(__size_of::<B>() == 2 * 8);

    let x = 123u8;
    let array: [u64;3] = [111, 222, 333];

    let a = A { r_u8: &x, r_array: &array };
    let b = B { r_a: &a, r_array: &[&a, &a, &a] };

    let r_x_ptr = asm(r: &x) { r: raw_ptr };
    let r_array_ptr = asm(r: &array) { r: raw_ptr };

    let r_a_ptr = asm(r: &a) { r: raw_ptr };

    let a_r_u8_ptr = asm(r: a.r_u8) { r: raw_ptr };
    let a_r_array_ptr = asm(r: a.r_array) { r: raw_ptr };

    assert(r_x_ptr == a_r_u8_ptr);
    assert(r_array_ptr == a_r_array_ptr);

    assert(*&x == *a.r_u8);

    let mut i = 0;
    while i < 3 {
        assert((*&array)[i] == (*a.r_array)[i]);
        assert((&array)[i] == a.r_array[i]);
        i += 1;
    }

    let b_r_a_ptr = asm(r: b.r_a) { r: raw_ptr };

    assert(r_a_ptr == b_r_a_ptr);

    let a_r_u8_ptr_val = a_r_u8_ptr.read::<u8>();
    let a_r_array_ptr_val = a_r_array_ptr.read::<[u64;3]>();

    assert(a_r_u8_ptr_val == x);

    let mut i = 0;
    while i < 3 {
        assert(a_r_array_ptr_val[1] == array[1]);
        i += 1;
    }

    let b_r_a_ptr_val = b_r_a_ptr.read::<A>();
    let a_r_u8_ptr_over_b = asm(r: b_r_a_ptr_val.r_u8) { r: raw_ptr };
    
    assert(a_r_u8_ptr_over_b == a_r_u8_ptr);

    assert(*(*b.r_a).r_u8 == *a.r_u8);
    assert(*b.r_a.r_u8 == *a.r_u8);

    let mut i = 0;
    while i < 3 {
        assert((*((*(*b.r_array)[0]).r_array))[i] == (*a.r_array)[i]);
        assert(((*(b.r_array)[0]).r_array)[i] == a.r_array[i]);
        i += 1;
    }
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
    assert(__size_of::<E>() == 2 * 8);

    let x = 123u8;
    let array: [u64;3] = [111, 222, 333];

    let a = A { r_u8: &x, r_array: &array };
    let b = B { r_a: &a, r_array: &[&a, &a, &a] };

    let r_a_ptr = asm(r: &a) { r: raw_ptr };
    let r_b_ptr = asm(r: &b) { r: raw_ptr };

    let e_r_a = E::R_A(&a);
    let e_r_b = E::R_B(&b);

    match e_r_a {
        E::R_A(r_a) => {
            let local_r_a: &A = r_a; // To proof `r_a` is of type `&A`.

            let local_r_a_ptr = asm(r: local_r_a) { r: raw_ptr };

            assert(local_r_a_ptr == r_a_ptr);

            assert(*(*local_r_a).r_u8 == *&x);
            assert(*local_r_a.r_u8 == *&x);

            let mut i = 0;
            while i < 3 {
                assert((*(*local_r_a).r_array)[i] == (*&array)[i]);
                assert((*local_r_a).r_array[i] == (&array)[i]);
                assert(local_r_a.r_array[i] == (& &array)[i]);
                i += 1;
            }
        }
        _ => assert(false),
    }

    match e_r_b {
        E::R_B(r_b) => {
            let local_r_b: &B = r_b; // To proof `r_b` is of type `&B`.

            let local_r_b_ptr = asm(r: local_r_b) { r: raw_ptr };

            assert(local_r_b_ptr == r_b_ptr);

            assert(*(*(*local_r_b).r_a).r_u8 == *&x);
            assert(*local_r_b.r_a.r_u8 == *&x);

            let mut i = 0;
            while i < 3 {
                assert((*(*(*(*local_r_b).r_array)[0]).r_array)[i] == (*&array)[i]);
                assert((*(*local_r_b).r_array[0]).r_array[i] == (&array)[i]);
                assert(local_r_b.r_array[0].r_array[i] == (& &array)[i]);
                i += 1;
            }
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
    let x = 123u8;
    let array: [u64;3] = [111, 222, 333];

    let a = A { r_u8: &x, r_array: &array };
    let b = B { r_a: &a, r_array: &[&a, &a, &a] };

    let r_b_ptr = asm(r: &b) { r: raw_ptr };

    let arr = [&b, &b, &b];

    assert(__size_of_val(arr) == 3 * 8);

    let r_arr_0_ptr = asm(r: arr[0]) { r: raw_ptr };
    let r_arr_1_ptr = asm(r: arr[1]) { r: raw_ptr };
    let r_arr_2_ptr = asm(r: arr[2]) { r: raw_ptr };

    assert(r_b_ptr == r_arr_0_ptr);
    assert(r_b_ptr == r_arr_1_ptr);
    assert(r_b_ptr == r_arr_2_ptr);

    let mut i = 0;
    while i < 3 {
        assert((*(*((*(*arr[1]).r_array)[2])).r_array)[i] == (*&array)[i]);
        assert((*((*arr[1]).r_array[2])).r_array[i] == (&array)[i]);
        assert(arr[1].r_array[2].r_array[i] == (& &array)[i]);
        i += 1;
    }
}

#[inline(never)]
fn in_arrays_not_inlined() {
    in_arrays()
}

#[inline(always)]
fn in_tuples() {
    let x = 123u8;
    let array: [u64;3] = [111, 222, 333];

    let a = A { r_u8: &x, r_array: &array };
    let b = B { r_a: &a, r_array: &[&a, &a, &a] };

    let r_b_ptr = asm(r: &b) { r: raw_ptr };

    let tuple = (&b, &b, &b);
    
    assert(__size_of_val(tuple) == 3 * 8);

    let r_tuple_0_ptr = asm(r: tuple.0) { r: raw_ptr };
    let r_tuple_1_ptr = asm(r: tuple.1) { r: raw_ptr };
    let r_tuple_2_ptr = asm(r: tuple.2) { r: raw_ptr };

    assert(r_b_ptr == r_tuple_0_ptr);
    assert(r_b_ptr == r_tuple_1_ptr);
    assert(r_b_ptr == r_tuple_2_ptr);

    let mut i = 0;
    while i < 3 {
        assert((*(*((*(*tuple.1).r_array)[2])).r_array)[i] == (*&array)[i]);
        assert((*((*tuple.1).r_array[2])).r_array[i] == (&array)[i]);
        assert(tuple.1.r_array[2].r_array[i] == (& &array)[i]);
        i += 1;
    }
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

fn poke<T>(_x: T) { }