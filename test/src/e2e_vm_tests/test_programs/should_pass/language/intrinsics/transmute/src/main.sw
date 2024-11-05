script;

fn main() -> u64 {
    __transmute::<[u8; 8], u64>([0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8])
}
