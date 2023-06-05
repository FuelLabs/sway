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

impl MyContract for Contract {
    #[storage(read, write)]
    fn withdraw(external_contract_id: ContractId) {
        let sender = msg_sender().unwrap();
        let bal = storage.balances.get(sender);

        assert(bal > 0);

        // External call
        let caller = abi(OtherContract, external_contract_id.into());
        caller.non_payable_method { coins: bal }();

        // Storage update _after_ external call
        storage.balances.insert(sender, 0);
    }
}
