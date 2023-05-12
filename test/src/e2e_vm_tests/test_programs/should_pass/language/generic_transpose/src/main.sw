script;

struct GenericStruct<T> {
    x:T
}

impl<T, E> Option<Result<T, E>> {
    pub fn transpose(self) -> Result<Option<T>, E> {
      match self {
          Some(Ok(x)) => Ok(Some(x)),
          Some(Err(e)) => Err(e),
          None => Ok(None),
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
    let y: Option<Result<u64, u8>> = Some(Ok(5));
    assert(y.transpose().unwrap().unwrap() == 5);

    let y: GenericStruct<Option<u64>> = GenericStruct{ x: Some(5)};
    assert(y.transpose().unwrap().x == 5);

    true
}
