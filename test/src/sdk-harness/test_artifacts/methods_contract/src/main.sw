contract;

use methods_abi::MethodsContract;

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
        int_option: None,
    },
}

impl MethodsContract for Contract {
    #[storage(read, write)]
    fn test_function() -> bool {
        let _ = bogus();
        let _ = bogus2();
        storage
            .stored_struct
            .write(MyStruct {
                int_option: Some(99u64),
            });
        let stored_struct = storage.stored_struct.read();
        let stored_option_in_struct = stored_struct.int_option;
        require(stored_option_in_struct.is_some(), "Error");
        true
    }
}
