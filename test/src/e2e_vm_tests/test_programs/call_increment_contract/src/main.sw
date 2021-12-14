script;
use increment_abi::Incrementor;
use std::constants::ETH_ID;
fn main() {
  let abi = abi(Incrementor, 0x15da979ac3c6636e3d0dd094d0acd173be9e2bc384f60f5006aedb68d7425687);
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
