library interface;

abi Vault {
    fn deposit();
    fn withdraw(amount: u64, asset: ContractId);
}
