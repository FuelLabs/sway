contract;

use std::{
    address::Address,
    assert::assert,
    chain::auth::{AuthError, Sender, msg_sender},
    context::{msg_amount, call_frames::{contract_id, msg_asset_id}},
    contract_id::ContractId,
    panic::panic,
    result::*,
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
        // assert(storage.state == State::Void);
        assert(storage.state == 0);

        storage.asset_amount = asset_amount;
        storage.buyer = User { address: buyer, approved: false, deposited: false };
        storage.seller = User { address: seller, approved: false, deposited: false };
        storage.asset = asset;
        storage.state = 1;
        // storage.state = State::Pending;

        true
    }

    fn deposit() -> bool {
        // assert(storage.state == State::Pending);
        assert(storage.state == 1);
        assert(storage.asset == msg_asset_id());
        assert(storage.asset_amount == msg_amount());

        let sender: Result<Sender, AuthError> = msg_sender();

        if let Sender::Address(address) = sender.unwrap() {
            assert(address == storage.buyer.address || address == storage.seller.address);

            if address == storage.buyer.address {
                assert(!storage.buyer.deposited);

                storage.buyer.deposited = true;
            }
            else if address == storage.seller.address {
                assert(!storage.seller.deposited);

                storage.seller.deposited = true;
            }
        } else {
            panic(0);
        };

        true
    }

    fn approve() -> bool {
        // assert(storage.state == State::Pending);
        assert(storage.state == 1);

        let sender: Result<Sender, AuthError> = msg_sender();

        if let Sender::Address(address) = sender.unwrap() {
            assert(address == storage.buyer.address || address == storage.seller.address);

            if address == storage.buyer.address {
                assert(storage.buyer.deposited);

                storage.buyer.approved = true;
            } 
            else if address == storage.seller.address {
                assert(storage.seller.deposited);

                storage.seller.approved = true;
            }

            if storage.buyer.approved && storage.seller.approved {
                // storage.state = State::Completed;
                storage.state = 2;

                transfer_to_output(storage.asset_amount, storage.asset, storage.buyer.address);
                transfer_to_output(storage.asset_amount, storage.asset, storage.seller.address);
            }
        } else {
            panic(0);
        };

        true
    }

    fn withdraw() -> bool {
        // assert(storage.state == State::Pending);
        assert(storage.state == 1);

        let sender: Result<Sender, AuthError> = msg_sender();

        if let Sender::Address(address) = sender.unwrap() {
            assert(address == storage.buyer.address || address == storage.seller.address);

            if address == storage.buyer.address {
                assert(storage.buyer.deposited);

                storage.buyer.deposited = false;
                storage.buyer.approved = false;

                transfer_to_output(storage.asset_amount, storage.asset, storage.buyer.address);
            } 
            else if address == storage.seller.address {
                assert(storage.seller.deposited);

                storage.seller.deposited = false;
                storage.seller.approved = false;

                transfer_to_output(storage.asset_amount, storage.asset, storage.seller.address);
            }
        } else {
            panic(0);
        };

        true
    }

}
