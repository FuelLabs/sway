contract;

use std::{
    address::Address,
    assert::require,
    chain::auth::{AuthError, Sender, msg_sender},
    context::{msg_amount, call_frames::{contract_id, msg_asset_id}},
    contract_id::ContractId,
    result::*,
    revert::revert,
    token::transfer_to_output,
};

abi Escrow {
    fn constructor(buyer: Address, seller: Address, asset: ContractId, asset_amount: u64) -> bool;
    fn deposit() -> bool;
    fn approve() -> bool;
    fn withdraw() -> bool;
}

// TODO: add enums back in when they are supported in storage and "matching" them is implemented
// enum State {
//     Void: (),
//     Pending: (),
//     Completed: (),
// }

enum Error {
    AlreadyDeposited: (),
    CannotReinitialize: (),
    DepositRequired: (),
    IncorrectAssetAmount: (),
    IncorrectAssetId: (),
    StateNotPending: (),
    UnauthorizedUser: (),
}

struct User {
    address: Address,
    approved: bool,
    deposited: bool,
}

storage {
    asset_amount: u64,
    buyer: User,
    seller: User,
    asset: ContractId,
    // state: State,
    state: u64
}

impl Escrow for Contract {

    fn constructor(buyer: Address, seller: Address, asset: ContractId, asset_amount: u64) -> bool {
        // require(storage.state == State::Void, Error::CannotReinitialize);
        require(storage.state == 0, Error::CannotReinitialize);

        storage.asset_amount = asset_amount;
        storage.buyer = User { address: buyer, approved: false, deposited: false };
        storage.seller = User { address: seller, approved: false, deposited: false };
        storage.asset = asset;
        storage.state = 1;
        // storage.state = State::Pending;

        true
    }

    fn deposit() -> bool {
        // require(storage.state == State::Pending, Error::StateNotPending);
        require(storage.state == 1, Error::StateNotPending);
        require(storage.asset == msg_asset_id(), Error::IncorrectAssetId);
        require(storage.asset_amount == msg_amount(), Error::IncorrectAssetAmount);

        let sender: Result<Sender, AuthError> = msg_sender();

        if let Sender::Address(address) = sender.unwrap() {
            require(address == storage.buyer.address || address == storage.seller.address, Error::UnauthorizedUser);

            if address == storage.buyer.address {
                require(!storage.buyer.deposited, Error::AlreadyDeposited);

                storage.buyer.deposited = true;
            }
            else if address == storage.seller.address {
                require(!storage.seller.deposited, Error::AlreadyDeposited);

                storage.seller.deposited = true;
            }
        } else {
            revert(0);
        };

        true
    }

    fn approve() -> bool {
        // require(storage.state == State::Pending, Error::StateNotPending);
        require(storage.state == 1, Error::StateNotPending);

        let sender: Result<Sender, AuthError> = msg_sender();

        if let Sender::Address(address) = sender.unwrap() {
            require(address == storage.buyer.address || address == storage.seller.address, Error::UnauthorizedUser);

            if address == storage.buyer.address {
                require(storage.buyer.deposited, Error::DepositRequired);

                storage.buyer.approved = true;
            } 
            else if address == storage.seller.address {
                require(storage.seller.deposited, Error::DepositRequired);

                storage.seller.approved = true;
            }

            if storage.buyer.approved && storage.seller.approved {
                // storage.state = State::Completed;
                storage.state = 2;

                transfer_to_output(storage.asset_amount, storage.asset, storage.buyer.address);
                transfer_to_output(storage.asset_amount, storage.asset, storage.seller.address);
            }
        } else {
            revert(0);
        };

        true
    }

    fn withdraw() -> bool {
        // require(storage.state == State::Pending, Error::StateNotPending);
        require(storage.state == 1, Error::StateNotPending);

        let sender: Result<Sender, AuthError> = msg_sender();

        if let Sender::Address(address) = sender.unwrap() {
            require(address == storage.buyer.address || address == storage.seller.address, Error::UnauthorizedUser);

            if address == storage.buyer.address {
                require(storage.buyer.deposited, Error::DepositRequired);

                storage.buyer.deposited = false;
                storage.buyer.approved = false;

                transfer_to_output(storage.asset_amount, storage.asset, storage.buyer.address);
            } 
            else if address == storage.seller.address {
                require(storage.seller.deposited, Error::DepositRequired);

                storage.seller.deposited = false;
                storage.seller.approved = false;

                transfer_to_output(storage.asset_amount, storage.asset, storage.seller.address);
            }
        } else {
            revert(0);
        };

        true
    }

}
