script;

/* ------------------*/

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

/* ------------------*/

enum Bar {
    a: (),
    b: (),
}

impl Bar {
    fn trivial(self) -> bool {
        false
    }
}

fn func2(m: Bar) -> bool {
    m.trivial()
}

/* ------------------*/

struct Foo2<T> {
    foo: T,
}

impl<T> Foo2<T> {
    fn trivial(self) -> bool {
        false
    }
}

fn func3(a: Foo2<u8>) -> Foo2<bool> {
    if a.trivial() {
        Foo2 {foo: false}
    } else {
        Foo2 {foo: true}
    }
}

fn func4(b: Foo2<bool>) -> Foo2<u8> {
    if b.trivial() {
        Foo2 {foo: 0u8} 
    } else {
        Foo2 {foo: 1u8}
    }
}

/* ------------------*/

pub enum Rezult<T, E> {
    Ok: T,
    Err: E,
}

pub enum DumbError {
    Error: (),
}

impl<T, E> Rezult<T, E> {
    fn trivial(self) -> bool {
        false
    }
}

pub fn func5(r: Rezult<u8, DumbError>) -> Rezult<u8, DumbError> {
    if r.trivial() {
        Rezult::Err(DumbError::Error)
    } else {
        Rezult::Ok(1u8)
    }
}

pub fn func6(r: Rezult<bool, DumbError>) -> Rezult<bool, DumbError> {
   if r.trivial() {
        Rezult::Err(DumbError::Error)
    } else {
        Rezult::Ok(true)
    }
}

/* ------------------*/

fn main() -> bool {
  true
}
