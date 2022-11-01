contract;

use std::{
    ecr::{
        ec_recover,
        ec_recover_address,
    },
    b512::B512,
};

abi EcRecover {
    fn recover_address(sig_r: b256, sig_v_s: b256, hash: b256) -> Address;
    fn recover_pub_key(sig_r: b256, sig_v_s: b256, hash: b256) -> (b256, b256);
}

impl EcRecover for Contract {
    fn recover_address(sig_r: b256, sig_v_s: b256, hash: b256) -> Address {
        let sig = ~B512::from(sig_r, sig_v_s);
        ec_recover_address(sig, hash).unwrap()
    }
    fn recover_pub_key(sig_r: b256, sig_v_s: b256, hash: b256) -> (b256, b256) {
        let sig = ~B512::from(sig_r, sig_v_s);
        ec_recover(sig, hash).unwrap().into()
    }
}