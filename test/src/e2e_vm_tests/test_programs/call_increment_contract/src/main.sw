script;
use increment_abi::Incrementor;
// using the below constant throws an error, see https://github.com/FuelLabs/sway/issues/297
// use std::constants::ETH_COLOR;
const ETH_COLOR = 0x0000000000000000000000000000000000000000000000000000000000000000;
fn main() {
  let abi = abi(Incrementor, 0x2748b9ae7ea005e7cbf3e65e3bb03850cbb0bfdf8e8b3261f74be7ce01eff516);   
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
