contract;

abi MyAbi
{
    // no interface methods
}
{
    fn method() -> u64 { 42 }
    fn method() -> u64 { 43 }    // error: duplicate impl method
}
