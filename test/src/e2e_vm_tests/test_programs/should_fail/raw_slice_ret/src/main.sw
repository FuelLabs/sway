script;

struct HasRawSlice {
    slice: raw_slice,
}

fn main() -> HasRawSlice {
    let val = HasRawSlice {
        slice: asm() { zero: raw_slice }
    };
    val
}
