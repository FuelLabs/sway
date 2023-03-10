contract;

abi MyContract {
    fn test_true() -> bool;
}

impl MyContract for Contract {
    fn test_true() -> bool {
        true
    }
}

abi MyContract2 {
    fn test_false() -> bool;
}

#[test]
fn test_contract_call() {
  let caller = abi(MyContract, CONTRACT_ID);
  let result = caller.test_true {}();
  assert(result == true)
}

#[test]
fn test_contract_2_call() {
  let caller = abi(MyContract2, contract2::CONTRACT_ID);
  let result = caller.test_false {}();
  assert(result == false)
}

#[test]
fn test_contract_multi_call() {
  let caller = abi(MyContract, CONTRACT_ID);
  let caller2 = abi(MyContract2, contract2::CONTRACT_ID);

  let should_be_true  = caller.test_true {}();
  let should_be_false = caller2.test_false {}();

  assert(should_be_true == true);
  assert(should_be_false == false);
}
