contract;

use std::contract_id::ContractId;
use call_frames_test_abi::CallFramesTest;
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

    fn get_second_param_u64(arg0: u64) -> u64 {
        second_param()
    }

    fn get_second_param_bool(arg0: bool) -> bool {
        read(second_param())
    }

    fn get_second_param_multiple_params(arg0: bool, arg1: u64) -> (bool, u64) {
        let (val0, val1) = read::<(bool, u64)>(second_param());
        (val0, val1)
    }
}
