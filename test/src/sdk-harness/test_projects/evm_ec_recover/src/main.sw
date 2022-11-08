contract;

use std::{
    b512::B512,
    ecr::ec_recover,
    vm::evm::{
        ecr::ec_recover_evm_address,
        evm_address::EvmAddress,
    },
};

abi EvmEcRecover {
    fn recover_evm_address(sig_r: b256, sig_v_s: b256, hash: b256) -> EvmAddress;
    fn recover_pub_key(sig_r: b256, sig_v_s: b256, hash: b256) -> (b256, b256);
}

impl EvmEcRecover for Contract {
    fn recover_evm_address(sig_r: b256, sig_v_s: b256, hash: b256) -> EvmAddress {
        let sig = B512::from((sig_r, sig_v_s));
        ec_recover_evm_address(sig, hash).unwrap()
    }
    fn recover_pub_key(sig_r: b256, sig_v_s: b256, hash: b256) -> (b256, b256) {
        let sig = B512::from((sig_r, sig_v_s));
        ec_recover(sig, hash).unwrap().into()
    }
}
