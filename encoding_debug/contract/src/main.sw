contract;

abi MyContract {
    fn accept_string_and_return_content(arg: String) -> [u64; 3];
}

use std::string::String;

impl MyContract for Contract {
    fn accept_string_and_return_content(arg: String) -> [u64; 3] {
    let b = arg.as_bytes();
    let a1 = b.get(0).unwrap();
    let b1 = b.get(1).unwrap();
    let c1 = b.get(2).unwrap();
      [a1, b1, c1]
    }
}
