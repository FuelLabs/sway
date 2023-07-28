// Inheritance graph:
//          MySuperAbi
//              |
//            MyAbi

contract;

abi MySuperAbi {
    fn method();
}

// For now we forbid ABIs to have methods with the same name
// as their superABIs
abi MyAbi : MySuperAbi {
    fn method();
}
