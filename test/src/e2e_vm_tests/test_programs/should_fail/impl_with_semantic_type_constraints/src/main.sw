library;

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
        0
    }
}

pub fn main() {
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

    let _d = a.get_first();
    let _e = a.get_second();
    let _f = a.add();

    let _g = b.get_first();
    let _h = b.get_second();
    let _i = b.add(); // should fail

    let _j = c.get_first(); // should fail
    let _k = c.get_second();
    let _l = c.add(); // should fail
}
