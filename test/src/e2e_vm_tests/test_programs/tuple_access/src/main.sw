script;

fn gimme_a_pair() -> (u32, u64) {
    (1u32, 2u64)
}

fn main() -> u32 {
    let (a,b) = gimme_a_pair();
    a
}
