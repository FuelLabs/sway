library;

pub struct Data {
    value: u64
}

pub fn gimme_a_pair() -> (u32, u64) {
    (1u32, 2u64)
}

pub fn gimme_one(x: u64) -> (u64, u64) {
    (x, 1u64)
}

pub fn test<T, E>(a: T, b: E) {
    let (x, y): (T, E) = (a, b);
} 
