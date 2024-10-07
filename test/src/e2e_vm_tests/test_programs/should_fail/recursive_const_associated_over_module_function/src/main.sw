library;

struct S {}

fn mod_fn() -> u8 {
    S::S_ASSOC
}

impl S {
    const S_ASSOC: u8 = mod_fn();
}