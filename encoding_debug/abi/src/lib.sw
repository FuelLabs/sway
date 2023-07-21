library;


// anything `pub` here will be exported as a part of this library's API

use std::string::String;

abi MyContract {
    fn accept_string_and_return_content(arg: String) -> [u64; 3];
}

