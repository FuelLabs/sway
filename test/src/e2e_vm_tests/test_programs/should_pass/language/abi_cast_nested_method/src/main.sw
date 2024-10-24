script;

abi AMM {
    fn pool() -> Option<ContractId>;
}
abi Exchange {
    fn swap_exact_output();
}

fn main() {
    let amm_contract = abi(AMM, 0x0000000000000000000000000000000000000000000000000000000000000000);

    let exchange_contract_id = amm_contract.pool();

    // let a =  exchange_contract_id.unwrap().into();
    // let exchange_contract = abi(Exchange, a);

    let exchange_contract = abi(Exchange, exchange_contract_id.unwrap().into());
    
    exchange_contract.swap_exact_output();
}