script;

mod impls;
use impls::*;

#[inline(always)]
fn reference_to_copy_type() {
    let mut x = 123u8;
    let r_x = &x;
    let r_mut_x = &mut x;
    
    let ptr = asm(r: &x) { r: raw_ptr };

    let (r_ret, r_mut_ret) = copy_type_ref(&x, &mut x, x, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    let r_mut_ret_ptr = asm(r: r_mut_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
    assert(r_mut_ret_ptr == ptr);

    assert(*r_ret == *r_x);
    assert(*r_mut_ret == *r_x);

    let (r_ret, r_mut_ret) = copy_type_ref(r_x, r_mut_x, x, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    let r_mut_ret_ptr = asm(r: r_mut_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
    assert(r_mut_ret_ptr == ptr);

    assert(*r_ret == *r_x);
    assert(*r_mut_ret == *r_x);
}

#[inline(never)]
fn reference_to_copy_type_not_inlined() {
    reference_to_copy_type()
}

fn copy_type_ref(r: &u8, r_mut: &mut u8, v: u8, ptr: raw_ptr) -> (&u8, &mut u8) {
    let r_ptr = asm(r: r) { r: raw_ptr };
    let r_mut_ptr = asm(r: r_mut) { r: raw_ptr };

    assert(r_ptr == ptr);
    assert(r_mut_ptr == ptr);
    assert(r_ptr.read::<u8>() == v);
    assert(r_mut_ptr.read::<u8>() == v);

    assert(*r == v);
    assert(*r_mut == v);

    (r, r_mut)
}

#[inline(always)]
fn reference_to_aggregate() {
    let mut s = Struct { x: 123 };
    let r_s = &s;
    let r_mut_s = &mut s;
    
    let ptr = asm(r: &s) { r: raw_ptr };
    assert(ptr == __addr_of(s));

    let (r_ret, r_mut_ret) = aggregate_ref(&s, &mut s, s, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    let r_mut_ret_ptr = asm(r: r_mut_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
    assert(r_mut_ret_ptr == ptr);

    assert(*r_ret == *r_s);
    assert(*r_mut_ret == *r_s);

    let (r_ret, r_mut_ret) = aggregate_ref(r_s, r_mut_s, s, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    let r_mut_ret_ptr = asm(r: r_mut_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
    assert(r_mut_ret_ptr == ptr);

    assert(*r_ret == *r_s);
    assert(*r_mut_ret == *r_s);
}

#[inline(never)]
fn reference_to_aggregate_not_inlined() {
    reference_to_aggregate()
}

fn aggregate_ref(r: &Struct, r_mut: &mut Struct, v: Struct, ptr: raw_ptr) -> (&Struct, &mut Struct) {
    let r_ptr = asm(r: r) { r: raw_ptr };
    let r_mut_ptr = asm(r: r_mut) { r: raw_ptr };

    assert(r_ptr == ptr);
    assert(r_mut_ptr == ptr);
    assert(r_ptr.read::<Struct>() == v);
    assert(r_mut_ptr.read::<Struct>() == v);

    assert(*r == v);
    assert(*r_mut == v);

    (r, r_mut)
}

#[inline(always)]
fn reference_to_generic() {
    reference_to_generic_test::<()>();
    reference_to_generic_test::<bool>();
    reference_to_generic_test::<u8>();
    reference_to_generic_test::<u16>();
    reference_to_generic_test::<u32>();
    reference_to_generic_test::<u64>();
    reference_to_generic_test::<u256>();
    reference_to_generic_test::<[u64;2]>();
    reference_to_generic_test::<[u64;0]>();
    reference_to_generic_test::<Struct>();
    reference_to_generic_test::<EmptyStruct>();
    reference_to_generic_test::<str>();
    reference_to_generic_test::<str[6]>();
    reference_to_generic_test::<Enum>();
    reference_to_generic_test::<(u8, u32)>();
    reference_to_generic_test::<b256>();
    reference_to_generic_test::<raw_ptr>();
    reference_to_generic_test::<raw_slice>();
}

#[inline(always)]
fn reference_to_generic_test<T>()
    where T: TestInstance + Eq
{
    let mut t = T::new();
    let r_t = &t;
    let r_mut_t = &mut t;
    
    let ptr = asm(r: &t) { r: raw_ptr };

    let (r_ret, r_mut_ret) = generic_ref(&t, &mut t, t, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    let r_mut_ret_ptr = asm(r: r_mut_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
    assert(r_mut_ret_ptr == ptr);

    assert(*r_ret == *r_t);
    assert(*r_mut_ret == *r_t);

    let (r_ret, r_mut_ret) = generic_ref(r_t, r_mut_t, t, ptr);
    let r_ret_ptr = asm(r: r_ret) { r: raw_ptr };
    let r_mut_ret_ptr = asm(r: r_mut_ret) { r: raw_ptr };
    assert(r_ret_ptr == ptr);
    assert(r_mut_ret_ptr == ptr);

    assert(*r_ret == *r_t);
    assert(*r_mut_ret == *r_t);
}

#[inline(never)]
fn reference_to_generic_not_inlined() {
    reference_to_generic()
}

fn generic_ref<T>(r: &T, r_mut: &mut T, v: T, ptr: raw_ptr) -> (&T, &mut T)
    where T: Eq
{
    let r_ptr = asm(r: r) { r: raw_ptr };
    let r_mut_ptr = asm(r: r_mut) { r: raw_ptr };

    assert(r_ptr == ptr);
    assert(r_mut_ptr == ptr);
    assert(r_ptr.read::<T>() == v);
    assert(r_mut_ptr.read::<T>() == v);

    assert(*r == v);
    assert(*r_mut == v);

    (r, r_mut)
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

    42
}