library;

use std::{b512::B512, vm::evm::{ecr::ec_recover_evm_address, evm_address::EvmAddress}};

#[allow(deprecated)]
#[test]
fn ecr_ec_recover() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_1 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let expected_evm_address = EvmAddress::from(0x0000000000000000000000000ec44cf95ce5051ef590e6d420f8e722dd160ecb);
    let signature_1: B512 = B512::from((hi_1, lo_1));

    let result_1 = ec_recover_evm_address(signature_1, msg_hash_1);

    assert(result_1.is_ok());
    assert(result_1.unwrap() == expected_evm_address);

    let hi_2 = 0xbd0c9b8792876713afa8bf1383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_2 = 0xee45573606c96c98ba170ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52cad30b89df1e4a9c4323;
    let signature_2: B512 = B512::from((hi_2, lo_2));

    let result_2 = ec_recover_evm_address(signature_2, msg_hash_2);

    assert(result_2.is_err());
}
