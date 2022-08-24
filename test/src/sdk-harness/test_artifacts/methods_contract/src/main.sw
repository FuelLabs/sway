contract;

use methods_abi::MethodsContract;

use std::identity::*;
use std::chain::auth::*;
use std::option::*;
use std::revert::*;

fn bogus() -> Identity {
    let sender = msg_sender();
    sender.unwrap()
}

fn bogus2() -> Identity {
    msg_sender().unwrap()
}

struct MyStruct {
    int_option: Option<u64>,
}

storage {
    stored_struct: MyStruct = MyStruct {
        int_option: Option::None,
    },
}

impl MethodsContract for Contract {
    #[storage(read, write)]fn test_function() -> bool {
        let identity = bogus();
        let identity2 = bogus2();
        storage.stored_struct = MyStruct {
            int_option: Option::Some(99u64), 
        };
        let stored_struct = storage.stored_struct;
        let stored_option_in_struct = stored_struct.int_option;
        require(stored_option_in_struct.is_some(), "Error");
        true
    }
}
