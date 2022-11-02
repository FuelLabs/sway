contract;

use call_frames_test_abi::{CallFramesTest, TestStruct, TestStruct2};
use std::{call_frames::*};

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

    fn get_second_param_u64(arg0: u64) -> u64 {
        second_param()
    }

    fn get_second_param_bool(arg0: bool) -> bool {
        second_param()
    }

    fn get_second_param_struct(arg0: TestStruct) -> TestStruct {
        second_param::<TestStruct>()
    }

    fn get_second_param_multiple_params(arg0: bool, arg1: u64) -> (bool, u64) {
        second_param::<(bool, u64)>()
    }

    fn get_second_param_multiple_params2(
        arg0: u64,
        arg1: TestStruct,
        arg2: TestStruct2,
    ) -> (u64, TestStruct, TestStruct2) {
        second_param::<(u64, TestStruct, TestStruct2)>()
    }
}
