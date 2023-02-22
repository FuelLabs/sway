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
  let contract2_id = 0xad4770679dec457bd9c0875d5ea52d75ac735ef28c0187d0bf7ee1dff5b9cee3;
  let caller = abi(MyContract2, contract2_id);
  let result = caller.test_false {}();
  assert(result == false)
}

#[test]
fn test_contract_multi_call() {
  let caller = abi(MyContract, CONTRACT_ID);

  let contract2_id = 0xad4770679dec457bd9c0875d5ea52d75ac735ef28c0187d0bf7ee1dff5b9cee3;
  let caller2 = abi(MyContract2, contract2_id);

  let should_be_true  = caller.test_true {}();
  let should_be_false = caller2.test_false {}();

  assert(should_be_true == true);
  assert(should_be_false == false);
}
