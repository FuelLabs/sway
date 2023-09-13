contract;

abi MySuperAbi {
    fn super_abi_method();
}

abi MyAbi : MySuperAbi {
    fn abi_method();
}

// The implementation of MyAbi for Contract must also implement MySuperAbi
// impl MySuperAbi for Contract {
//   ...
// }

impl MyAbi for Contract {
    fn abi_method() { }
}
