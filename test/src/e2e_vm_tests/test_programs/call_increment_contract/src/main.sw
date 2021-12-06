script;
use increment_abi::Incrementor;
use std::constants::ETH_ID;
fn main() {
  let abi = abi(Incrementor, 0x1c1034f66a300fb0af4d3dcfba823e6237d329ce823f4e2a7f517b330ea5e875);
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
