// Inheritance graph
//          MySuperAbi1          MySuperAbi2
//             \                      /
//                      MyAbi

contract;

abi MySuperAbi1 {
    fn method();
}

abi MySuperAbi2 {
    fn method();
}

// For now we forbid ABIs to inherit methods with the same name
// from their superABIs
abi MyAbi : MySuperAbi1 + MySuperAbi2 {
    fn method2();
}
