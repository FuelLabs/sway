library;

use std::{
    b512::B512,
    bytes::Bytes,
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

#[allow(deprecated)]
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

#[allow(deprecated)]
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

#[allow(deprecated)]
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

#[allow(deprecated)]
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

// Calculated with ed25519-dalek in a rust program

// use ed25519_dalek::ed25519::signature::Signer;
// use ed25519_dalek::SECRET_KEY_LENGTH;
// use ed25519_dalek::SigningKey;

// fn main() {
//     let bytes = hex::decode("638aa7abd1acd372c1ab3bc4951d9df3b33eabb2c019bf60a8c1ff2e424adeb67127a92630327cfa3fac37b0dcc969968da0efb18bbbbf498c16966373973b21").unwrap();
//     let bytes: [u8; 64] = bytes.try_into().unwrap();
//     let signing_key: SigningKey = SigningKey::from_keypair_bytes(&bytes).unwrap();

//     let keypair_bytes = signing_key.to_keypair_bytes();
//     let secret_key = &keypair_bytes[..SECRET_KEY_LENGTH];
//     let public_key = &keypair_bytes[SECRET_KEY_LENGTH..];

//     println!("Secret Key: {}", hex::encode(secret_key));
//     println!("Public Key: {}", hex::encode(public_key));


//     for x in [1, 16, 32, 64] {
//         let mut bytes = Vec::new();

//         for i in 0..x {
//             bytes.push(i as u8);
//         }

//         let bytes: &[u8] = &bytes;

//         let signature = signing_key.sign(bytes);
//         let signature = signature.to_bytes();
//         let lo: [u8; 32] = signature[0..32].try_into().unwrap();
//         let hi: [u8; 32] = signature[32..64].try_into().unwrap();

//         println!("x = {}, ({}, {})", x, hex::encode(lo), hex::encode(hi));
//     }
// }

// Secret Key: 638aa7abd1acd372c1ab3bc4951d9df3b33eabb2c019bf60a8c1ff2e424adeb6
// Public Key: 7127a92630327cfa3fac37b0dcc969968da0efb18bbbbf498c16966373973b21
// x = 1, (f5a5aafe874a12bf3460b0a31428306a3c0bf148b23c0726add73f149fb4238f, 11fd17bd7e9e64878f1cf680c316df925ff29784798cca9c8b70209f58fc6004)
// x = 16, (5573fe0bf140c8f1ca1b6b41fd4dc0bfcf92aefc67ab7dfd8aac1c264a66e67f, b47ed5cd8285cc2e8bf4a24a5e923a543278c43630f6e3d3da5a884de4982406)
// x = 32, (00d8a17c74a926854155f0092fe8c2db55220cff891a38f0ee00e549fec8ba07, f2dda3573b2f03d19eefebf93aa93d4ebca81e2c42de5b0f52d8c957f6390a0b)
// x = 64, (9a9e7077c905c855c86fb6aea6052f50a2cf29f70205f465d809cb0b81c6503f, fea5d320a5f9d4164b7eca627d3e81293083e7f6682b3b1ebc257459fcf89b08)
#[allow(deprecated)]
#[test]
fn ecr_ed_verify() {
    let pub_key = 0x7127a92630327cfa3fac37b0dcc969968da0efb18bbbbf498c16966373973b21;

    let lens = [1, 16, 32, 64];
    let sigs = [
        (
            0xf5a5aafe874a12bf3460b0a31428306a3c0bf148b23c0726add73f149fb4238f,
            0x11fd17bd7e9e64878f1cf680c316df925ff29784798cca9c8b70209f58fc6004,
        ),
        (
            0x5573fe0bf140c8f1ca1b6b41fd4dc0bfcf92aefc67ab7dfd8aac1c264a66e67f,
            0xb47ed5cd8285cc2e8bf4a24a5e923a543278c43630f6e3d3da5a884de4982406,
        ),
        (
            0x00d8a17c74a926854155f0092fe8c2db55220cff891a38f0ee00e549fec8ba07,
            0xf2dda3573b2f03d19eefebf93aa93d4ebca81e2c42de5b0f52d8c957f6390a0b,
        ),
        (
            0x9a9e7077c905c855c86fb6aea6052f50a2cf29f70205f465d809cb0b81c6503f,
            0xfea5d320a5f9d4164b7eca627d3e81293083e7f6682b3b1ebc257459fcf89b08,
        ),
    ];

    let mut i = 0;
    while i < 4 {
        let len = lens[i];
        let sig = B512::from((sigs[i].0, sigs[i].1));

        let mut msg = Bytes::new();
        let mut j = 0_u8;
        while j < len {
            msg.push(j);
            j += 1;
        }

        let verified = ed_verify(pub_key, sig, msg);
        assert(verified.is_ok());
        assert(verified.unwrap());

        i += 1;
    }
}

#[allow(deprecated)]
#[test]
fn ecr_ed_verify_fail() {
    let pub_key = 0x7127a92630327cfa3fac37b0dcc969968da0efb18bbbbf498c16966373973b21;
    let msg = Bytes::new();
    let sig = B512::from((
        0x19d821bfe7da223e53428b72a59e316c6981fcbba63dff89a11f01ce3d33af44,
        0xb49089aa12883bfffda92f3aadfd9153f654fb235baef6ab7958c6029fa35f0a,
    ));

    let verified = ed_verify(pub_key, sig, msg);
    // Should return error for msg len 0
    assert(verified.is_err());
}
