script;

dep lib;

// const shadowing an imported const with alias
use lib::X as Y;
const Y = 7;

fn main() {
    // var shadowing an imported const with alias
    let Y = 4;
}
