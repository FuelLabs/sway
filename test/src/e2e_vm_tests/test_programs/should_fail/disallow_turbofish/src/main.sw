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
  let _data = Data::<bool> {
    value: 7u64
  };
  let _elem1 = Elem::<bool> {
    value: true
  };
  let _elem2 = Elem {
    value: 6u64
  };
  let _res1 = Result::<u64, u8> {
    yes: 1u64,
    no: 8u8
  };
  let _res1 = Result::<bool> {
    yes: true,
    no: false
  };
  let _res1 = Result {
    yes: 1u32,
    no: 8u32
  };
  foo::<u64>();
}
