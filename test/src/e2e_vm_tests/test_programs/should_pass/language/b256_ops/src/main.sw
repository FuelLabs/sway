script;

use std::hash::{keccak256, sha256};

fn main() -> u64 {
    let aaaa = 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;
    let aaab = 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa_b;
    let abaa = 0xa_b_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;
    if aaaa == aaab {
        0
    } else if aaaa == abaa {
        1
    } else if !(aaaa == aaaa) {
        2
    } else if !(sha256(aaaa) == 0xe0e77a507412b120f6ede61f62295b1a7b2ff19d3dcc8f7253e51663470c888e) {
        3
    } else if !(keccak256(aaaa) == 0x20ee8f1366f06926e9e8771d8fb9007a8537c8dfdb6a3f8c2cfd64db19d2ec90) {
        4
    } else if !(sha256((aaaa, abaa)) == 0xa4bca8eb8f338f7fda26960fa43bfe34fbc562e2ee0d7c6e8856c1c587f215ce) {
        5
    } else if !(keccak256((aaaa, abaa)) == 0x4fce5a297040d82eecf7b0ae4855ad43698f191ee38820e27748648765bc42bd) {
        6
    } else {
        100
    }
}
