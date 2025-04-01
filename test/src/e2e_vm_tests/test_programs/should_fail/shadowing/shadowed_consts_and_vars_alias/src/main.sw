script;

mod lib;

// const shadowing an imported const with alias
use lib::X as Y;
const Y: u64 = 7;

use lib::L as M;

fn main() {
    // var shadowing an imported const with alias
    let Y = 4;

    let M = 4;

    use lib::P as R;
    let R = 5;
}
