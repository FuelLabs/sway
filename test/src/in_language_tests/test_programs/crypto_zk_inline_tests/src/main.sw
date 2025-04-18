library;

use std::crypto::{
    alt_bn128::{
        alt_bn128_add,
        alt_bn128_mul,
        alt_bn128_pairing_check,
    },
    point2d::*,
    scalar::*,
};

#[test]
fn zk_alt_bn128_add() {
    // From https://github.com/bluealloy/revm/blob/main/crates/precompile/src/bn128.rs
    let p1_1 = Point2D::from([
        0x18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9,
        0x063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266,
    ]);
    let p2_1 = Point2D::from([
        0x07c2b7f58a84bd6145f00c9c2bc0bb1a187f20ff2c92963a88019e7c6a014eed,
        0x06614e20c147e940f2d70da3f74c9a17df361706a4485c742bd6788478fa17d7,
    ]);
    let expected_point_1 = Point2D::from([
        0x2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee202839703,
        0x301d1d33be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c915,
    ]);

    let result_1 = alt_bn128_add(p1_1, p2_1);
    assert(result_1 == expected_point_1);

    // From https://github.com/bluealloy/revm/blob/main/crates/precompile/src/bn128.rs
    let p1_2 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);
    let p2_2 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);
    let expected_point_2 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);

    let result_2 = alt_bn128_add(p1_2, p2_2);
    assert(result_2 == expected_point_2);

    // From https://github.com/ethereum/tests/blob/develop/GeneralStateTests/stZeroKnowledge2/ecadd_1145-3932_2969-1336_21000_128.json
    let p1_3 = Point2D::from([
        0x17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa9,
        0x01e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c,
    ]);
    let p2_3 = Point2D::from([
        0x039730ea8dff1254c0fee9c0ea777d29a9c710b7e616683f194f18c43b43b869,
        0x073a5ffcc6fc7a28c30723d6e58ce577356982d65b833a5a5c15bf9024b43d98,
    ]);
    let expected_point_3 = Point2D::from([
        0x15bf2bb17880144b5d1cd2b1f46eff9d617bffd1ca57c37fb5a49bd84e53cf66,
        0x049c797f9ce0d17083deb32b5e36f2ea2a212ee036598dd7624c168993d1355f,
    ]);

    let result_3 = alt_bn128_add(p1_3, p2_3);
    assert(result_3 == expected_point_3);

    // From https://github.com/matter-labs/era-compiler-tests/blob/2253941334797eb2a997941845fb9eb0d436558b/yul/precompiles/ecadd.yul#L123
    let p1_4 = Point2D::from([
        0x17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa9,
        0x01e0559bacb160664764a357af8a9fe70baa9258e0b959273ffc5718c6d4cc7c,
    ]);
    let p2_4 = Point2D::from([
        0x17c139df0efee0f766bc0204762b774362e4ded88953a39ce849a8a7fa163fa9,
        0x2e83f8d734803fc370eba25ed1f6b8768bd6d83887b87165fc2434fe11a830cb,
    ]);
    let expected_point_4 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);

    let result_4 = alt_bn128_add(p1_4, p2_4);
    assert(result_4 == expected_point_4);

    // From https://github.com/poanetwork/parity-ethereum/blob/2ea4265b0083c4148571b21e1079c641d5f31dc2/ethcore/benches/builtin.rs#L486
    let p1_5 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ]);
    let p2_5 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ]);
    let expected_point_5 = Point2D::from([
        0x030644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd3,
        0x15ed738c0e0a7c92e7845f96b2ae9c0a68a6a449e3538fc7ff3ebf7a5a18a2c4,
    ]);

    let result_5 = alt_bn128_add(p1_5, p2_5);
    assert(result_5 == expected_point_5);
}

#[test(should_revert)]
fn revert_zk_alt_bn128_add_fail() {
    // From https://github.com/bluealloy/revm/blob/main/crates/precompile/src/bn128.rs
    let p1 = Point2D::from([
        0x1111111111111111111111111111111111111111111111111111111111111111,
        0x1111111111111111111111111111111111111111111111111111111111111111,
    ]);
    let p2 = Point2D::from([
        0x1111111111111111111111111111111111111111111111111111111111111111,
        0x1111111111111111111111111111111111111111111111111111111111111111,
    ]);

    let _result = alt_bn128_add(p1, p2);
}

// TODO: Uncomment and implement test when another curve is supported
// #[test(should_revert)]
// fn revert_zk_alt_bn128_add_invalid_point() {
//     // This needs to be an invalid point
//     let p1 = Point2D::from();
//     let p2 = Point2D::from();

//     let _result = alt_bn128_add(p1, p2);
// }


#[test]
fn zk_alt_bn128_mul() {
    // From https://github.com/bluealloy/revm/blob/main/crates/precompile/src/bn128.rs
    let p1_1 = Point2D::from([
        0x2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb7,
        0x21611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb204,
    ]);
    let scalar_1 = Scalar::from(0x00000000000000000000000000000000000000000000000011138ce750fa15c2);
    let expected_point_1 = Point2D::from([
        0x070a8d6a982153cae4be29d434e8faef8a47b274a053f5a4ee2a6c9c13c31e5c,
        0x031b8ce914eba3a9ffb989f9cdd5b0f01943074bf4f0f315690ec3cec6981afc,
    ]);

    let result_1 = alt_bn128_mul(p1_1, scalar_1);
    assert(result_1 == expected_point_1);

    // From https://github.com/bluealloy/revm/blob/main/crates/precompile/src/bn128.rs
    let p1_2 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);
    let scalar_2 = Scalar::from(0x0200000000000000000000000000000000000000000000000000000000000000);
    let expected_point_2 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);

    let result_2 = alt_bn128_mul(p1_2, scalar_2);
    assert(result_2 == expected_point_2);

    // From https://github.com/ethereum/tests/blob/develop/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1456_21000_96.json
    let p1_3 = Point2D::from([
        0x1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe3,
        0x1a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f6,
    ]);
    let scalar_3 = Scalar::from(0x0000000000000000000000000000000100000000000000000000000000000000);
    let expected_point_3 = Point2D::from([
        0x1051acb0700ec6d42a88215852d582efbaef31529b6fcbc3277b5c1b300f5cf0,
        0x135b2394bb45ab04b8bd7611bd2dfe1de6a4e6e2ccea1ea1955f577cd66af85b,
    ]);

    let result_3 = alt_bn128_mul(p1_3, scalar_3);
    assert(result_3 == expected_point_3);

    // From https://github.com/matter-labs/era-compiler-tests/blob/2253941334797eb2a997941845fb9eb0d436558b/yul/precompiles/ecmul.yul#L185C21-L185C98
    let p1_4 = Point2D::from([
        0x1a87b0584ce92f4593d161480614f2989035225609f08058ccfa3d0f940febe3,
        0x1a2f3c951f6dadcc7ee9007dff81504b0fcd6d7cf59996efdc33d92bf7f9f8f6,
    ]);
    let scalar_4 = Scalar::from(0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001);
    let expected_point_4 = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);

    let result_4 = alt_bn128_mul(p1_4, scalar_4);
    assert(result_4 == expected_point_4);

    // From https://github.com/poanetwork/parity-ethereum/blob/2ea4265b0083c4148571b21e1079c641d5f31dc2/ethcore/benches/builtin.rs#L516
    let p1_5 = Point2D::from([
        0x2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb7,
        0x21611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb204,
    ]);
    let scalar_5 = Scalar::from(0x00000000000000000000000000000000000000000000000011138ce750fa15c2);
    let expected_point_5 = Point2D::from([
        0x070a8d6a982153cae4be29d434e8faef8a47b274a053f5a4ee2a6c9c13c31e5c,
        0x031b8ce914eba3a9ffb989f9cdd5b0f01943074bf4f0f315690ec3cec6981afc,
    ]);

    let result_5 = alt_bn128_mul(p1_5, scalar_5);
    assert(result_5 == expected_point_5);
}

#[test(should_revert)]
fn revert_zk_alt_bn128_mul_fail() {
    // From https://github.com/bluealloy/revm/blob/main/crates/precompile/src/bn128.rs
    let p = Point2D::from([
        0x1111111111111111111111111111111111111111111111111111111111111111,
        0x1111111111111111111111111111111111111111111111111111111111111111,
    ]);
    let scalar = Scalar::from(0x0f00000000000000000000000000000000000000000000000000000000000000);

    let _result = alt_bn128_mul(p, scalar);
}

// TODO: Uncomment and implement test when another curve is supported
// #[test(should_revert)]
// fn revert_zk_alt_bn128_mul_invalid_point() {
//     // This should be an invalid point
//     let p = Point2D::from();
//     let scalar = Scalar::from(0x00000000000000000000000000000000000000000000000011138ce750fa15c2);

//     let _result = alt_bn128_mul(p, scalar);
// }

// TODO: Uncomment and implement test when another curve is supported
// #[test(should_revert)]
// fn revert_zk_alt_bn128_mul_invalid_scalar() {
//     // This should be an invalid scalar
//     let p = Point2D::from([0x2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb7, 0x21611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb204]);
//     let scalar = Scalar::from();

//     let _result = alt_bn128_mul(p, scalar);
// }


#[test]
fn zk_alt_bn128_pairing_check() {
    // From https://github.com/bluealloy/revm/blob/main/crates/precompile/src/bn128.rs
    let mut points_vec_1: Vec<(Point2D, [Point2D; 2])> = Vec::new();
    let p1_1_1 = Point2D::from((
        0x1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f59,
        0x3034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41,
    ));
    let g1_1_1 = Point2D::from((
        0x209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf7,
        0x04bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a41678,
    ));
    let g2_1_1 = Point2D::from((
        0x2bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d,
        0x120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550,
    ));
    points_vec_1.push((p1_1_1, [g1_1_1, g2_1_1]));

    let p1_2_1 = Point2D::from((
        0x111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c,
        0x2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411,
    ));
    let g1_2_1 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_2_1 = Point2D::from((
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
    ));
    points_vec_1.push((p1_2_1, [g1_2_1, g2_2_1]));

    assert(alt_bn128_pairing_check(points_vec_1));

    // From https://github.com/ethereum/tests/blob/develop/GeneralStateTests/stZeroKnowledge/ecpairing_three_point_match_1.json
    let mut points_vec_2: Vec<(Point2D, [Point2D; 2])> = Vec::new();
    let p1_1_2 = Point2D::from((
        0x105456a333e6d636854f987ea7bb713dfd0ae8371a72aea313ae0c32c0bf1016,
        0x0cf031d41b41557f3e7e3ba0c51bebe5da8e6ecd855ec50fc87efcdeac168bcc,
    ));
    let g1_1_2 = Point2D::from((
        0x0476be093a6d2b4bbf907172049874af11e1b6267606e00804d3ff0037ec57fd,
        0x3010c68cb50161b7d1d96bb71edfec9880171954e56871abf3d93cc94d745fa1,
    ));
    let g2_1_2 = Point2D::from((
        0x14c059d74e5b6c4ec14ae5864ebe23a71781d86c29fb8fb6cce94f70d3de7a21,
        0x01b33461f39d9e887dbb100f170a2345dde3c07e256d1dfa2b657ba5cd030427,
    ));
    points_vec_2.push((p1_1_2, [g1_1_2, g2_1_2]));

    let p1_2_2 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_2_2 = Point2D::from((
        0x1a2c3013d2ea92e13c800cde68ef56a294b883f6ac35d25f587c09b1b3c635f7,
        0x290158a80cd3d66530f74dc94c94adb88f5cdb481acca997b6e60071f08a115f,
    ));
    let g2_2_2 = Point2D::from((
        0x2f997f3dbd66a7afe07fe7862ce239edba9e05c5afff7f8a1259c9733b2dfbb9,
        0x29d1691530ca701b4a106054688728c9972c8512e9789e9567aae23e302ccd75,
    ));
    points_vec_2.push((p1_2_2, [g1_2_2, g2_2_2]));

    assert(alt_bn128_pairing_check(points_vec_2));

    // From https://github.com/ethereum/tests/blob/develop/GeneralStateTests/stZeroKnowledge/ecpairing_three_point_fail_1.json
    let mut points_vec_3: Vec<(Point2D, [Point2D; 2])> = Vec::new();
    let p1_1_3 = Point2D::from((
        0x105456a333e6d636854f987ea7bb713dfd0ae8371a72aea313ae0c32c0bf1016,
        0x0cf031d41b41557f3e7e3ba0c51bebe5da8e6ecd855ec50fc87efcdeac168bcc,
    ));
    let g1_1_3 = Point2D::from((
        0x0476be093a6d2b4bbf907172049874af11e1b6267606e00804d3ff0037ec57fd,
        0x3010c68cb50161b7d1d96bb71edfec9880171954e56871abf3d93cc94d745fa1,
    ));
    let g2_1_3 = Point2D::from((
        0x14c059d74e5b6c4ec14ae5864ebe23a71781d86c29fb8fb6cce94f70d3de7a21,
        0x01b33461f39d9e887dbb100f170a2345dde3c07e256d1dfa2b657ba5cd030427,
    ));
    points_vec_3.push((p1_1_3, [g1_1_3, g2_1_3]));

    let p1_2_3 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_2_3 = Point2D::from((
        0x1a2c3013d2ea92e13c800cde68ef56a294b883f6ac35d25f587c09b1b3c635f7,
        0x290158a80cd3d66530f74dc94c94adb88f5cdb481acca997b6e60071f08a115f,
    ));
    let g2_2_3 = Point2D::from((
        0x00cacf3523caf879d7d05e30549f1e6fdce364cbb8724b0329c6c2a39d4f018e,
        0x0692e55db067300e6e3fe56218fa2f940054e57e7ef92bf7d475a9d8a8502fd2,
    ));
    points_vec_3.push((p1_2_3, [g1_2_3, g2_2_3]));

    assert(!alt_bn128_pairing_check(points_vec_3));

    // From https://github.com/ethereum/tests/blob/develop/GeneralStateTests/stZeroKnowledge/ecpairing_three_point_fail_1.json
    let mut points_vec_4: Vec<(Point2D, [Point2D; 2])> = Vec::new();
    let p1_1_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_1_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_1_4 = Point2D::from((
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
    ));
    points_vec_4.push((p1_1_4, [g1_1_4, g2_1_4]));

    let p1_2_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_2_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_2_4 = Point2D::from((
        0x275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec,
        0x1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d,
    ));
    points_vec_4.push((p1_2_4, [g1_2_4, g2_2_4]));

    let p1_3_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_3_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_3_4 = Point2D::from((
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
    ));
    points_vec_4.push((p1_3_4, [g1_3_4, g2_3_4]));

    let p1_4_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_4_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_4_4 = Point2D::from((
        0x275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec,
        0x1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d,
    ));
    points_vec_4.push((p1_4_4, [g1_4_4, g2_4_4]));

    let p1_5_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_5_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_5_4 = Point2D::from((
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
    ));
    points_vec_4.push((p1_5_4, [g1_5_4, g2_5_4]));

    let p1_6_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_6_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_6_4 = Point2D::from((
        0x275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec,
        0x1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d,
    ));
    points_vec_4.push((p1_6_4, [g1_6_4, g2_6_4]));

    let p1_7_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_7_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_7_4 = Point2D::from((
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
    ));
    points_vec_4.push((p1_7_4, [g1_7_4, g2_7_4]));

    let p1_8_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_8_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_8_4 = Point2D::from((
        0x275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec,
        0x1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d,
    ));
    points_vec_4.push((p1_8_4, [g1_8_4, g2_8_4]));

    let p1_9_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_9_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_9_4 = Point2D::from((
        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa,
    ));
    points_vec_4.push((p1_9_4, [g1_9_4, g2_9_4]));
    let p1_10_4 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000002,
    ));
    let g1_10_4 = Point2D::from((
        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
    ));
    let g2_10_4 = Point2D::from((
        0x275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec,
        0x1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d,
    ));
    points_vec_4.push((p1_10_4, [g1_10_4, g2_10_4]));
    assert(alt_bn128_pairing_check(points_vec_4));
}

#[test(should_revert)]
fn revert_zk_alt_bn128_pairing_check() {
    let mut points_vec: Vec<(Point2D, [Point2D; 2])> = Vec::new();
    let p1_1 = Point2D::from((
        0x1111111111111111111111111111111111111111111111111111111111111111,
        0x1111111111111111111111111111111111111111111111111111111111111111,
    ));
    let g1_1 = Point2D::from((
        0x1111111111111111111111111111111111111111111111111111111111111111,
        0x1111111111111111111111111111111111111111111111111111111111111111,
    ));
    let g2_1 = Point2D::from((
        0x1111111111111111111111111111111111111111111111111111111111111111,
        0x1111111111111111111111111111111111111111111111111111111111111111,
    ));
    points_vec.push((p1_1, [g1_1, g2_1]));

    let _ = alt_bn128_pairing_check(points_vec);
}
