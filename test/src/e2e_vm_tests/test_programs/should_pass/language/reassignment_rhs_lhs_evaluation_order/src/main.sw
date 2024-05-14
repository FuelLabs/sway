script;

// In Sway, like in Rust, we first evaluate the RHS.

fn inc_i(ref mut i: u64) -> u64 {
    i += 1;
    i
}

fn main() -> u64 {
    let mut array = [0, 0, 0];
    let mut i = 0;

    array[inc_i(i)] = inc_i(i);

    assert_eq(array[0], 0);
    assert_eq(array[1], 0);
    assert_eq(array[2], 1);

    1
}
