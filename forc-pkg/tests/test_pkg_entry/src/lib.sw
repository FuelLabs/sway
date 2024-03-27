library;

#[test]
fn log_unsigned_int() {
  let a = 10;
  log(a);
}

#[test]
fn log_bool() {
  let a = true;
  log(a);
}

#[test]
fn log_string_slice() {
  let a = "test";
  log(a);
}

#[test]
fn log_array() {
  let a: [u8; 5] = [1, 2, 3, 4, 5];
  log(a);
  let b: [u8; 5] = [1, 2, 3, 4, 5];
}

#[test]
fn log_tuple() {
  let a = (1,2);
  log(a);
  let b = (1,2);
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
}


enum Bar {
  Foo: (),
}

#[test]
fn log_enum() {
  let a: Bar = Bar::Foo;
  log(a);
}


#[test]
fn log_multiple() {
  let a = 10;
  log(a);
  let a = true;
  log(a);
  let a = "test";
  log(a);
  let a: [u8; 5] = [1, 2, 3, 4, 5];
  log(a);
  let a = (1,2);
  log(a);
  let a = Foo {
  	f1: 1,
  };
  log(a);
  let a: Bar = Bar::Foo;
  log(a);
}
