contract;

abi MyAbi
{
    fn interface_method();
}
{
    fn impl_method() { }
}

impl MyAbi for Contract {
    fn interface_method() { }
}
