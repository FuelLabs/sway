library;

abi BlockTest {
    fn get_block_height() -> u32;

    fn get_timestamp() -> u64;

    fn get_timestamp_of_block(block_height: u32) -> u64;

    fn get_block_and_timestamp() -> (u32, u64);

    fn get_block_header_hash(h: u32) -> b256;

    fn get_chain_id() -> u64;
}
