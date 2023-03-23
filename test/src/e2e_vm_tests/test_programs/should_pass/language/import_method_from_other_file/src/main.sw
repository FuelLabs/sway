script;

mod context;
mod asset;
mod utils;

use context::Context;
use utils::Wrapper;

fn eq_test() {
   let w1 = Wrapper::new(3);
   let w2 = Wrapper::new(3);

   assert(w1 == w2);
   assert(w1.asset == w2.asset);
}

fn main() -> u64 {
   eq_test();

   let x = Context::foo();
   x.something
}
