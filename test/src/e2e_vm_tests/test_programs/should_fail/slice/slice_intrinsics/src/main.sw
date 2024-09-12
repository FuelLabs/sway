script;

// slice cannot be consts
const GLOBAL_ARRAY: [u64; 5] = [1, 2, 3, 4, 5];
const GLOBAL_SLICE: &__slice[u64] = __slice(&GLOBAL_ARRAY, 0, 5);

fn main()  {
    type_check();

    // slice cannot be consts
    const LOCAL_ARRAY: [u64; 5] = [1, 2, 3, 4, 5];
    const LOCAL_SLICE: &__slice[u64] = __slice(&LOCAL_ARRAY, 0, 5);

    // Wrong start index
    let a: [u64; 5] = [1, 2, 3, 4, 5];
    let _ = __slice(&a, 6, 7);

    // Wrong end index
    let a: [u64; 5] = [1, 2, 3, 4, 5];
    let _ = __slice(&a, 0, 6);

    // Wrong first argument
    __slice(0, 0, 0);

    // Wrong start index
    __slice(&a, "", 0);

    // Wrong end index
    __slice(&a, 0, "");

    let a: [u64; 5] = [1, 2, 3, 4, 5];
    let s: &__slice[u64] = __slice(&LOCAL_ARRAY, 0, 5);

    // Wrong first argument
    __elem_at(0, 0);

    // Wrong index type
    __elem_at(&a, "");

    // Wrong index type
    __elem_at(s, "");
}

fn type_check() {
    // Cannot get mut ref from an immutable array
    let immutable_array: [u64; 5] = [1, 2, 3, 4, 5];
    let _: &mut u64 = __elem_at(&immutable_array, 0);

    // Cannot get mut slice from an immutable array
    let _: &mut __slice[u64] = __slice(&immutable_array, 0, 5);

    // Cannot get mut ref from an immutable slice
    let immutable_slice: &__slice[u64] = __slice(&immutable_array, 0, 5);
    let _: &mut u64 = __elem_at(immutable_slice, 0);
}
