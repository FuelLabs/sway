contract;

use contract_b::CONTRACT_ID as CONTRACT_B_ID;

abi MyContract {
      fn test_function();
}
  
  impl MyContract for Contract {
      fn test_function() {
    	  let CONTRACT_B_ID = CONTRACT_B_ID;
      }
  }

