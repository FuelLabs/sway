script;

use find_associated_methods_library::MyContract;
use std::assert::*;

fn main() -> bool {
    let the_abi = abi(MyContract, 0x4b0e0324f65fc5440440962c0d13352dff4d5d358890427b5af36ee86ecdc221);

    let res = the_abi.test_function();
    assert(res);

    1
}
