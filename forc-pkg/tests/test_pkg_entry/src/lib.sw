library;

#[test]
fn log_unsigned_int() {
  let a = 10;
  log(a);
  assert(a == 10)
}

#[test]
fn log_bool() {
  let a = true;
  log(a);
  assert(a)
}

#[test]
fn log_string_slice() {
  let a = "test";
  log(a);
  assert(a == "test")
}

#[test]
fn log_array() {
  let a: [u8; 5] = [1, 2, 3, 4, 5];
  log(a);
  let b: [u8; 5] = [1, 2, 3, 4, 5];
  assert(a[0] == b[0])
}

#[test]
fn log_tuple() {
  let a = (1,2);
  log(a);
  let b = (1,2);
  assert(a.0 == b.0)
}

struct Foo {
    f1: u32,
}

#[test]
fn log_struct() {
  let a = Foo {
  	f1: 1,
  };

  log(a);
  let b = Foo {
  	f1: 1,
  };
  assert(a.f1 == b.f1)
}


enum Bar {
  Foo: (),
}

#[test]
fn log_enum() {
  let a: Bar = Bar::Foo;
  log(a);
}
