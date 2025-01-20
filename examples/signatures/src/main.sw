script;

fn main() {}

use std::{
    crypto::{
        ed25519::*,
        message::*,
        public_key::*,
        secp256k1::*,
        secp256r1::*,
        signature::*,
    },
    hash::{
        Hash,
        sha256,
    },
    vm::evm::evm_address::EvmAddress,
};

fn public_key_recovery() {
    // ANCHOR: public_key_recovery
    // Secp256rk1 Public Key Recovery
    let secp256k1_signature: Signature = Signature::Secp256k1(Secp256k1::from((
        0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8,
        0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678,
    )));
    let signed_message = Message::from(0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15);
    // A recovered public key pair.
    let secp256k1_public_key = secp256k1_signature.recover(signed_message);
    assert(secp256k1_public_key.is_ok());
    assert(
        secp256k1_public_key
            .unwrap() == PublicKey::from((
            0x41a55558a3486b6ee3878f55f16879c0798afd772c1506de44aba90d29b6e65c,
            0x341ca2e0a3d5827e78d838e35b29bebe2a39ac30b58999e1138c9467bf859965,
        )),
    );

    // Secp256r1 Public Key Recovery
    let secp256r1_signature = Signature::Secp256r1(Secp256r1::from((
        0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac,
        0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d,
    )));
    let signed_message = Message::from(0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323);
    // A recovered public key pair.
    let secp256r1_public_key = secp256r1_signature.recover(signed_message);
    assert(secp256r1_public_key.is_ok());
    assert(
        secp256r1_public_key
            .unwrap() == PublicKey::from((
            0xd6ea577a54ae42411fbc78d686d4abba2150ca83540528e4b868002e346004b2,
            0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452,
        )),
    );
    // ANCHOR_END: public_key_recovery
}

fn address_recovery() {
    // ANCHOR: address_recovery
    // Secp256k1 Address Recovery
    let secp256k1_signature = Signature::Secp256k1(Secp256k1::from((
        0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8,
        0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678,
    )));
    let signed_message = Message::from(0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15);
    // A recovered Fuel address.
    let secp256k1_address = secp256k1_signature.address(signed_message);
    assert(secp256k1_address.is_ok());
    assert(
        secp256k1_address
            .unwrap() == Address::from(0x02844f00cce0f608fa3f0f7408bec96bfd757891a6fda6e1fa0f510398304881),
    );

    // Secp256r1 Address Recovery
    let secp256r1_signature = Signature::Secp256r1(Secp256r1::from((
        0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c,
        0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d,
    )));
    let signed_message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    // A recovered Fuel address.
    let secp256r1_address = secp256r1_signature.address(signed_message);
    assert(secp256r1_address.is_ok());
    assert(
        secp256r1_address
            .unwrap() == Address::from(0xb4a5fabee8cc852084b71f17107e9c18d682033a58967027af0ab01edf2f9a6a),
    );

    // ANCHOR_END: address_recovery
}

fn evm_address_recovery() {
    // ANCHOR: evm_address_recovery
    // Secp256k1 EVM Address Recovery
    let secp256k1_signature = Signature::Secp256k1(Secp256k1::from((
        0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
        0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d,
    )));
    let signed_message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    // A recovered EVM address.
    let secp256k1_evm_address = secp256k1_signature.evm_address(signed_message);
    assert(secp256k1_evm_address.is_ok());
    assert(
        secp256k1_evm_address
            .unwrap() == EvmAddress::from(0x0000000000000000000000000ec44cf95ce5051ef590e6d420f8e722dd160ecb),
    );

    // Secp256r1 EVM Address Recovery
    let secp256r1_signature = Signature::Secp256r1(Secp256r1::from((
        0x62CDC20C0AB6AA7B91E63DA9917792473F55A6F15006BC99DD4E29420084A3CC,
        0xF4D99AF28F9D6BD96BDAAB83BFED99212AC3C7D06810E33FBB14C4F29B635414,
    )));
    let signed_message = Message::from(0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    // A recovered EVM address.
    let secp256r1_evm_address = secp256r1_signature.evm_address(signed_message);
    assert(secp256r1_evm_address.is_ok());
    assert(
        secp256r1_evm_address
            .unwrap() == EvmAddress::from(0x000000000000000000000000408eb2d97ef0beda0a33848d9e052066667cb00a),
    );
    // ANCHOR_END: evm_address_recovery
}

fn signature_verification() {
    // ANCHOR: signature_verification
    // Secp256k1 Signature Verification
    let secp256k1_signature = Signature::Secp256k1(Secp256k1::from((
        0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8,
        0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678,
    )));
    let secp256k1_public_key = PublicKey::from((
        0x41a55558a3486b6ee3878f55f16879c0798afd772c1506de44aba90d29b6e65c,
        0x341ca2e0a3d5827e78d838e35b29bebe2a39ac30b58999e1138c9467bf859965,
    ));
    let signed_message = Message::from(0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15);
    // A verified public key
    let secp256k1_verified = secp256k1_signature.verify(secp256k1_public_key, signed_message);
    assert(secp256k1_verified.is_ok());

    // Secp256r1 Signature Verification
    let secp256r1_signature = Signature::Secp256r1(Secp256r1::from((
        0xbd0c9b8792876712afadbff382e1bf31c44437823ed761cc3600d0016de511ac,
        0x44ac566bd156b4fc71a4a4cb2655d3da360c695edb27dc3b64d621e122fea23d,
    )));
    let secp256r1_public_key = PublicKey::from((
        0xd6ea577a54ae42411fbc78d686d4abba2150ca83540528e4b868002e346004b2,
        0x62660ecce5979493fe5684526e8e00875b948e507a89a47096bc84064a175452,
    ));
    let signed_message = Message::from(0x1e45523606c96c98ba970ff7cf9511fab8b25e1bcd52ced30b81df1e4a9c4323);
    // A verified public key 
    let secp256r1_verified = secp256r1_signature.verify(secp256r1_public_key, signed_message);
    assert(secp256r1_verified.is_ok());

    // Ed25519 Signature Verification
    let ed25519_public_key = PublicKey::from(0x314fa58689bbe1da2430517de2d772b384a1c1d2e9cb87e73c6afcf246045b10);
    let ed25519_signature = Signature::Ed25519(Ed25519::from((
        0xf38cef9361894be6c6e0eddec28a663d099d7ddff17c8077a1447d7ecb4e6545,
        0xf5084560039486d3462dd65a40c80a74709b2f06d450ffc5dc00345c6b2cdd00,
    )));
    let hashed_message = Message::from(sha256(b256::zero()));
    // A verified public key  
    let ed25519_verified = ed25519_signature.verify(ed25519_public_key, hashed_message);
    assert(ed25519_verified.is_ok());
    // ANCHOR_END: signature_verification
}

fn address_verification() {
    // ANCHOR: address_verification
    // Secp256k1 Address Verification
    let secp256k1_address = Address::from(0x02844f00cce0f608fa3f0f7408bec96bfd757891a6fda6e1fa0f510398304881);
    let secp256k1_signature = Secp256k1::from((
        0x61f3caf4c0912cec69ff0b226638d397115c623a7f057914d48a7e4daf1cf6d8,
        0x2555de81cd3a40382d3d64eb1c77e463eea5a76d65ec85f283e0b3d568352678,
    ));
    let signed_message = Message::from(0xa13f4ab54057ce064d3dd97ac3ff30ed704e73956896c03650fe59b1a561fe15);
    // A verifed address
    let secp256k1_verified = secp256k1_signature.verify_address(secp256k1_address, signed_message);
    assert(secp256k1_verified.is_ok());

    // Secp256r1 Address Verification
    let secp256r1_address = Address::from(0xb4a5fabee8cc852084b71f17107e9c18d682033a58967027af0ab01edf2f9a6a);
    let secp256r1_signature = Signature::Secp256r1(Secp256r1::from((
        0xbd0c9b8792876713afa8bf3383eebf31c43437823ed761cc3600d0016de5110c,
        0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d,
    )));
    let signed_message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    // A verified address
    let secp256r1_verified = secp256r1_signature.verify_address(secp256r1_address, signed_message);
    assert(secp256r1_verified.is_ok());

    // ANCHOR_END: address_verification
}

fn evm_address_verification() {
    // ANCHOR: evm_address_verification
    // Secp256k1 Address Verification
    let secp256k1_evm_address = EvmAddress::from(0x0000000000000000000000000ec44cf95ce5051ef590e6d420f8e722dd160ecb);
    let secp256k1_signature = Signature::Secp256k1(Secp256k1::from((
        0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
        0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d,
    )));
    let signed_message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    // A recovered EVM address.
    let secp256k1_verified = secp256k1_signature.verify_evm_address(secp256k1_evm_address, signed_message);
    assert(secp256k1_verified.is_ok());

    // Secp256r1 Address Verification
    let secp256r1_evm_address = EvmAddress::from(0x000000000000000000000000408eb2d97ef0beda0a33848d9e052066667cb00a);
    let secp256r1_signature = Signature::Secp256r1(Secp256r1::from((
        0x62CDC20C0AB6AA7B91E63DA9917792473F55A6F15006BC99DD4E29420084A3CC,
        0xF4D99AF28F9D6BD96BDAAB83BFED99212AC3C7D06810E33FBB14C4F29B635414,
    )));
    let signed_message = Message::from(0x290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563);
    // A recovered EVM address.
    let secp256r1_verified = secp256r1_signature.verify_evm_address(secp256r1_evm_address, signed_message);
    assert(secp256r1_verified.is_ok());
    // ANCHOR_END: evm_address_verification
}
