script;

pub trait MyEq {
    fn my_eq(self, other: Self);
}

impl MyEq for u64 {
    fn my_eq(self, other: Self) {
    }
}

fn test_my_eq<T>(x: T, y: T) where T: MyEq {
  x.my_eq(y)
}

fn main() {
  test_my_eq(42, 42);
}
