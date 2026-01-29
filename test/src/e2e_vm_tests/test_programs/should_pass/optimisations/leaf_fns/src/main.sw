script;

#[inline(never)]
fn leaf_fn_0() {
}

#[inline(never)]
fn leaf_fn_6(a: u64, b: u64, c: u64, d: u64, e: u64, f: u64) -> u64 {
    a + b + c + d + e + f
}

#[inline(never)]
fn leaf_fn_7(a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64) -> u64 {
    a + b + c + d + e + f + g
}

fn main() {
    leaf_fn_0();
    leaf_fn_6(0, 1, 2, 3, 4, 5);
    leaf_fn_7(0, 1, 2, 3, 4, 5, 6);
}
