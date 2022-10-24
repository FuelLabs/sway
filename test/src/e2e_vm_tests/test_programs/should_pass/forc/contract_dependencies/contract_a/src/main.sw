contract;

use contract_b::CONTRACT_ID as CONTRACT_B_ID;
use contract_c::CONTRACT_ID as CONTRACT_C_ID;

abi MyContract {
      fn test_function();
}
  
  impl MyContract for Contract {
      fn test_function() {
    	  let CONTRACT_B_ID = CONTRACT_B_ID;
    	  let CONTRACT_C_ID = CONTRACT_C_ID;
      }
  }

