// Inheritance graph
//          MySuperSuperAbi
//                |
//            MySuperAbi
//                |
//              MyAbi

contract;

abi MySuperSuperAbi
{
}
{
    fn method1() {}
}

abi MySuperAbi : MySuperSuperAbi {
    fn method2();
}

// For now we forbid ABIs to have methods with the same name
// as their superABIs
abi MyAbi : MySuperAbi {
    fn method1();
}
