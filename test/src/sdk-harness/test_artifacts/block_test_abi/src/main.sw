library block_test_abi;

abi BlockTest {
    fn get_block_height() -> u64;

    fn get_timestamp() -> u64;

    fn get_timestamp_of_block(block_height: u64) -> u64;

    fn get_block_and_timestamp() -> (u64, u64);
}
