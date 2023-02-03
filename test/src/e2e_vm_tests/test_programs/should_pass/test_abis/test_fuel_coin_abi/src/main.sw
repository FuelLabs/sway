library test_fuel_coin_abi;

abi TestFuelCoin {
    fn mint(mint_amount: u64);
    fn burn(burn_amount: u64);
    fn force_transfer(coins: u64, asset_id: ContractId, c_id: ContractId);
}
