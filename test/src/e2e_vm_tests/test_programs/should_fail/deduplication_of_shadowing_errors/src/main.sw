// This test proves that https://github.com/FuelLabs/sway/issues/5502 is fixed.
script;

mod lib;

use lib::Struct;
use lib::Struct;
use lib::Struct;

const X: u64 = 0;

fn main() -> () {
    let X = 1;

    let y = 3;

    {
        const y: u64 = 4;
    }

    {
        const y: u64 = 6;
    }
}

fn var_shadows_const_x() {
    let X = 3;
}

fn generic<T, T, T>(_x: T) { }