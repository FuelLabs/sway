script;

struct Foo1 {
    a: u64,
    b: u64,
}

impl Foo1 {
    fn trivial(self) -> bool {
      false
    }
}

fn func1() -> bool {
    let f = Foo1 {a: 0, b: 0};
    f.trivial()
}


enum Bar {
    a: (),
    b: (),
}

impl Bar {
    fn trivial(self) -> bool {
        false
    }
}

fn bar(m: Bar) -> bool {
    m.trivial()
}


struct Foo2<T> {
    foo: T,
}

fn func2(a: Foo2<u8>) -> u8 {
  a.foo
}

fn func3(b: Foo2<u32>) -> u32 {
  b.foo
}


pub enum Rezult<T, E> {
    Ok: T,
    Err: E,
}

pub enum DumbError {
    Error: (),
}

pub fn func4() -> Rezult<u8, DumbError> {
    if false {
        Rezult::Err(DumbError::Error)
    } else {
        Rezult::Ok(1u8)
    }
}

pub fn func5() -> Rezult<bool, DumbError> {
   if false {
        Rezult::Err(DumbError::Error)
    } else {
        Rezult::Ok(true)
    }
}

fn main() -> bool {
  true
}
