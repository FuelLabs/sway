script;

fn main() -> u64 {
    let a = 1u8;
    __transmute::<[u8;4], u64>([0u8, 0u8, 0u8, a])
}
