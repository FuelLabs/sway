contract;

use context_testing_abi::ContextTesting;
use std::{asset::mint, hash::*};

abi ContextCaller {
    fn call_get_this_balance_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_get_balance_of_contract_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_get_amount_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_get_asset_id_with_coins(send_amount: u64, context_id: ContractId) -> b256;
    fn call_get_gas_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_get_global_gas_with_coins(send_amount: u64, context_id: ContractId) -> u64;
    fn call_receive_coins(send_amount: u64, target: ContractId);
    fn mint_coins(mint_amount: u64);
}

impl ContextCaller for Contract {
    fn call_get_this_balance_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.bits();
        let context_contract = abi(ContextTesting, id);
        let asset_id = AssetId::default();
        let asset_id_b256: b256 = asset_id.into();

        context_contract.get_this_balance {
            gas: 500_000,
            coins: send_amount,
            asset_id: asset_id.into(),
        }(asset_id_b256)
    }

    fn call_get_balance_of_contract_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.bits();
        let context_contract = abi(ContextTesting, id);
        let asset_id = AssetId::default();
        let asset_id_b256: b256 = asset_id.into();

        context_contract.get_balance_of_contract {
            gas: 500_000,
            coins: send_amount,
            asset_id: asset_id.into(),
        }(asset_id_b256, target)
    }

    fn call_get_amount_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.bits();
        let context_contract = abi(ContextTesting, id);
        let asset_id = AssetId::default();

        context_contract.get_amount {
            gas: 500_000,
            coins: send_amount,
            asset_id: asset_id.into(),
        }()
    }

    fn call_get_asset_id_with_coins(send_amount: u64, target: ContractId) -> b256 {
        let id = target.bits();
        let context_contract = abi(ContextTesting, id);
        let asset_id = AssetId::default();

        context_contract.get_asset_id {
            gas: 500_000,
            coins: send_amount,
            asset_id: asset_id.into(),
        }()
    }

    fn call_get_gas_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.bits();
        let context_contract = abi(ContextTesting, id);
        let asset_id = AssetId::default();

        context_contract.get_gas {
            gas: 500_000,
            coins: send_amount,
            asset_id: asset_id.into(),
        }()
    }

    fn call_get_global_gas_with_coins(send_amount: u64, target: ContractId) -> u64 {
        let id = target.bits();
        let context_contract = abi(ContextTesting, id);
        let asset_id = AssetId::default();

        context_contract.get_global_gas {
            gas: 500_000,
            coins: send_amount,
            asset_id: asset_id.into(),
        }()
    }

    fn call_receive_coins(send_amount: u64, target: ContractId) {
        let id = target.bits();
        let context_contract = abi(ContextTesting, id);
        let asset_id = AssetId::default();

        context_contract
            .receive_coins {
                gas: 500_000,
                coins: send_amount,
                asset_id: asset_id.into(),
            }();
    }

    fn mint_coins(mint_amount: u64) {
        mint(b256::zero(), mint_amount)
    }
}
