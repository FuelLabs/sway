script;

struct GenericStruct<T> {
    x:T
}

pub enum MyResult<T, E> {
    MyOk: T,
    MyErr: E,
}

impl<T, E> MyResult<T, E> {
    pub fn unwrap(self) -> T {
        match self {
            Self::MyOk(inner_value) => inner_value,
            _ => revert(0),
        }
    }
}

trait OptionTranspose<T, E> {
    fn transpose(self) -> MyResult<Option<T>, E>;
}

impl<T, E> OptionTranspose<T, E> for Option<MyResult<T, E>> {
    fn transpose(self) -> MyResult<Option<T>, E> {
      match self {
          Some(MyResult::MyOk(x)) => MyResult::MyOk(Some(x)),
          Some(MyResult::MyErr(e)) => MyResult::MyErr(e),
          None => MyResult::MyOk(None),
      }
    }
}

impl<T> GenericStruct<Option<T>> {
    pub fn transpose(self) -> Option<GenericStruct<T>> {
      match self {
          GenericStruct{x:Some(y)} => Some(GenericStruct{ x: y}),
          GenericStruct{x:None} => None,
      }
    }
}

fn main() -> bool {
    let y: Option<MyResult<u64, u8>> = Some(MyResult::MyOk(5));
    assert(y.transpose().unwrap().unwrap() == 5);

    let y: GenericStruct<Option<u64>> = GenericStruct{ x: Some(5)};
    assert(y.transpose().unwrap().x == 5);

    true
}
