script;
use increment_abi::Incrementor;
use std::constants::ETH_COLOR;
fn main() {
  let abi = abi(Incrementor, 0xe50103684750e4916cd9825b14cf7e6763ffcc6523a9e0af63de93dbd6e3d736);   
  //abi.initialize(10000, 0, ETH_COLOR, 0); // comment this line out to just increment without initializing
  let result = abi.increment(10000, 0, ETH_COLOR, 42);
  log(result);
}

fn log(input: u64) {
  asm(r1: input, r2: 42) {
   log r1 r2 r2 r2; 
  }
}
