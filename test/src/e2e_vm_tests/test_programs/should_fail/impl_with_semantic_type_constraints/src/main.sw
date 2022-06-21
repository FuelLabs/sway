script;

struct DoubleIdentity<T, F> {
    first: T,
    second: F,
}

impl<T> DoubleIdentity<T, T> {
    fn get_first(self) -> T {
        self.first
    }
}

impl<T, F> DoubleIdentity<T, F> {
    fn get_second(self) -> F {
        self.second
    }
}

impl DoubleIdentity<u8, u8> {
    fn add(self) -> u8 {
        self.first + self.second
    }
}

fn main() {
    let a = DoubleIdentity {
        first: 0u8,
        second: 1u8
    };
    let b = DoubleIdentity {
        first: true,
        second: false,
    };
    let c = DoubleIdentity {
        first: 0u64,
        second: "hi"
    };

    let d = a.get_first();
    let e = a.get_second();
    let f = a.add();

    let g = b.get_first();
    let h = b.get_second();
    // should fail
    let i = b.add();

    // should fail
    let j = c.get_first();
    let k = c.get_second();
    // should fail
    let l = c.add();
}
