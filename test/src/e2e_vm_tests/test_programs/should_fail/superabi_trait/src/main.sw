contract;

abi MySuperAbi {
    fn super_abi_method();
}

// error: traits cannot have superABIs
trait MyAbi : MySuperAbi {
    fn abi_method();
}
