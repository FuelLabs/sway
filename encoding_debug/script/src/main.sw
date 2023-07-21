script;

use abi::MyContract;
use std::string::String;

fn main() {
    let call_me = abi(MyContract, CONTRACT_ID);
    let arg = String::from_ascii_str("abc");
    let ret = call_me.accept_string_and_return_content(arg);
    ()
}
