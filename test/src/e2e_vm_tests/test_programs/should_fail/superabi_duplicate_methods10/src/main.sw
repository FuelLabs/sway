// Inheritance graph
//          MySuperSuperAbi1          MySuperSuperAbi2
//                |                           |
//          MySuperAbi1               MySuperAbi2
//                \                          /
//                          MyAbi

contract;

abi MySuperSuperAbi1
{
}
{
    fn method() {}
}

abi MySuperSuperAbi2
{
}
{
    fn method() {}
}

abi MySuperAbi1: MySuperSuperAbi1 {
}

abi MySuperAbi2: MySuperSuperAbi2 {
}

// For now we forbid ABIs to inherit methods with the same name
// from their superABIs (transitively)
abi MyAbi : MySuperAbi1 + MySuperAbi2 {
    fn method2();
}
