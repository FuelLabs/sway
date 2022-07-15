library call_frames_test_abi;

use std::contract_id::ContractId;

pub struct TestStruct {
    value0: u64,
    value1: bool,
}

pub struct TestStruct2 {
    value: u64,
}

abi CallFramesTest {
    fn get_id() -> ContractId;
    fn get_asset_id() -> ContractId;
    fn get_code_size() -> u64;
    fn get_first_param() -> u64;
    fn get_selector() -> u32;
    fn get_selector_with_arguments(arg0: u64) -> u32;
    fn get_arguments_u64(arg0: u64) -> u64;
    fn get_arguments_bool(arg0: bool) -> bool;
    fn get_arguments_struct(arg0: TestStruct) -> TestStruct;
    fn get_arguments_multiple_params(arg0: bool, arg1: u64) -> (bool, u64);
    fn get_arguments_multiple_params2(arg0: u64, arg1: TestStruct, arg2: TestStruct2) -> (u64, TestStruct, TestStruct2);
}
