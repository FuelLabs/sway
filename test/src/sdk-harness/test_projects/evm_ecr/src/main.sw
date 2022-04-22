contract;

use std::vm::evm::ecr::*;
use std::address::Address;
use std::b512::B512;
use std::result::*;


abi EvmEcrecover {
    fn recover_ethereum_address(signature: B512, msg_hash: b256) -> Result<Address, EcRecoverError>;
}

impl EvmEcrecover for Contract {
    fn recover_ethereum_address(signature: B512, msg_hash: b256) -> Result<Address, EcRecoverError> {
        ec_recover_address(signature, msg_hash)
    }
}