contract;

configurable {
  INITIAL_OWNER: u64 = 0u64,
}

storage {
  owner: u64 = INITIAL_OWNER,
}

abi MyContract {
    fn test_function() -> u64;
}

impl MyContract for Contract {
    fn test_function() -> u64 {
      storage.owner
    }
}
