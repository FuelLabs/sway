// Inheritance graph
//          MySuperAbi          MySuperTrait
//              \                    /
//                      MyAbi

contract;

trait MySuperTrait {
    fn method() -> u64;
}

abi MySuperAbi {
    fn method() -> u64;
}

abi MyAbi : MySuperAbi + MySuperTrait {
    fn method1() -> u64;
}

impl MySuperTrait for Contract {
    fn method() -> u64 {
        42
    }
}

impl MySuperAbi for Contract {
    fn method() -> u64 {
        0xBAD
    }
}

impl MyAbi for Contract {
    // should return 42 (Self::method should resolve to MySuperTrait::method)
    fn method1() -> u64 {
        Self::method()
    }
}
