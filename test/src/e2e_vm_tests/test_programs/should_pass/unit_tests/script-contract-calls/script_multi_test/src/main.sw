script;

abi MyContract {
    fn test_false() -> bool;
}

fn main() {}

#[test]
fn test_contract_call() {
  let caller = abi(MyContract, contract_to_call::CONTRACT_ID);
  let result = caller.test_false();
  assert(result == false)
}
