library;

use std::{
    crypto::{
        ed25519::Ed25519,
        message::Message,
        public_key::PublicKey,
        secp256k1::Secp256k1,
        secp256r1::Secp256r1,
        signature::Signature,
    },
    hash::{
        Hash,
        sha256,
    },
    vm::evm::evm_address::EvmAddress,
};

#[test]
fn signature_recover() {
    let hi_1 = 0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash_1 = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let pub_hi_1 = 0xd6ea577a54ae42411fbc78d686d4abba2150ca83540528e4b868002e346004b2;
    let pub_lo_1 = 0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452;

    let signature_1: Signature = Signature::Secp256r1(Secp256r1::from((hi_1, lo_1)));
    let public_key_1: PublicKey = PublicKey::from((pub_hi_1, pub_lo_1));
    let message_1: Message = Message::from(msg_hash_1);

    // A recovered public key pair.
    let result_public_key_1 = signature_1.recover(message_1);
    assert(result_public_key_1.is_ok());
    assert(public_key_1 == result_public_key_1.unwrap());

    let hi_2 = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo_2 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_2 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let pub_hi_2 = 0x41a55558a3486b6ee3878f55f16879c0798afd772c1506de44aba90d29b6e65c;
    let pub_lo_2 = 0x341ca2e0a3d5827e78d838e35b29bebe2a39ac30b58999e1138c9467bf859965;
    let signature_2: Signature = Signature::Secp256k1(Secp256k1::from((hi_2, lo_2)));
    let public_key_2 = PublicKey::from((pub_hi_2, pub_lo_2));
    let message_2 = Message::from(msg_hash_2);

    // A recovered public key pair.
    let result_public_key_2 = signature_2.recover(message_2);
    assert(result_public_key_2.is_ok());
    assert(public_key_2 == result_public_key_2.unwrap());

    let pub_key_3 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_3 = b256::zero();
    let msg_hash_3 = sha256(msg_3);
    let hi_3 = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo_3 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key_3: PublicKey = PublicKey::from(pub_key_3);
    let signature_3: Signature = Signature::Ed25519(Ed25519::from((hi_3, lo_3)));
    let message_3: Message = Message::from(msg_hash_3);

    // A verified public key with signature 
    let verified_3 = signature_3.recover(message_3);
    assert(verified_3.is_err());
}

#[test]
fn signature_address() {
    let hi_1 = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo_1 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_1 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let address_1 = Address::from(0x02844f00cce0f608fa3f0f7408bec96bfd757891a6fda6e1fa0f510398304881);
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let message_1 = Message::from(msg_hash_1);

    // A recovered Fuel address.
    let result_address_1 = signature_1.address(message_1);
    assert(result_address_1.is_ok());
    assert(result_address_1.unwrap() == address_1);

    let hi_2 = 0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address_2 = Address::from(0xb4a5fabee8cc852084b71f17107e9c18d682033a58967027af0ab01edf2f9a6a);
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_2, lo_2)));
    let message_2 = Message::from(msg_hash_2);

    // A recovered Fuel address.
    let result_address_2 = signature_2.address(message_2);
    assert(result_address_2.is_ok());
    assert(result_address_2.unwrap() == address_2);

    let pub_key_3 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_3 = b256::zero();
    let msg_hash_3 = sha256(msg_3);
    let hi_3 = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo_3 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key_3: PublicKey = PublicKey::from(pub_key_3);
    let signature_3: Signature = Signature::Ed25519(Ed25519::from((hi_3, lo_3)));
    let message_3: Message = Message::from(msg_hash_3);

    // A verified public key with signature 
    let verified_3 = signature_3.address(message_3);
    assert(verified_3.is_err());
}

#[test]
fn signature_evm_address() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_1 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let expected_evm_address_1 = EvmAddress::from(0x0000000000000000000000000ec44cf95ce5051ef590e6d420f8e722dd160ecb);
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let message_1 = Message::from(msg_hash_1);

    let result_1 = signature_1.evm_address(message_1);
    assert(result_1.is_ok());
    assert(result_1.unwrap() == expected_evm_address_1);

    let hi_2 = 0x62CDC20C0AB6AA7B91E63DA9917792473F55A6F15006BC99DD4E29420084A3CC;
    let lo_2 = 0xF4D99AF28F9D6BD96BDAAB83BFED99212AC3C7D06810E33FBB14C4F29B635414;
    let msg_hash_2 = 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563;
    let expected_evm_address_2 = EvmAddress::from(0x000000000000000000000000408eb2d97ef0beda0a33848d9e052066667cb00a);
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_2, lo_2)));
    let message_2 = Message::from(msg_hash_2);

    let result_2 = signature_2.evm_address(message_2);
    assert(result_2.is_ok());
    assert(result_2.unwrap() == expected_evm_address_2);

    let pub_key_3 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_3 = b256::zero();
    let msg_hash_3 = sha256(msg_3);
    let hi_3 = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo_3 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key_3: PublicKey = PublicKey::from(pub_key_3);
    let signature_3: Signature = Signature::Ed25519(Ed25519::from((hi_3, lo_3)));
    let message_3: Message = Message::from(msg_hash_3);

    // A verified public key with signature 
    let verified_3 = signature_3.evm_address(message_3);
    assert(verified_3.is_err());
}

#[test]
fn signature_verify() {
    let hi_1 = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo_1 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_1 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let pub_hi_1 = 0x41a55558a3486b6ee3878f55f16879c0798afd772c1506de44aba90d29b6e65c;
    let pub_lo_1 = 0x341ca2e0a3d5827e78d838e35b29bebe2a39ac30b58999e1138c9467bf859965;
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let public_key_1 = PublicKey::from((pub_hi_1, pub_lo_1));
    let message_1 = Message::from(msg_hash_1);

    // A recovered public key pair.
    let result_1 = signature_1.verify(public_key_1, message_1);
    assert(result_1.is_ok());

    let hi_2 = 0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac;
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d;
    let msg_hash_2 = 0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323;
    let pub_hi_2 = 0xd6ea577a54ae42411fbc78d686d4abba2150ca83540528e4b868002e346004b2;
    let pub_lo_2 = 0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452;
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_2, lo_2)));
    let public_key_2 = PublicKey::from((pub_hi_2, pub_lo_2));
    let message_2 = Message::from(msg_hash_2);

    // A recovered public key pair.
    let result_2 = signature_2.verify(public_key_2, message_2);
    assert(result_2.is_ok());

    let pub_key_3 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_3 = b256::zero();
    let msg_hash_3 = sha256(msg_3);
    let hi_3 = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo_3 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key_3: PublicKey = PublicKey::from(pub_key_3);
    let signature_3: Signature = Signature::Ed25519(Ed25519::from((hi_3, lo_3)));
    let message_3: Message = Message::from(msg_hash_3);

    // A verified public key with signature 
    let verified_3 = signature_3.verify(public_key_3, message_3);
    assert(verified_3.is_ok());
}

#[test]
fn signature_verify_address() {
    let hi_1 = 0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8;
    let lo_1 = 0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678;
    let msg_hash_1 = 0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15;
    let address_1 = Address::from(0x02844f00cce0f608fa3f0f7408bec96bfd757891a6fda6e1fa0f510398304881);
    let signature_1 = Secp256k1::from((hi_1, lo_1));
    let message_1 = Message::from(msg_hash_1);

    // A recovered Fuel address.
    let result_1 = signature_1.verify_address(address_1, message_1);
    assert(result_1.is_ok());

    let hi_2 = 0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_2 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_2 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address_2 = Address::from(0xb4a5fabee8cc852084b71f17107e9c18d682033a58967027af0ab01edf2f9a6a);
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_2, lo_2)));
    let message_2 = Message::from(msg_hash_2);

    // A recovered Fuel address.
    let result_2 = signature_2.verify_address(address_2, message_2);
    assert(result_2.is_ok());

    let pub_key_3 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_3 = b256::zero();
    let msg_hash_3 = sha256(msg_3);
    let hi_3 = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo_3 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key_3: PublicKey = PublicKey::from(pub_key_3);
    let signature_3: Signature = Signature::Ed25519(Ed25519::from((hi_3, lo_3)));
    let message_3: Message = Message::from(msg_hash_3);

    // A verified public key with signature 
    let verified_3 = signature_3.verify_address(address_2, message_3);
    assert(verified_3.is_err());
}

#[test]
fn signature_verify_evm_address() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let msg_hash_1 = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    let address_1 = EvmAddress::from(0x0000000000000000000000000ec44cf95ce5051ef590e6d420f8e722dd160ecb);
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let message_1 = Message::from(msg_hash_1);

    // A recovered Evm address.
    let result_1 = signature_1.verify_evm_address(address_1, message_1);
    assert(result_1.is_ok());

    let hi_2 = 0x62CDC20C0AB6AA7B91E63DA9917792473F55A6F15006BC99DD4E29420084A3CC;
    let lo_2 = 0xF4D99AF28F9D6BD96BDAAB83BFED99212AC3C7D06810E33FBB14C4F29B635414;
    let msg_hash_2 = 0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563;
    let address_2 = EvmAddress::from(0x000000000000000000000000408eb2d97ef0beda0a33848d9e052066667cb00a);
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_2, lo_2)));
    let message_2 = Message::from(msg_hash_2);

    // A recovered EVM address.
    let result_2 = signature_2.verify_evm_address(address_2, message_2);
    assert(result_2.is_ok());

    let pub_key_3 = 0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10;
    let msg_3 = b256::zero();
    let msg_hash_3 = sha256(msg_3);
    let hi_3 = 0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545;
    let lo_3 = 0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00;
    let public_key_3: PublicKey = PublicKey::from(pub_key_3);
    let signature_3: Signature = Signature::Ed25519(Ed25519::from((hi_3, lo_3)));
    let message_3: Message = Message::from(msg_hash_3);

    // A verified public key with signature 
    let verified_3 = signature_3.verify_evm_address(address_1, message_3);
    assert(verified_3.is_err());
}

#[test]
fn signature_as_secp256k1() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_1, lo_1)));
    let signature_3 = Signature::Ed25519(Ed25519::from((hi_1, lo_1)));

    assert(signature_1.as_secp256k1().unwrap() == Secp256k1::from((hi_1, lo_1)));
    assert(signature_2.as_secp256k1().is_none());
    assert(signature_3.as_secp256k1().is_none());
}

#[test]
fn signature_as_secp256r1() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_1, lo_1)));
    let signature_3 = Signature::Ed25519(Ed25519::from((hi_1, lo_1)));

    assert(signature_1.as_secp256r1().is_none());
    assert(signature_2.as_secp256r1().unwrap() == Secp256r1::from((hi_1, lo_1)));
    assert(signature_3.as_secp256r1().is_none());
}

#[test]
fn signature_as_ed25519() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_1, lo_1)));
    let signature_3 = Signature::Ed25519(Ed25519::from((hi_1, lo_1)));

    assert(signature_1.as_ed25519().is_none());
    assert(signature_2.as_ed25519().is_none());
    assert(signature_3.as_ed25519().unwrap() == Ed25519::from((hi_1, lo_1)));
}

#[test]
fn signature_is_secp256k1() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_1, lo_1)));
    let signature_3 = Signature::Ed25519(Ed25519::from((hi_1, lo_1)));

    assert(signature_1.is_secp256k1());
    assert(!signature_2.is_secp256k1());
    assert(!signature_3.is_secp256k1());
}

#[test]
fn signature_is_secp256r1() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_1, lo_1)));
    let signature_3 = Signature::Ed25519(Ed25519::from((hi_1, lo_1)));

    assert(!signature_1.is_secp256r1());
    assert(signature_2.is_secp256r1());
    assert(!signature_3.is_secp256r1());
}

#[test]
fn signature_is_ed25519() {
    let hi_1 = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo_1 = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let signature_1 = Signature::Secp256k1(Secp256k1::from((hi_1, lo_1)));
    let signature_2 = Signature::Secp256r1(Secp256r1::from((hi_1, lo_1)));
    let signature_3 = Signature::Ed25519(Ed25519::from((hi_1, lo_1)));

    assert(!signature_1.is_ed25519());
    assert(!signature_2.is_ed25519());
    assert(signature_3.is_ed25519());
}

#[test]
fn signature_bits() {
    let new_secp256r1 = Secp256r1::new();
    let new_secp256k1 = Secp256r1::new();
    let new_ed25519 = Ed25519::new();

    let secp256r1_bits = new_secp256r1.bits();
    let mut iter = 0;
    while iter < 64 {
        assert(secp256r1_bits[iter] == 0u8);
        iter += 1;
    }

    let secp256k1_bits = new_secp256k1.bits();
    let mut iter = 0;
    while iter < 64 {
        assert(secp256k1_bits[iter] == 0u8);
        iter += 1;
    }

    let ed25519_bits = new_ed25519.bits();
    let mut iter = 0;
    while iter < 64 {
        assert(ed25519_bits[iter] == 0u8);
        iter += 1;
    }
}

#[test]
fn signature_eq() {
    let secp256r1_1 = Signature::Secp256r1(Secp256r1::from((b256::zero(), b256::zero())));
    let secp256r1_2 = Signature::Secp256r1(Secp256r1::from((b256::zero(), b256::zero())));
    let secp256r1_3 = Signature::Secp256r1(Secp256r1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    )));
    let secp256r1_4 = Signature::Secp256r1(Secp256r1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    )));
    let secp256r1_5 = Signature::Secp256r1(Secp256r1::from((b256::max(), b256::max())));
    let secp256r1_6 = Signature::Secp256r1(Secp256r1::from((b256::max(), b256::max())));

    let secp256k1_1 = Signature::Secp256k1(Secp256k1::from((b256::zero(), b256::zero())));
    let secp256k1_2 = Signature::Secp256k1(Secp256k1::from((b256::zero(), b256::zero())));
    let secp256k1_3 = Signature::Secp256k1(Secp256k1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    )));
    let secp256k1_4 = Signature::Secp256k1(Secp256k1::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    )));
    let secp256k1_5 = Signature::Secp256k1(Secp256k1::from((b256::max(), b256::max())));
    let secp256k1_6 = Signature::Secp256k1(Secp256k1::from((b256::max(), b256::max())));

    let ed25519_1 = Signature::Ed25519(Ed25519::from((b256::zero(), b256::zero())));
    let ed25519_2 = Signature::Ed25519(Ed25519::from((b256::zero(), b256::zero())));
    let ed25519_3 = Signature::Ed25519(Ed25519::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    )));
    let ed25519_4 = Signature::Ed25519(Ed25519::from((
        b256::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    )));
    let ed25519_5 = Signature::Ed25519(Ed25519::from((b256::max(), b256::max())));
    let ed25519_6 = Signature::Ed25519(Ed25519::from((b256::max(), b256::max())));

    assert(secp256r1_1 == secp256r1_2);
    assert(secp256r1_3 == secp256r1_4);
    assert(secp256r1_5 == secp256r1_6);
    assert(secp256r1_1 != secp256r1_3);
    assert(secp256r1_1 != secp256r1_5);
    assert(secp256r1_3 != secp256r1_5);

    assert(secp256k1_1 == secp256k1_2);
    assert(secp256k1_3 == secp256k1_4);
    assert(secp256k1_5 == secp256k1_6);
    assert(secp256k1_1 != secp256k1_3);
    assert(secp256k1_1 != secp256k1_5);
    assert(secp256k1_3 != secp256k1_5);

    assert(ed25519_1 == ed25519_2);
    assert(ed25519_3 == ed25519_4);
    assert(ed25519_5 == ed25519_6);
    assert(ed25519_1 != ed25519_3);
    assert(ed25519_1 != ed25519_5);
    assert(ed25519_3 != ed25519_5);

    assert(secp256r1_1 != secp256k1_1);
    assert(secp256r1_1 != ed25519_1);
    assert(secp256r1_3 != secp256k1_3);
    assert(secp256r1_3 != ed25519_3);
    assert(secp256r1_5 != secp256k1_5);
    assert(secp256r1_5 != ed25519_5);

    assert(secp256r1_1 != ed25519_3);
    assert(secp256r1_1 != secp256k1_3);
    assert(secp256r1_1 != ed25519_5);
    assert(secp256r1_1 != secp256k1_5);

    assert(secp256k1_1 != ed25519_1);
    assert(secp256k1_3 != ed25519_3);
    assert(secp256k1_5 != ed25519_5);

    assert(secp256k1_1 != ed25519_3);
    assert(secp256k1_1 != ed25519_5);
}

#[test]
fn signature_codec() {
    let signature = Signature::Secp256r1(Secp256r1::from((b256::zero(), b256::zero())));
    log(signature);
}
