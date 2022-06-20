script;

use find_associated_methods_library::*;
use std::assert::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0xd3883e70d14edf2505c888b46045532ae7a01fd682e6fe111cecbaf2668ba01e);

    let res = the_abi.test_function();
    assert(res);

    true
}
