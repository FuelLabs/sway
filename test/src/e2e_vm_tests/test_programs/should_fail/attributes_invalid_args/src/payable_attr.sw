library;

abi Abi {
    #[payable]
    fn ok_1();

    #[payable(invalid)] // Should be no invalid arg error or warning here.
    fn also_ok();
}