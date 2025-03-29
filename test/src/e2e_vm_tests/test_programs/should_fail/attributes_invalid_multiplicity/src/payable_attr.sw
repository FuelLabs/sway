library;

abi Abi {
    #[payable]
    fn ok();

    #[payable]
    #[payable, payable]
    #[payable]
    fn not_ok();
}