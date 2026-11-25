library;

pub trait T {
    fn tf(self) -> u64;
}

pub struct S {
    pub x: u64,
}

impl T for S {
    fn tf(self) -> u64 {
        1
    }
}

// The duplicated trait impl fails the construction of the impl declaration,
// and leaves its reserved decl slot open in the decl engine shared with
// the member package.
impl T for S {
    fn tf(self) -> u64 {
        2
    }
}

pub fn make() -> u64 {
    42
}
