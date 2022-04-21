library call_frames_test_abi;

use std::contract_id::ContractId;

abi CallFramesTest {
    fn get_id() -> ContractId;
    fn get_asset_id() -> ContractId;
    fn get_code_size() -> u64;
    fn get_first_param() -> u64;
    fn get_second_param() -> u64;
}
