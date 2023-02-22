library interface;

dep data_structures;

use data_structures::MyStruct;

abi MyContract {
    #[storage(read)]
    fn test_function() -> Option<MyStruct>;
}