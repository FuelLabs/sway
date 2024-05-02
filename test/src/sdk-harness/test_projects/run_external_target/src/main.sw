contract;

abi RunExternalTest {
    fn double_value(foo: u64) -> u64;
}

impl RunExternalTest for Contract {
    fn double_value(foo: u64) -> u64 {
        __log(2);
        foo * 2
    }
}

#[fallback]
fn fallback() -> u64 {
    use std::call_frames::*;
    __log(3);
    __log(called_method());
    __log("double_value");
    __log(called_method() == "double_value");
    let foo = called_args::<u64>();
    foo * 3
}
