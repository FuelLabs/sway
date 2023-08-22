library;

abi TestFuelCoin {
    fn mint(mint_amount: u64);
    fn burn(burn_amount: u64);
    fn force_transfer(coins: u64, asset_id: AssetId, c_id: ContractId);
}
