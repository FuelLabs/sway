script;

use core::ops::Eq;

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

impl Eq for S {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}

#[inline(always)]
fn reference_to_copy_type() {
    let x = 123u8;
    let r_x = &x;
    
    let ptr = asm(r: &x) { r: raw_ptr };

    let r_ret = copy_type_ref(&x, x, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);

    let r_ret = copy_type_ref(r_x, x, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
}

#[inline(never)]
fn reference_to_copy_type_not_inlined() {
    reference_to_copy_type()
}

fn copy_type_ref(r: &u8, v: u8, ptr: raw_ptr) -> &u8 {
    let r_ptr = asm(r: r) { r: raw_ptr };

    assert(r_ptr == ptr);
    assert(r_ptr.read::<u8>() == v);

    r
}

#[inline(always)]
fn reference_to_aggregate() {
    let s = S { x: 123u8 };
    let r_s = &s;
    
    let ptr = asm(r: &s) { r: raw_ptr };
    assert(ptr == __addr_of(s));

    let r_ret = aggregate_ref(&s, s, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);

    let r_ret = aggregate_ref(r_s, s, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
}

#[inline(never)]
fn reference_to_aggregate_not_inlined() {
    reference_to_aggregate()
}

fn aggregate_ref(r: &S, v: S, ptr: raw_ptr) -> &S {
    let r_ptr = asm(r: r) { r: raw_ptr };

    assert(r_ptr == ptr);
    assert(r_ptr.read::<S>() == v);

    r
}

impl Eq for [u64;2] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}

struct EmptyStruct { }

impl Eq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}

enum E {
    A: u8,
}

impl Eq for E {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (E::A(r), E::A(l)) => r == l,
        }
    }
}

#[inline(always)]
fn reference_to_generic() {
    // TODO-IG: Uncomment once referencing copy type function parameters is implemented.
    //reference_to_generic_test(123u8);
    //reference_to_generic_test(123u64);
    //reference_to_generic_test(true);

    //let s = S { x: 0 };
    //let ptr_s = __addr_of(s);

    //reference_to_generic_test(ptr_s);

    reference_to_generic_test(S { x: 123u8 });
    reference_to_generic_test(EmptyStruct { });
    reference_to_generic_test([123u64, 123u64]);
    reference_to_generic_test(E::A(123u8));
}

#[inline(always)]
fn reference_to_generic_test<T>(t: T)
    where T: Eq
{
    let r_t = &t;
    
    let ptr = asm(r: &t) { r: raw_ptr };

    let r_ret = generic_ref(&t, t, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);

    let r_ret = generic_ref(r_t, t, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
}

#[inline(never)]
fn reference_to_generic_not_inlined() {
    reference_to_generic()
}

fn generic_ref<T>(r: &T, v: T, ptr: raw_ptr) -> &T
    where T: Eq
{
    let r_ptr = asm(r: r) { r: raw_ptr };

    assert(r_ptr == ptr);
    assert(r_ptr.read::<T>() == v);

    r
}

#[inline(never)]
fn test_all_inlined() {
    reference_to_copy_type();
    reference_to_aggregate();
    reference_to_generic();
}

#[inline(never)]
fn test_not_inlined() {
    reference_to_copy_type_not_inlined();
    reference_to_aggregate_not_inlined();
    reference_to_generic_not_inlined();
}

fn main() -> u64 {
    test_all_inlined();
    test_not_inlined();

    S::new().use_me();

    42
}

#[inline(never)]
fn poke<T>(_x: T) { }