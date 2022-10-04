contract;

use context_testing_abi::ContextTesting;
use std::{context::call_frames::contract_id, token::mint};

abi ContextCaller {
    fn call_get_this_balance_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_get_balance_of_contract_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_get_amount_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_get_asset_id_with_coins(send_amount: u64, context_id: ContractId) -> ContractId;
    fn call_get_gas_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_get_global_gas_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_receive_coins(send_amount: u64, target: ContractId);
    fn mint_coins(mint_amount: u64);
}

impl ContextCaller for Contract {
    fn call_get_this_balance_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.value;
        let context_contract = abi(ContextTesting, id);

        context_contract.get_this_balance {
            gas: 500_000,
            coins: send_amount,
            asset_id: ~ContractId::into(contract_id()),
        }(contract_id())
    }

    fn call_get_balance_of_contract_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.value;
        let context_contract = abi(ContextTesting, id);

        context_contract.get_balance_of_contract {
            gas: 500_000,
            coins: send_amount,
            asset_id: ~ContractId::into(contract_id()),
        }(contract_id(), target)
    }

    fn call_get_amount_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.value;
        let context_contract = abi(ContextTesting, id);

        context_contract.get_amount {
            gas: 500_000,
            coins: send_amount,
            asset_id: ~ContractId::into(contract_id()),
        }()
    }

    fn call_get_asset_id_with_coins(send_amount: u64, target: ContractId) -> ContractId {
        let id = target.value;
        let context_contract = abi(ContextTesting, id);

        context_contract.get_asset_id {
            gas: 500_000,
            coins: send_amount,
            asset_id: ~ContractId::into(contract_id()),
        }()
    }

    fn call_get_gas_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.value;
        let context_contract = abi(ContextTesting, id);

        context_contract.get_gas {
            gas: 500_000,
            coins: send_amount,
            asset_id: ~ContractId::into(contract_id()),
        }()
    }

    fn call_get_global_gas_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.value;
        let context_contract = abi(ContextTesting, id);

        context_contract.get_global_gas {
            gas: 500_000,
            coins: send_amount,
            asset_id: ~ContractId::into(contract_id()),
        }()
    }

    fn call_receive_coins(send_amount: u64, target: ContractId) {
        let id = target.value;
        let context_contract = abi(ContextTesting, id);

        context_contract.receive_coins {
            gas: 500_000,
            coins: send_amount,
            asset_id: ~ContractId::into(contract_id()),
        }();
    }

    fn mint_coins(mint_amount: u64) {
        mint(mint_amount)
    }
}
