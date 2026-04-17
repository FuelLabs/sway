library;

pub fn check_args() {
    let _ = __add();
    let _ = __add(42u64);
    let _ = __add((), 42u64);
    let _ = __add(42u64, 1u32);
    let _ = __add::<u64>(42u64, 1u64);
    let _ = __add::<u32>(42, 1);
}
