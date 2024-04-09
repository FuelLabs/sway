contract;

abi RunExternalTest{
    fn double_value(foo: u64) -> u64;
}

impl RunExternalTest for Contract {
    fn double_value(foo: u64) -> u64 {
        foo * 2
    }
}

// ANCHOR: fallback
#[fallback]
fn fallback() -> u64 {
    use std::call_frames::*;
    let foo = second_param::<u64>();
    foo * 3
}
// ANCHOR_END: fallback