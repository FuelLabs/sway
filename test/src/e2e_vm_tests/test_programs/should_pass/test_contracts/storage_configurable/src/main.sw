contract;

configurable {
  INITIAL_OWNER: Option<Identity> = Option::None,
}

storage {
  owner: Option<Identity> = INITIAL_OWNER,
}

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}