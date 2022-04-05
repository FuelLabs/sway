script;

struct Generic1<T> {
    a: T,
}

struct Generic2<T> {
    b: Generic1<T>,
}

enum Generic3<T> {
  A: T,
  B: T
}

enum Generic4<T> {
  C: Generic3<T>,
  D: Generic3<T>
}

fn main() -> u64 {
  1
}