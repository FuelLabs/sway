// Inheritance graph
//          MySuperAbi
//              |
//            MyAbi

script;

abi MySuperAbi {
    fn superabi_method();
}

abi MyAbi : MySuperAbi {
    fn abi_method();
}

impl MySuperAbi for Contract {
    fn superabi_method() { }
}

impl MyAbi for Contract {
    fn abi_method() { }
}

fn main() -> u64 {
    let caller = abi(MyAbi, 0x0000000000000000000000000000000000000000000000000000000000000000);
    caller.superabi_method();
    caller.abi_method();
    0
}
