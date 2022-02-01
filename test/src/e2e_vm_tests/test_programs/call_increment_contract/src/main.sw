script;
use increment_abi::Incrementor;
use std::constants::ETH_ID;
fn main() {
  let abi = abi(Incrementor, 0x19a4738f92544ccf46d2de5b84e273507512e42058dd8efd652546f576ac8bc0);
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
