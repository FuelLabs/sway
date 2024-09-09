script;

struct S {}

impl S {
    fn method(self, a: u64) -> u64 {
        a
    }
}

fn main() {
    let _ = S {}.method(non_existing);
}