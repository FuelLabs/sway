script;

struct S {
    x: u8,
}

impl S {
    fn new() -> Self {
        Self { x: 0 }
    }

    fn use_me(self) {
        poke(self.x);
    }
}

impl PartialEq for S {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}
impl Eq for S {}

// TODO: (REFERENCES) Extend with `mut` parameters once declaring `mut` parameters is implemented.
// TODO: (REFERENCES) Extend with `&` and `&mut` parameters once proper referencing of copy type parameters is implemented.
#[inline(always)]
fn u8_parameter(p: u8) {
   let r_p_1 = &p;
   let r_p_2 = &p;

   let p_ptr = asm(r: &p) { r: raw_ptr };
   let r_p_1_ptr = asm(r: r_p_1) { r: raw_ptr };
   let r_p_2_ptr = asm(r: r_p_2) { r: raw_ptr };

   assert(p_ptr == r_p_1_ptr);
   assert(p_ptr == r_p_2_ptr);

   assert(p_ptr.read::<u8>() == p);
}

#[inline(never)]
fn u8_parameter_not_inlined(p: u8) {
   u8_parameter(p)
}

#[inline(always)]
fn array_parameter(p: [u64; 2]) {
    let r_p_1 = &p;
    let r_p_2 = &p;

    let p_ptr = asm(r: &p) {
        r: raw_ptr
    };
    let r_p_1_ptr = asm(r: r_p_1) {
        r: raw_ptr
    };
    let r_p_2_ptr = asm(r: r_p_2) {
        r: raw_ptr
    };

    assert(p_ptr == r_p_1_ptr);
    assert(p_ptr == r_p_2_ptr);

    assert(p_ptr.read::<[u64; 2]>() == p);

    assert(*r_p_1 == *r_p_2);

    assert(r_p_1[0] == r_p_2[0]);
    assert(r_p_1[1] == r_p_2[1]);
}

#[inline(never)]
fn array_parameter_not_inlined(p: [u64; 2]) {
    array_parameter(p)
}

struct EmptyStruct {}

impl PartialEq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}
impl Eq for EmptyStruct {}

#[inline(always)]
fn empty_struct_parameter(p: EmptyStruct) {
    let r_p_1 = &p;
    let r_p_2 = &p;

    let p_ptr = asm(r: &p) {
        r: raw_ptr
    };
    let r_p_1_ptr = asm(r: r_p_1) {
        r: raw_ptr
    };
    let r_p_2_ptr = asm(r: r_p_2) {
        r: raw_ptr
    };

    assert(p_ptr == r_p_1_ptr);
    assert(p_ptr == r_p_2_ptr);

    assert(p_ptr.read::<EmptyStruct>() == p);

    assert(*r_p_1 == *r_p_2);
}

#[inline(never)]
fn empty_struct_parameter_not_inlined(p: EmptyStruct) {
    empty_struct_parameter(p)
}

#[inline(always)]
fn struct_parameter(p: S) {
    let r_p_1_addr_of = __addr_of(p);
    assert(r_p_1_addr_of == __addr_of(p));

    let r_p_1 = &p;
    let r_p_2 = &p;

    let p_ptr = asm(r: &p) {
        r: raw_ptr
    };
    let r_p_1_ptr = asm(r: r_p_1) {
        r: raw_ptr
    };
    let r_p_2_ptr = asm(r: r_p_2) {
        r: raw_ptr
    };

    assert(p_ptr == r_p_1_ptr);
    assert(p_ptr == r_p_2_ptr);

    assert(p_ptr.read::<S>() == p);

    assert(*r_p_1 == *r_p_2);

    assert(r_p_1.x == r_p_2.x);

    let q = S::new();
    assert(r_p_1_addr_of != __addr_of(q));
}

#[inline(never)]
fn struct_parameter_not_inlined(p: S) {
    struct_parameter(p)
}

#[inline(always)]
fn tuple_parameter(p: (u64, u64)) {
    let r_p_1 = &p;
    let r_p_2 = &p;

    let p_ptr = asm(r: &p) {
        r: raw_ptr
    };
    let r_p_1_ptr = asm(r: r_p_1) {
        r: raw_ptr
    };
    let r_p_2_ptr = asm(r: r_p_2) {
        r: raw_ptr
    };

    assert(p_ptr == r_p_1_ptr);
    assert(p_ptr == r_p_2_ptr);

    assert(p_ptr.read::<(u64, u64)>() == p);

    assert(*r_p_1 == *r_p_2);

    assert(r_p_1.0 == r_p_2.0);
    assert(r_p_1.1 == r_p_2.1);
}

#[inline(never)]
fn tuple_parameter_not_inlined(p: (u64, u64)) {
    tuple_parameter(p)
}

enum E {
    A: u8,
}

impl PartialEq for E {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (E::A(r), E::A(l)) => r == l,
        }
    }
}
impl Eq for E {}

#[inline(always)]
fn enum_parameter(p: E) {
    let r_p_1 = &p;
    let r_p_2 = &p;

    let p_ptr = asm(r: &p) {
        r: raw_ptr
    };
    let r_p_1_ptr = asm(r: r_p_1) {
        r: raw_ptr
    };
    let r_p_2_ptr = asm(r: r_p_2) {
        r: raw_ptr
    };

    assert(p_ptr == r_p_1_ptr);
    assert(p_ptr == r_p_2_ptr);

    assert(p_ptr.read::<E>() == p);

    assert(*r_p_1 == *r_p_2);
}

#[inline(never)]
fn enum_parameter_not_inlined(p: E) {
    enum_parameter(p)
}

#[inline(always)]
fn generic_parameter() {
    generic_parameter_test(123u8);
    generic_parameter_test(123u64);
    generic_parameter_test(true);

    let s = S { x: 0 };
    let ptr_s = __addr_of(s);

    generic_parameter_test(ptr_s);

    generic_parameter_test(S { x: 123u8 });
    generic_parameter_test(EmptyStruct {});
    generic_parameter_test([123u64, 123u64]);
    generic_parameter_test(E::A(123u8));
}

#[inline(always)]
fn generic_parameter_test<T>(p: T)
where
    T: Eq,
{
    let r_p_1 = &p;
    let r_p_2 = &p;

    let p_ptr = asm(r: &p) {
        r: raw_ptr
    };
    let r_p_1_ptr = asm(r: r_p_1) {
        r: raw_ptr
    };
    let r_p_2_ptr = asm(r: r_p_2) {
        r: raw_ptr
    };

    assert(p_ptr == r_p_1_ptr);
    assert(p_ptr == r_p_2_ptr);

    assert(p_ptr.read::<T>() == p);

    assert(*r_p_1 == *r_p_2);
}

#[inline(never)]
fn generic_parameter_not_inlined() {
    generic_parameter()
}

#[inline(never)]
fn test_all_inlined() {
    u8_parameter(123u8);
    array_parameter([111u64, 222u64]);
    empty_struct_parameter(EmptyStruct {});
    struct_parameter(S { x: 123u8 });
    tuple_parameter((111u64, 222u64));
    enum_parameter(E::A(123u8));
    generic_parameter();
}

#[inline(never)]
fn test_not_inlined() {
    u8_parameter_not_inlined(123u8);
    array_parameter_not_inlined([111u64, 222u64]);
    empty_struct_parameter_not_inlined(EmptyStruct {});
    struct_parameter_not_inlined(S { x: 123u8 });
    tuple_parameter_not_inlined((111u64, 222u64));
    enum_parameter_not_inlined(E::A(123u8));
    generic_parameter_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    S::new().use_me();

    42
}

#[inline(never)]
fn poke<T>(_x: T) {}
