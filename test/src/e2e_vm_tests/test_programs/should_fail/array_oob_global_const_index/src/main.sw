script;

const i: u64 = 4;

fn main() -> u64 {
    // index out of bounds: the length is 3 but the index is 4
    let ary = [1, 2, 3];
    ary[i]
}
