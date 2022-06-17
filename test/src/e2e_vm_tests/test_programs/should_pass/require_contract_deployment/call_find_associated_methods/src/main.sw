script;

use find_associated_methods_library::*;
use std::assert::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x8afb04df8c2b85db4b33550a7b736795a6a687303ca58f48aa98a487bfda91a9);

    let res = the_abi.test_function();
    assert(res);

    true
}
