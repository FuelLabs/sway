script;

use std::u128::U128;

pub enum Error {
    Overflow: (),
}

fn main() {
    let x = U128 {
        upper: 0,
        lower: 0,
    };
    let cond = false;
    require(cond || (x < U128::from((1, 1)) || x == U128::from((1, 1))), Error::Overflow);
}
