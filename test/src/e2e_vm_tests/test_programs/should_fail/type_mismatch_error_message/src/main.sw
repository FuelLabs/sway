script;

pub enum Result<T, E> {
    Ok: T,
    Err: E,
}

struct Data<T> {
  value: T,
  other: u64
}

fn example() {
  let foo = Result::Ok::<Data<bool>, str[4]>(Data { value: true, other: 1 });
  foo.does_not_exist();
}

fn main() {

}
