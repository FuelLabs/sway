library call_frames_test_abi;

use std::contract_id::ContractId;

pub struct TestStruct {
    value0: u64,
    value1: bool,
}

abi CallFramesTest {
    fn get_id() -> ContractId;
    fn get_asset_id() -> ContractId;
    fn get_code_size() -> u64;
    fn get_first_param() -> u64;
    fn get_second_param_u64(arg0: u64) -> u64;
    fn get_second_param_bool(arg0: bool) -> bool;
    fn get_second_param_struct(arg0: TestStruct) -> TestStruct;
    fn get_second_param_multiple_params(arg0: bool, arg1: u64) -> (bool, u64);
    fn get_second_param_multiple_params2(arg0: u64, arg1: TestStruct, arg2: b256) -> (u64, TestStruct, b256);
}
