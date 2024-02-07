// This test proves that https://github.com/FuelLabs/sway/issues/5598 is fixed.
script; 

mod lib_a;
mod lib_b;

use ::lib_a::E as Lib_A_E;
use ::lib_a::S as Lib_A_S;

use ::lib_b::E as Lib_B_E;
use ::lib_b::S as Lib_B_S;

fn main() {
  let _: Lib_A_E = Lib_B_E::X(123);
  let _: Lib_A_S = Lib_B_S { x: 123 };
}
