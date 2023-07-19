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
    fn superabi_method() {}
}

impl MyAbi for Contract {
    fn abi_method() {}
}

fn main() {
    let caller = abi(MyAbi, 0x0000000000000000000000000000000000000000000000000000000000000000);
    caller.abi_method();
    // superABI methods become ABI methods too
    caller.superabi_method()
}
