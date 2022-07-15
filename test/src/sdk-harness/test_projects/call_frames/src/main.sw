contract;

use std::contract_id::ContractId;
use call_frames_test_abi::{CallFramesTest, TestStruct, TestStruct2};
use std::context::call_frames::*;
use std::mem::read;

impl CallFramesTest for Contract {
    fn get_id() -> ContractId {
        contract_id()
    }

    fn get_asset_id() -> ContractId {
        msg_asset_id()
    }

    fn get_code_size() -> u64 {
        code_size()
    }

    fn get_first_param() -> u64 {
        first_param()
    }

    fn get_selector() -> u32 {
        selector()
    }

    fn get_selector_with_arguments(arg0: u64) -> u32 {
        selector()
    }

    fn get_arguments_u64(arg0: u64) -> u64 {
        arguments()
    }

    fn get_arguments_bool(arg0: bool) -> bool {
        arguments()
    }

    fn get_arguments_struct(arg0: TestStruct) -> TestStruct {
        arguments::<TestStruct>()
    }

    fn get_arguments_multiple_params(arg0: bool, arg1: u64) -> (bool, u64) {
        arguments::<(bool, u64)>()
    }

    fn get_arguments_multiple_params2(arg0: u64, arg1: TestStruct, arg2: TestStruct2) -> (u64, TestStruct, TestStruct2) {
        arguments::<(u64, TestStruct, TestStruct2)>()
    }
}
