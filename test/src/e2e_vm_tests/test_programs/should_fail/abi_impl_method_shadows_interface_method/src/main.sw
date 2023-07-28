contract;

abi MyAbi
{
    fn method() -> u64;
}
{
    // impl methods cannot shadow interface methods
    fn method() -> u64 { 42 }
}
