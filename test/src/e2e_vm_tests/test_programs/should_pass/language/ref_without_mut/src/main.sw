script;

struct S {
    value: u64,
}

impl S {
    fn method(self, ref _x: u64) -> u64 {
        _x
    }

    fn associated_fn(ref _x: u64) -> u64 {
        _x
    }
}

fn ref_pass(ref _x: u64) -> u64 {
    _x
}

fn main() -> u64 {
    let x = 42;

    let a = ref_pass(x);
    let b = S { value: 1 }.method(x);
    let c = S::associated_fn(x);

    if a == 42 && b == 42 && c == 42 {
        42
    } else {
        0
    }
}
