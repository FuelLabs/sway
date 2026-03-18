library;

abi Vault {
    #[payable]
    fn deposit();
    fn withdraw(amount: u64, asset: ContractId);
}
