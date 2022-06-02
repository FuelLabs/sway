script;

use std::assert::assert;

struct S { }

impl<T> S {
  fn f(self, value: T) {
    __size_of::<T>();
  }
}

struct Data<T, F> {
    x: T,
    y: F
}

impl<T> Data<T, T> {
    fn get_x(self) -> T {
        self.x
    }
}

impl<T, F> Data<T, F> {
    fn get_y(self) -> F {
        self.y
    }
}

fn main() {
    let a = (S{}).f(true);
    assert(a == 1);

    let b = Data {
        x: true,
        y: false
    };
    let c = b.get_x();
    assert(c == true);

    let d = Data {
        x: true,
        y: 7u64
    };
    let e = d.get_y();
    assert(e == 7u64);

    // should fail
    let f = d.get_x();
    assert(f == true);
}
