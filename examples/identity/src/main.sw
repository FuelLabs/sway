contract;

dep r#abi;
dep errors;

use abi::IdentityExample;
use errors::MyError;

use std::{
    chain::auth::{
        AuthError,
        msg_sender,
    },
    constants::{
        BASE_ASSET_ID,
        ZERO_B256,
    },
    token::{
        force_transfer_to_contract,
        transfer_to_address,
    },
};

storage {
    owner: Identity = Identity::ContractId(ContractId::from(ZERO_B256)),
}

impl IdentityExample for Contract {
    fn cast_to_identity() {
        // ANCHOR: cast_to_identity
        let raw_address: b256 = 0xddec0e7e6a9a4a4e3e57d08d080d71a299c628a46bc609aab4627695679421ca;
        let my_identity: Identity = Identity::Address(Address::from(raw_address));
        // ANCHOR_END: cast_to_identity
    }

    fn identity_to_contract_id(my_identity: Identity) {
        // ANCHOR: identity_to_contract_id
        let my_contract_id: ContractId = match my_identity {
            Identity::ContractId(identity) => identity,
            _ => revert(0),
        };
        // ANCHOR_END: identity_to_contract_id
    }

    fn different_executions(my_identity: Identity) {
        let amount = 1;
        let token_id = BASE_ASSET_ID;

        // ANCHOR: different_executions
        match my_identity {
            Identity::Address(address) => transfer_to_address(amount, token_id, address),
            Identity::ContractId(contract_id) => force_transfer_to_contract(amount, token_id, contract_id),
        };
        // ANCHOR_END: different_executions
    }

    #[storage(read)]
    fn access_control_with_identity() {
        // ANCHOR: access_control_with_identity
        let sender: Result<Identity, AuthError> = msg_sender();
        require(sender.unwrap() == storage.owner, MyError::UnauthorizedUser);
        // ANCHOR_END: access_control_with_identity
    }
}
