script;

mod lib;

// const shadowing an imported const
use lib::X;
const X = 6; 

// const shadowing a local const
const Y = 7;
const Y = 8;

const C = 1;

use lib::L;

fn main() {
    // variable shadowing an imported const
    let X = 9;
    {
        let X = 3; // no error message here
    }

    // variable shadowing a const
    const Z = 3;
    let Z = 4;

    // const shadowing a variable
    let W = 2;
    const W = 1;

    // variable shadowing a variable - this is okay!
    let P = 7;
    let P = 8;

    // variable shadowing a global const
    let Y = 10;

    {
        // scoped variable shadowing a global const
        let C = 2;
    }

    let A = 1;
    {
        // scoped const shadowing a variable
        const A = 2;
    }

    const B = 1;
    {
        // scoped variable shadowing a const
        let B = 2;
    }

    {
        // scoped variable shadowing imported const
        let L = 1;
    }

    use lib::R;
    let R = 1;
}
