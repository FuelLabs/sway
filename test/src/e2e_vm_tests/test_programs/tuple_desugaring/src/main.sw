script;

fn gimme_a_pair() -> (u32, u64) {
    (1u32, 2u64)
}

fn main() -> u32 {
    let x = gimme_a_pair();
    match x {
        (a, 2u64) => { a },
        (a, b) => { 0u32 },
    }
}
