// Inheritance graph:
//          MySuperAbi
//              |
//            MyAbi

contract;

abi MySuperAbi
{
}
{
    fn method() {}
}

abi MyAbi : MySuperAbi
{
}
{
    fn method() {}
}

