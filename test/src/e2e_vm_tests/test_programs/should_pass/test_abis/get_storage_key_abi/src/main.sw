library get_storage_key_abi;

abi TestContract {
    fn from_f1() -> b256;
    fn from_f2() -> b256;
    fn from_f3() -> b256;
    fn from_f4() -> b256;
    fn from_callers() -> (b256, b256, b256, b256);
}
