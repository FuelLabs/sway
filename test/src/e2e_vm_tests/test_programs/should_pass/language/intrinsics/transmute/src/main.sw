script;

fn main() -> u64 {
    let a = __transmute::<[u8; 8], u64>([0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8]);
    let b = __transmute::<(u64,), u64>((2,));
    a + b
}
