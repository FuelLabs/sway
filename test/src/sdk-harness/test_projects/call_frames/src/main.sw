contract;

use call_frames_test_abi::{CallFramesTest, TestStruct, TestStruct2};
use std::call_frames::*;

impl CallFramesTest for Contract {
    fn get_id_contract_id_this() -> ContractId {
        ContractId::this()
    }

    fn get_asset_id() -> AssetId {
        msg_asset_id()
    }

    fn get_code_size() -> u64 {
        code_size()
    }

    fn get_first_param() -> u64 {
        first_param()
    }

    fn get_second_param_u64(_arg0: u64) -> u64 {
        second_param()
    }

    fn get_second_param_bool(_arg0: bool) -> bool {
        called_args::<bool>()
    }

    fn get_second_param_struct(_arg0: TestStruct) -> TestStruct {
        called_args::<TestStruct>()
    }

    fn get_second_param_multiple_params(_arg0: bool, _arg1: u64) -> (bool, u64) {
        called_args::<(bool, u64)>()
    }

    fn get_second_param_multiple_params2(
        _arg0: u64,
        _arg1: TestStruct,
        _arg2: TestStruct2,
    ) -> (u64, TestStruct, TestStruct2) {
        called_args::<(u64, TestStruct, TestStruct2)>()
    }
}
