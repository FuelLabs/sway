library;

pub fn check_args() {
    // Missing type arguments
    let _ = __transmute(1u64);
    let _ = __transmute::<u64>(1u64);

     // Wrong source type
    let _ = __transmute::<u64, u8>(1u32);

    // TODO: This actually doesn't work. There is no error emitted.
    //       Fix this together with type checking of intrinsics in general:
    //          https://github.com/FuelLabs/sway/issues/7596
    // Different sizes
    let _ = __transmute::<u64, u8>(1u64);
}
