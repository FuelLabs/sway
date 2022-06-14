predicate;

use std::tx::get_predicate_data;

fn main() -> bool {
    let received: b256 = get_predicate_data();
    let expected: b256 = 0xef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a;

    received == expected
}
