script;

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
    fn return_false(self, other: OtherOption<Self>) -> bool {
        false
    }
}

fn main() {
    (S{}).f(true);
    let a = Option::Some(5u64);
    let b = OtherOption::Some(Option::None(()));
    let c = a.return_false(b);
}
