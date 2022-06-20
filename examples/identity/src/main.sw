contract;

dep abi;
dep errors;

use abi::IdentityExample;
use errors::MyError;

use std::{
    address::Address,
    assert::require,
    chain::auth::{AuthError, msg_sender},
    constants::BASE_ASSET_ID,
    contract_id::ContractId,
    identity::*,
    result::*,
    revert::revert,
    token::{force_transfer_to_contract, transfer_to_output}
};

storage {
    owner: Identity,
}

impl IdentityExample for Contract {
    fn cast_to_identity() {
        // ANCHOR: cast_to_identity
        let my_address: Address = ~Address::from(BASE_ASSET_ID);
        let my_identity: Identity = Identity::Address(my_address);
        // ANCHOR_END: cast_to_identity
    }

    fn identity_to_contract_id(my_identity: Identity) {
        // ANCHOR: identity_to_contract_id
        let my_contract_id: ContractId = match my_identity {
            Identity::ContractId(identity) => {
                identity
            },
            _ => {
                revert(0);
            }
        };
        // ANCHOR_END: identity_to_contract_ids
    }

    fn different_executions(my_identity: Identity) {
        let amount = 1;
        let token_id = ~ContractId::from(BASE_ASSET_ID);

        // ANCHOR: different_executions
        match my_identity {
            Identity::Address(identity) => {
                transfer_to_output(amount, token_id, identity);
            },
            Identity::ContractId(identity) => {
                force_transfer_to_contract(amount, token_id, identity);
            },
        };
        // ANCHOR_END: different_executions
    }

    #[storage(read)]fn access_control_with_identity() {
        // ANCHOR: access_control_with_identity
        let sender: Result<Identity, AuthError> = msg_sender();
        require(sender.unwrap() == storage.owner, MyError::UnauthorizedUser);
        // ANCHOR_END: access_control_with_identity
    }
}
