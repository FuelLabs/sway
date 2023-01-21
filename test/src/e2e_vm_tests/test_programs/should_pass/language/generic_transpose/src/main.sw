script;

struct GenericStruct<T> {
    x:T
}

impl<T, E> Option<Result<T, E>> {
    pub fn transpose(self) -> Result<Option<T>, E> {
      match self {
          Option::Some(Result::Ok(x)) => Result::Ok(Option::Some(x)),
          Option::Some(Result::Err(e)) => Result::Err(e),
          Option::None => Result::Ok(Option::None),
      }
    }
}

impl<T> GenericStruct<Option<T>> {
    pub fn transpose(self) -> Option<GenericStruct<T>> {
      match self {
          GenericStruct{x:Option::Some(y)} => Option::Some(GenericStruct{ x: y}),
          GenericStruct{x:Option::None} => Option::None,
      }
    }
}

fn main() -> bool {
    let y: Option<Result<u64, u8>> = Option::Some(Result::Ok(5));
    assert(y.transpose().unwrap().unwrap() == 5);

    let y: GenericStruct<Option<u64>> = GenericStruct{ x: Option::Some(5)};
    assert(y.transpose().unwrap().x == 5);

    true
}
