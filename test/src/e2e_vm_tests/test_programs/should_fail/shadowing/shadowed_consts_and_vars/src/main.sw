script;

dep lib;

// const shadowing an imported const
use lib::X;
const X = 6; 

// const shadowing a local const
const Y = 7;
const Y = 8;
fn main() {
    // variable shadowing an imported const
    let X = 9;

    // variable shadowing a const
    const Z = 3;
    let Z = 4;

    // const shadowing a variable
    let W = 2;
    const W = 1;

    // Variable shadowing a variable - this is okay!
    let P = 7;
    let P = 8;
}
