contract;

use find_associated_methods_library::MyContract;

use std::result::*;
use std::identity::*;
use std::chain::auth::*;
use std::option::*;
use std::assert::*;

fn bogus() -> Identity {
    let sender = msg_sender();
    sender.unwrap()
}

struct MyStruct {
    int_option: Option<u64>
}

storage {
    stored_struct: MyStruct,
}

impl MyContract for Contract {
    #[storage(read)]
    fn test_function() -> bool {
        let identity = bogus();
        let stored_struct = storage.stored_struct;
        let stored_option_in_struct = stored_struct.int_option;
        require(stored_option_in_struct.is_some(), "Error");
        true
    }
}