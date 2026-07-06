library;

pub fn check_args() {
    let _ = __dbg();
    let _ = __dbg(42u64, 1u32);
    // TODO: This actually doesn't work. There is no error emitted.
    //       Fix this together with type checking of intrinsics in general:
    //          https://github.com/FuelLabs/sway/issues/7596
    let _ = __dbg::<u64>(42u64);
}
