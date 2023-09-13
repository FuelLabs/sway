contract;

abi MySuperAbi {
    fn superabi_test() -> u8;
}

abi MyContract : MySuperAbi {
    fn abi_test() -> u8;
}

impl MySuperAbi for Contract {
    fn superabi_test() -> u8 {
        41
    }
}

impl MyContract for Contract {
    fn abi_test() -> u8 {
        42
    }
}
