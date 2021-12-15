script;
use increment_abi::Incrementor;
use std::constants::ETH_ID;
fn main() {
  let abi = abi(Incrementor, 0xf7f9d5f37723833e266ff185bd3ded8af980f50703b1719dd47a446af2dabc70);
  abi.initialize(10000, 0, ETH_ID, 0); // comment this line out to just increment without initializing
  abi.increment(10000, 0, ETH_ID, 5);
  let result = abi.increment(10000, 0, ETH_ID, 5);
  log(result);
}

fn log(input: u64) {
  asm(r1: input, r2: 42) {
   log r1 r2 r2 r2;
  }
}
