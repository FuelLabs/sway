script;

struct A<T> {
    a: T,
}

impl<T> A<T> {
    #[inline(never)]
    fn getter<G>(self, b: G) -> T {
        self.a
    }
}

fn func<T>(u: T) -> u64 {
    42
}

fn main() {
    let s = A { a: 1u64 };
    s.getter(1u32);
    s.getter(2u32);

    func(1u32);
    func(2u32);
}

// ::check-ir::
// regex: NUM=[0-9]+
// check: fn getter_$NUM(self !$NUM: { u64 }, b !$NUM: u64) -> u64, !$NUM {
// not: fn getter_$NUM(self !$NUM: { u64 }, b !$NUM: u64) -> u64, !$NUM {

// check: fn func_$NUM(u !$NUM: u64) -> u64, !$NUM {
// not: fn func_$NUM(u !$NUM: u64) -> u64, !$NUM {

