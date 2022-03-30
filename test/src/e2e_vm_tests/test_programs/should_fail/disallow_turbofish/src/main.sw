script;

fn foo() { }

struct Data {
  value: u64
}

struct Elem<T> {
  value: T
}

struct Result<T, E> {
  yes: T,
  no: E
}

fn main() {
  let data = Data::<bool> {
    value: 7u64
  };
  let elem1 = Elem::<bool> {
    value: true
  };
  let elem2 = Elem {
    value: 6u64
  };
  let res1 = Result::<u64, u8> {
    yes: 1u64,
    no: 8u8
  };
    let res1 = Result::<bool> {
    yes: true,
    no: false
  };
  let res1 = Result {
    yes: 1u32,
    no: 8u32
  };
  foo::<u64>();
}
