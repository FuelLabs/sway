library;

abi Abi {
    #[payable]
    fn ok_1();

    #[payable()]
    fn ok_2();

    #[payable(invalid)]
    fn not_ok_1();
}