script;

fn main() -> bool {
  let mut vector = Vec::new();
  vector.push(1);
  vector.push(2);
  vector.push(3);
  vector.push(4);

  let mut iter = vector.iter();
  assert_eq(Some(1), iter.next());
  assert_eq(Some(2), iter.next());
  assert_eq(Some(3), iter.next());
  assert_eq(Some(4), iter.next());
  assert_eq(None, iter.next());
  assert_eq(None, iter.next());
  
  true
}