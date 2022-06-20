library methods_abi;

abi MethodsContract {
    #[storage(read, write)]
    fn test_function() -> bool;
}
