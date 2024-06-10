library;

use std::{
    b512::B512,
    ecr::{
        ec_recover,
        ec_recover_address,
        ec_recover_address_r1,
        ec_recover_r1,
        ed_verify,
    },
    hash::{
        Hash,
        sha256,
    },
};

#[test]
fn ecr_ec_recover() {
    let hi_1 = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo_1 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_1 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let pub_hi = 0x41a55558a3486b6ee3878f55f16879c0798afd772c1506de44aba90d29b6e65c;
    let pub_lo = 0x341ca2e0a3d5827e78d838e35b29bebe2a39ac30b58999e1138c9467bf859965;
    let signature_1: B512 = B512::from((hi_1, lo_1));
    // A recovered public key pair.
    let result_1 = ec_recover(signature_1, msg_hash_1);

    assert(result_1.is_ok());
    assert(result_1.unwrap().bits()[0] == pub_hi);
    assert(result_1.unwrap().bits()[1] == pub_lo);

    let hi_2 = b256::zero();
    let lo_2 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_2 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let signature_2: B512 = B512::from((hi_2, lo_2));
    // A recovered public key pair.
    let result_2 = ec_recover(signature_2, msg_hash_2);

    assert(result_2.is_err());
}

#[test]
fn ecr_ec_recover_r1() {
    let hi_1 = 0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash_1 = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let pub_hi_1 = 0xd6ea577a54ae42411fbc78d686d4abba2150ca83540528e4b868002e346004b2;
    let pub_lo_1 = 0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452;
    let signature_1: B512 = B512::from((hi_1, lo_1));
    // A recovered public key pair.
    let result_1 = ec_recover_r1(signature_1, msg_hash_1);

    assert(result_1.is_ok());
    assert(result_1.unwrap().bits()[0] == pub_hi_1);
    assert(result_1.unwrap().bits()[1] == pub_lo_1);

    let hi_2 = b256::zero();
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash_2 = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let signature_2: B512 = B512::from((hi_2, lo_2));
    let result_2 = ec_recover_r1(signature_2, msg_hash_2);

    assert(result_2.is_err());
}

#[test]
fn ecr_ec_recover_address() {
    let hi_1 = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo_1 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_1 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let address_1 = Address::from(0x02844f00cce0f608fa3f0f7408bec96bfd757891a6fda6e1fa0f510398304881);
    let signature_1: B512 = B512::from((hi_1, lo_1));
    // A recovered Fuel address.
    let result_1 = ec_recover_address(signature_1, msg_hash_1);
    assert(result_1.is_ok());
    assert(result_1.unwrap() == address_1);

    let hi_2 = b256::zero();
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let signature_2: B512 = B512::from((hi_2, lo_2));
    let result_2 = ec_recover_address(signature_2, msg_hash_2);

    assert(result_2.is_err());
}

#[test]
fn ecr_ec_recover_address_r1() {
    let hi_1 = 0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_1 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address_1 = Address::from(0xb4a5fabee8cc852084b71f17107e9c18d682033a58967027af0ab01edf2f9a6a);
    let signature_1: B512 = B512::from((hi_1, lo_1));
    // A recovered Fuel address.
    let result_1 = ec_recover_address_r1(signature_1, msg_hash_1);
    assert(result_1.is_ok());
    assert(result_1.unwrap() == address_1);

    let hi_2 = b256::zero();
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let signature_2: B512 = B512::from((hi_2, lo_2));
    let result_2 = ec_recover_address_r1(signature_2, msg_hash_2);

    assert(result_2.is_err());
}

#[test]
fn ecr_ed_verify() {
    let pub_key_1 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_1 = b256::zero();
    let msg_hash_1 = sha256(msg_1);

    let hi_1 = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo_1 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let signature_1: B512 = B512::from((hi_1, lo_1));
    // A verified public key with signature 
    let verified_1 = ed_verify(pub_key_1, signature_1, msg_hash_1);
    assert(verified_1.is_ok());
    assert(verified_1.unwrap());

    let pub_key_2 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_2 = b256::zero();
    let msg_hash_2 = sha256(msg_2);

    let hi_2 = b256::zero();
    let lo_2 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let signature_2: B512 = B512::from((hi_2, lo_2));
    let verified_2 = ed_verify(pub_key_2, signature_2, msg_hash_2);

    assert(verified_2.is_err());
}
