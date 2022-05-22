contract;

use std::{
    address::Address,
    assert::require,
    chain::auth::{AuthError, Sender, msg_sender},
    context::{call_frames::{contract_id, msg_asset_id}, msg_amount, this_balance},
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
    fn get_balance() -> u64;
    fn get_user_data(user: Address) -> (bool, bool);
    fn get_state() -> u64;
}

// TODO: add enums back in when they are supported in storage and "matching" them is implemented
// enum State {
//     Void: (),
//     Pending: (),
//     Completed: (),
// }

enum Error {
    CannotReinitialize: (),
    DepositRequired: (),
    IncorrectAssetAmount: (),
    IncorrectAssetId: (),
    StateNotInitialized: (),
    StateNotPending: (),
    UnauthorizedUser: (),
    UserHasAlreadyDeposited: (),
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
    state: u64,
}

impl Escrow for Contract {
    /// Initializes the escrow with the users, the asset and amount of asset
    ///
    /// # Panics
    ///
    /// The function will panic when
    /// - The constructor is called more than once
    fn constructor(buyer: Address, seller: Address, asset: ContractId, asset_amount: u64) -> bool {
        // require(storage.state == State::Void, Error::CannotReinitialize);
        require(storage.state == 0, Error::CannotReinitialize);

        storage.asset_amount = asset_amount;
        storage.buyer = User {
            address: buyer, approved: false, deposited: false
        };
        storage.seller = User {
            address: seller, approved: false, deposited: false
        };
        storage.asset = asset;
        storage.state = 1;
        // storage.state = State::Pending;

        true
    }

    /// Updates the user state to indicate that they have deposited
    /// A successful deposit unlocks the approval functionality
    ///
    /// # Panics
    ///
    /// The function will panic when
    /// - The constructor has not been called to initialize
    /// - The user is not an authorized user that has been set in the constructor
    /// - The user deposits an asset that is not the specified asset in the constructor
    /// - The user sends an incorrect amount of the asset that has been specified in the constructor
    /// - The user deposits when they still have their previous deposit in the escrow
    fn deposit() -> bool {
        // require(storage.state == State::Pending, Error::StateNotPending);
        require(storage.state == 1, Error::StateNotPending);
        require(storage.asset == msg_asset_id(), Error::IncorrectAssetId);
        require(storage.asset_amount == msg_amount(), Error::IncorrectAssetAmount);

        let sender: Result<Sender, AuthError> = msg_sender();

        if let Sender::Address(address) = sender.unwrap() {
            require(address == storage.buyer.address || address == storage.seller.address, Error::UnauthorizedUser);

            if address == storage.buyer.address {
                require(!storage.buyer.deposited, Error::UserHasAlreadyDeposited);

                storage.buyer.deposited = true;
            } else if address == storage.seller.address {
                require(!storage.seller.deposited, Error::UserHasAlreadyDeposited);

                storage.seller.deposited = true;
            }
        } else {
            revert(0);
        };

        true
    }

    /// Updates the user state to indicate that they have approved
    /// Once both of the users approve the escrow will automatically transfers the assets back to the users
    ///
    /// # Panics
    ///
    /// The function will panic when
    /// - The constructor has not been called to initialize
    /// - The user is not an authorized user that has been set in the constructor
    /// - The user has not successfully deposited through the deposit() function
    /// - The user approves again after both users have approved and the escrow has completed its process
    fn approve() -> bool {
        // require(storage.state == State::Pending, Error::StateNotPending);
        require(storage.state == 1, Error::StateNotPending);

        let sender: Result<Sender, AuthError> = msg_sender();

        if let Sender::Address(address) = sender.unwrap() {
            require(address == storage.buyer.address || address == storage.seller.address, Error::UnauthorizedUser);

            if address == storage.buyer.address {
                require(storage.buyer.deposited, Error::DepositRequired);

                storage.buyer.approved = true;
            } else if address == storage.seller.address {
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

    /// Returns the deposited asset back to the user and resets their approval to false
    ///
    /// # Panics
    ///
    /// The function will panic when
    /// - The constructor has not been called to initialize
    /// - The user is not an authorized user that has been set in the constructor
    /// - The user has not successfully deposited through the deposit() function
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
            } else if address == storage.seller.address {
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

    /// Returns the amount of the specified asset in this contract
    fn get_balance() -> u64 {
        this_balance(storage.asset)
    }

    /// Returns data regarding the state of a user i.e. whether they have (deposited, approved)
    ///
    /// # Panics
    ///
    /// The function will panic when
    /// - The constructor has not been called to initialize
    /// - The user is not an authorized user that has been set in the constructor
    fn get_user_data(user: Address) -> (bool, bool) {
        // require(storage.state != State::Void, Error::StateNotInitialized);
        require(storage.state != 0, Error::StateNotInitialized);
        require(user == storage.buyer.address || user == storage.seller.address, Error::UnauthorizedUser);

        if user == storage.buyer.address {
            (storage.buyer.deposited, storage.buyer.approved)
        } else {
            (storage.seller.deposited, storage.seller.approved)
        }
    }

    /// Returns a value indicating the current state of the escrow
    ///
    /// # State
    ///
    /// 0 = The constructor has yet to be called to initialize the contract state
    /// 1 = The constructor has been called to initialize the contract and is pending the deposit & approval from both parties
    /// 2 = Both parties have deposited and approved and the escrow has completed its purpose
    fn get_state() -> u64 {
        storage.state
    }
}
