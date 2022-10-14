contract;

use contract_b::CONTRACT_ID as contract_b_id;

abi MyContract {
      fn test_function();
}
  
  impl MyContract for Contract {
      fn test_function() {
    	  let contract_b_id = contract_b_id;
      }
  }

