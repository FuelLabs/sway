contract;

dep folder1/dep_1;
dep folder2/dep_2;

use dep_1::*;
use dep_2::*;

abi MyContract {
    fn function(
        arg1: MyStruct1,
        arg2: MyStruct2,
        arg3: Option<u64>,
    ) -> str[6];
}

impl MyContract for Contract {
    fn function(
        arg1: MyStruct1,
        arg2: MyStruct2,
        arg3: Option<u64>,
    ) -> str[6] {
        "fuel42"
    }
}
