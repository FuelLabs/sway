contract;

struct MyStruct {
    x: u8,
}

struct MyConstStruct {
    x: u8,
}

const MY_CONST_STRUCT: MyConstStruct = MyConstStruct {};

storage {
    my_struct: MyStruct = MyStruct {},
}

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        let s = MyStruct { x: 0 };
        poke(s.x);

        let s = MyConstStruct { x: 0 };
        poke(s.x);

        poke(MY_CONST_STRUCT);
        
        true
    }
}

fn poke<T>(_x: T) {}
