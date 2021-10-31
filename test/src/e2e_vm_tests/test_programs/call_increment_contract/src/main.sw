script;
use increment_abi::Incrementor;
use std::constants::ETH_COLOR;
fn main() {
  let abi = abi(Incrementor, 0x58c94181ce5b34028163958f9be8eb7a389e0d6aa8020762bea1aad6ea05a2ba);   
  abi.initialize(10000, 0, ETH_COLOR, 0); // comment this line out to just increment without initializing
  abi.increment(10000, 0, ETH_COLOR, 5);
  let result = abi.increment(10000, 0, ETH_COLOR, 5);
  log(result);
}

fn log(input: u64) {
  asm(r1: input, r2: 42) {
   log r1 r2 r2 r2; 
  }
}
