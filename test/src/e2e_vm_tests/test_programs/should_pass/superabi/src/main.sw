contract;

abi MySuperAbi {
    fn super_abi_method();
}

abi MyAbi : MySuperAbi {
    fn abi_method();
}

impl MySuperAbi for Contract {
    fn super_abi_method() { }
}

// The implementation of MyAbi for Contract must also implement MySuperAbi
impl MyAbi for Contract {
    fn abi_method() { }
}
