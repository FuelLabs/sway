script;

fn bar(a: bool, b: u64) -> u64 {
  if a {
    b * 2
  } else {
    0
  }
}

fn main() -> u64 {
  bar(true)
}
