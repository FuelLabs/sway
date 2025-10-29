contract;

mod other_contract;

use other_contract::*;

abi MyContract {
    #[storage(read, write)]
    fn withdraw(external_contract_id: ContractId);
}

storage {
    balances: StorageMap<Identity, u64> = StorageMap {},
}

fn call_using_const_generics_as_coins<const N: u64>(external_contract_id: ContractId) {
    let caller = abi(OtherContract, external_contract_id.into());
    let _ = caller.non_payable_method { coins: N }();
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn withdraw(external_contract_id: ContractId) {
        let sender = msg_sender().unwrap();
        let bal = storage.balances.get(sender);

        assert(bal > 0);

        // External call
        let caller = abi(OtherContract, external_contract_id.into());
        caller.non_payable_method { coins: bal }();

        // External call using const generics
        call_using_const_generics_as_coins::<0>(external_contract_id);

        // Storage update _after_ external call
        storage.balances.insert(sender, 0);
    }
}
