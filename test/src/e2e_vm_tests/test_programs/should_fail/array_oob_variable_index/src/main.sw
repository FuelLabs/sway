// TODO: Enable this test once is https://github.com/FuelLabs/sway/issues/7521 fixed.
library;

fn variable_index() -> u64 {
    let ary = [1, 2, 3];
    let i = 4;
    ary[i]
}

#[test]
fn test() {
    poke(variable_index());
}

#[inline(never)]
fn poke<T>(_x: T) { }
