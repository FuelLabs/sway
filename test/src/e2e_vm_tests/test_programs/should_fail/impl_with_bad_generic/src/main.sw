library;

struct S { }

impl<T> S {
  fn f(self, value: T) {
    __size_of::<T>();
  }
}

enum Option<T> {
    Some: T,
    None: ()
}

enum OtherOption<T> {
    Some: T,
    None: ()
}

impl<T> Option<T> {
    fn return_false(self, _other: OtherOption<Self>) -> bool {
        false
    }
}

pub fn main() {
    (S{}).f(true);
    let a = Option::Some(5u64);
    let b = OtherOption::Some(Option::None);
    let _ = a.return_false(b);
}
