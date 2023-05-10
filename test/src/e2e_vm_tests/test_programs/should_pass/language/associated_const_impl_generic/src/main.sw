script;

struct Struct<T> { }

impl<T> Struct<T> {
    const ID: u32 = 1;
}

fn main() -> u32 {
  Struct::<u32>::ID
}

// TODO: errors with generics are not supported here