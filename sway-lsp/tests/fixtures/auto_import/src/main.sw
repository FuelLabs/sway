contract;

mod test_mod;
mod deep_mod;

use test_mod::A;

pub fn fun() {
    let _ = EvmAddress {
        value: b256::min(),
    };

    test_fun();
    deep_fun();
    A::fun();

    let _ = ZERO_B256; // l 18 c 19
    let _ = DeepEnum::Variant; // TODO: open an issue for this
    let _ = DeepStruct::<u64> { field: 0 };
}
