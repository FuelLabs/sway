// Inheritance graph
//          MySuperAbi1   MySuperAbi2          MySuperTrait
//              \              /                  /
//                      MyAbi

contract;

trait MySuperTrait {
    fn method() -> u64;
}

abi MySuperAbi1 {
    fn method() -> u64;
}

abi MySuperAbi2 {
    fn method() -> u64;
}

abi MyAbi : MySuperAbi1 + MySuperTrait + MySuperAbi2 {
    fn method1() -> u64;
}
