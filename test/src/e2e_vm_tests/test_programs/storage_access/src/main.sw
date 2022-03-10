contract;

storage {
    number: u64 = 0,
}

abi TestAbi {
  fn get_number() -> u64;
}

impl TestAbi for Contract {
    impure fn get_number() {
        storage.number  
    }
}
