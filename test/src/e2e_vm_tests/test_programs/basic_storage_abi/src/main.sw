library basic_storage_abi;

pub struct StoreU64Request {
  key: b256,
  value: u64
}

abi StoreU64 {
  fn store_u64(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, storage: StoreU64Request);
  fn get_u64(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, key: b256) -> u64;
}

