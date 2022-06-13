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
    let b = a.get_first();
    let c = a.get_second();
    let d = a.add();

    let e = DoubleIdentity {
        first: true,
        second: "hi"
    };
    let f = e.get_second();
}
