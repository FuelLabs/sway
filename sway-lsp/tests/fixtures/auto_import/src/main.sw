contract;

mod test_mod;
mod deep_mod;

use test_mod::test_fun;

pub fn fun() {
    let _ = EvmAddress {
        value: b256::min(),
    };

    test_fun();
    deep_fun();
    A::fun();

    let a: DeepEnum = DeepEnum::Variant;
    let _ = DeepStruct::<u64> { field: 0 };

    let _ = TEST_CONST;
    let _ = ZERO_B256;

    let _ = overflow();
    let _: Result<Identity, AuthError> = msg_sender();
}

struct LocalStruct {
    field: u64,
}

impl DeepTrait for LocalStruct {
    fn deep_method(self) {}
}

impl TryFrom<u32> for LocalStruct {
    fn try_from(u: u32) -> Option<Self> {
        None
    }
}
