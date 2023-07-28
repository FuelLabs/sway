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

impl MyAbi for Contract {
    // this must fail, because contract methods cannot call each other
    fn abi_method() { Self::super_abi_method() }
}
