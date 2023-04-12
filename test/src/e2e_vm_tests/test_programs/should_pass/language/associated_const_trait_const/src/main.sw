script;

trait T {
    const C: bool;
}

struct S {}

impl T for S {
    const C: bool = true;
}

fn main() -> bool {
    S::C
}
