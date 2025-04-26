script;

use std::debug::*;

struct S {}
enum E { None: (), Some: S }

fn f() -> std::result::Result<u64, u64> {
    Ok(1u64)
}

fn main() -> u64 {
    let _ = __dbg(());

    let _ = __dbg(true);
    let _ = __dbg(false);

    let _ = __dbg(u8::min());
    let _ = __dbg(1u8);
    let _ = __dbg(10u8);
    let _ = __dbg(100u8);
    let _ = __dbg(u8::max());

    let _ = __dbg(u16::min());
    let _ = __dbg(1u16);
    let _ = __dbg(10u16);
    let _ = __dbg(100u16);
    let _ = __dbg(u16::max());

    let _ = __dbg(u32::min());
    let _ = __dbg(1u32);
    let _ = __dbg(10u32);
    let _ = __dbg(100u32);
    let _ = __dbg(u32::max());

    let _ = __dbg(u64::min());
    let _ = __dbg(1u64);
    let _ = __dbg(10u64);
    let _ = __dbg(100u64);
    let _ = __dbg(u64::max());

    let _ = __dbg(u256::min());
    let _ = __dbg(1u256);
    let _ = __dbg(10u256);
    let _ = __dbg(100u256);
    let _ = __dbg(u256::max());

    let _ = __dbg(b256::min());
    let _ = __dbg(b256::max());

    // strings
    let _ = __dbg("A");
    let _ = __dbg(__to_str_array("A"));

    // Aggregates
    let _ = __dbg(("A", 0u8));
    let _ = __dbg([0u8, 1u8]);
    let _ = __dbg(__slice(&[0u8, 1u8], 0, 2));

    // Strucs and Enum
    let _ = __dbg(S { });
    let _ = __dbg(E::None);
    let _ = __dbg(E::Some(S { }));

    // all std library types must be debug
    let _ = __dbg(std::address::Address::zero());
    let _ = __dbg(std::asset_id::AssetId::default());
    let _ = __dbg(std::auth::AuthError::InputsNotAllOwnedBySameAddress);
    let _ = __dbg(std::b512::B512::zero());
    use std::block::*;
    let _ = __dbg(std::block::BlockHashError::BlockHeightTooHigh);
    let _ = __dbg({
        let mut bytes = std::bytes::Bytes::new();
        bytes.push(1);
        bytes.push(2);
        bytes.push(3);
        bytes
    });
    let _ = __dbg(std::contract_id::ContractId::zero());
    use std::ecr::*;
    let _ = __dbg(std::ecr::EcRecoverError::ZeroLengthMessage);
    let _ = __dbg(std::identity::Identity::Address(Address::zero()));
    use std::inputs::*;
    let _ = __dbg(std::inputs::Input::Coin);
    use std::low_level_call::*;
    let _ = __dbg(std::low_level_call::CallParams {
        coins: 1,
        asset_id: std::asset_id::AssetId::default(),
        gas: 2,
    });
    let _ = __dbg(std::option::Option::Some(1u8));
    use std::outputs::*;
    let _ = __dbg(std::outputs::Output::Coin);
    let _ = __dbg(f());
    use std::string::*;
    let _ = __dbg(std::string::String::from_ascii_str("hello"));
    let _ = __dbg(std::tx::Transaction::Script);
    use std::u128::*;
    let _ = __dbg(std::u128::U128::zero());
    let _ = __dbg(std::u128::U128Error::LossOfPrecision);
    let _ = __dbg({
        let mut v = std::vec::Vec::new();
        v.push(1u64);
        v.push(2u64);
        v.push(3u64);
        v
    });
    let _ = __dbg({
        let mut v = std::vec::Vec::new();
        v.push(1u64);
        v.push(2u64);
        v.push(3u64);
        v.iter()
    });

    // should return its argument
    __dbg(11u64)
}

#[test]
fn call_main() {
    let r = main();
    assert(r == 11u64);
}
