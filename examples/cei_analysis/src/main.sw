contract;

mod other_contract;

use other_contract::*;
use std::hash::*;

abi MyContract {
    #[storage(read, write)]
    fn withdraw(external_contract_id: ContractId);
}

storage {
    balances: StorageMap<Identity, u64> = StorageMap::<Identity, u64> {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn withdraw(external_contract_id: ContractId) {
        let sender = msg_sender().unwrap();
        let bal = storage.balances.get(sender).try_read().unwrap_or(0);

        assert(bal > 0);

        // External call
        let caller = abi(OtherContract, external_contract_id.into());
        caller.external_call { coins: bal }();

        // Storage update _after_ external call
        storage.balances.insert(sender, 0);
    }
}
