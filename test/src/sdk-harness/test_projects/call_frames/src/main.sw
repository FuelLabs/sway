contract;

use std::contract_id::ContractId;
use call_frames_test_abi::CallFramesTest;
use std::context::call_frames::*;

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

    fn get_second_param() -> u64 {
        second_param()
    }
}
