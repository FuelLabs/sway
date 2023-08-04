contract;

abi MySuperAbi {
    fn super_abi_method();
}

abi MyAbi : MySuperAbi {
    fn abi_method();
} {
    // this must fail, because contract methods cannot call each other
    fn foo() { Self::super_abi_method() }
}
