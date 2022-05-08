script;

use std::assert::require;

fn main() -> bool {
    const MY_CUSTOM_ERROR_MESSAGE = 100;

    let a = 5;
    let b = 5;
    let c = 6;

    require(1 + 1 == 2, 11);
    require(true == true, 42);
    require(false != true, 0);
    require(7 > 5, 3);
    require(a == b, MY_CUSTOM_ERROR_MESSAGE);

    true
}
