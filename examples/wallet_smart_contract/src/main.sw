// NOTE: Storage is a work in progress (see
// https://github.com/FuelLabs/sway/pull/646), but once it is implemented,
// declaring storage should look like this.

contract;

use std::*;
use std::chain::assert;

const OWNER_ADDRESS: b256 = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;
const ETH_ID: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

// storage {
//     balance: u64,
// }

abi Wallet {
    fn receive_funds(gas_to_forward: u64, coins_to_forward: u64, asset_id: b256, unused: ());
    fn send_funds(gas_to_forward: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest);
}

impl Wallet for Contract {
    fn receive_funds(gas_to_forward: u64, coins_to_forward: u64, asset_id: b256, unused: ()) {
        // if asset_id == ETH_ID {
        //     let balance = storage.balance.write();
        //     deref balance = balance + coins_to_forward;
        // };
    }

    fn send_funds(gas_to_forward: u64, coins_to_forward: u64, asset_id: b256, req: SendFundsRequest) {
        // assert(sender() == OWNER_ADDRESS);
        // assert(storage.balance > req.amount_to_send);
        // storage.balance = storage.balance - req.amount_to_send;
        // transfer_coins(asset_id, req.recipient_address, req.amount_to_send);
    }
}

struct SendFundsRequest {
    amount_to_send: u64,
    recipient_address: b256,
}
