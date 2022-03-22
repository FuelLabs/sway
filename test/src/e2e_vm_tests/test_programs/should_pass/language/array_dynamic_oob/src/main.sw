script;

fn main() -> u32 {
    let ary = [1, 2, 3, 4];
    let idx = 45;
    // thread 'main' panicked at 'index out of bounds: the len is 3 but the index is 45'
    ary[idx]
}
