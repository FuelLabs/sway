contract;

impl Contract {
    fn test_function() -> bool {
        true
    }
}

#[test]
fn test_log_4() {
  log(4);
  assert(1 == 1)
}

#[test]
fn test_log_2() {
  log(2);
  assert(1 == 1)
}
