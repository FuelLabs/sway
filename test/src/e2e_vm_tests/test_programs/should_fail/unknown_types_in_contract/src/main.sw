contract;

// Correct path
// use std::vm::evm::evm_address::EvmAddress;

use std::vm::evm::EvmAddress;

abi MyContract {
    fn test_function() -> EvmAddress;
}

fn test_it() -> EvmAddress {
    EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000)
}

impl MyContract for Contract {
    fn test_function() -> EvmAddress {
        test_it()
    }
}
