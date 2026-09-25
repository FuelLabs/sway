script;

fn ref_immutable(ref x: u64) {
    x = 10;
}

fn ref_mut_immutable_arg(ref mut _x: u64) { }

fn main() -> u64 {
    let y = 42;

    ref_immutable(y);

    ref_mut_immutable_arg(y);

    y
}
