script;

// slice cannot be consts
const GLOBAL_ARRAY: [u64; 5] = [1, 2, 3, 4, 5];
const GLOBAL_SLICE: &__slice[u64] = __slice(GLOBAL_ARRAY, 0, 5);

fn main()  {
    // slice cannot be consts
    const LOCAL_ARRAY: [u64; 5] = [1, 2, 3, 4, 5];
    const LOCAL_SLICE: &__slice[u64] = __slice(LOCAL_ARRAY, 0, 5);

    // Wrong start index
    let a: [u64; 5] = [1, 2, 3, 4, 5];
    let _ = __slice(a, 6, 7);

    // Wrong end index
    let a: [u64; 5] = [1, 2, 3, 4, 5];
    let _ = __slice(a, 0, 6);

    // Wrong first argument
    __slice(0, 0, 0);

    // Wrong start index
    __slice(0, "", 0);

    // Wrong end index
    __slice(0, 0, "");
}
