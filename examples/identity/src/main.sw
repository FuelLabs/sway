contract;

dep r#abi;
dep errors;

use r#abi::IdentityExample;
use errors::MyError;

use std::{
    address::Address,
    assert::require,
    chain::auth::{AuthError, msg_sender},
    constants::{BASE_ASSET_ID, ZERO_B256},
    contract_id::ContractId,
    identity::Identity,
    result::Result,
    revert::revert,
    token::{force_transfer_to_contract, transfer_to_output}
};

storage {
    owner: Identity = Identity::ContractId(~ContractId::from(ZERO_B256)),
}

impl IdentityExample for Contract {
    fn cast_to_identity() {
        // ANCHOR: cast_to_identity
        let raw_address: b256 = 0xddec0e7e6a9a4a4e3e57d08d080d71a299c628a46bc609aab4627695679421ca;
        let my_identity: Identity = Identity::Address(~Address::from(raw_address));
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
        // ANCHOR_END: identity_to_contract_id
    }

    fn different_executions(my_identity: Identity) {
        let amount = 1;
        let token_id = BASE_ASSET_ID;

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
