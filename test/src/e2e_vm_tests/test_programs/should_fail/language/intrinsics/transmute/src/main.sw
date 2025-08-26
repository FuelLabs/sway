script;

fn main() {
    // Missing type arguments
    let _ = __transmute(1u64);
    let _ = __transmute::<u64>(1u64);

     // Wrong source type
    let _ = __transmute::<u64, u8>(1u32);

    // Different sizes
    let _ = __transmute::<u64, u8>(1u64);
}
