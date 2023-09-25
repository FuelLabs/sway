contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}

#[test]
fn test_bam() {
  assert(1 == 1)
}

#[test]
fn test_bum() {
  assert(1 == 1)
}
