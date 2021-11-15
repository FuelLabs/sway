script;

use std::address::Address;

fn main() -> bool {
    let bits = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;
    let addr = ~Address::from_b256(bits);
    addr.inner == bits
}
