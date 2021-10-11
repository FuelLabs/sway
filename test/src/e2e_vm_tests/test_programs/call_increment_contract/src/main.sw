script;
use increment_abi::Incrementor;
use std::constants::ETH_COLOR;
fn main() {
  let abi = abi(Incrementor, 0xf804f1578bad017ad47b5d38e3930a041b680fab20a887e52ec077666247d3a5);   
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
