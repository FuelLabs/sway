# Wallet Smart Contract
_Contract storage in the language syntax is a work-in-progress feature, and the following example does not currently compile._
```sway
contract;

const OWNER_ADDRESS: b256 = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;
const ETH_COLOR: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
use std::*;

abi Wallet {
    storage balance: u64 = 0;
    fn receive_funds(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, unused: ());
    fn send_funds(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, req: SendFundsRequest);
}

impl Wallet for Contract {
    fn receive_funds(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, unused: ()) {
        if color_of_coins == ETH_COLOR {
            balance += coins_to_forward;
        };
    }

    fn send_funds(gas_to_forward: u64, coins_to_forward: u64, color_of_coins: b256, req: SendFundsRequest) {
        assert(sender() == OWNER_ADDRESS);
        assert(balance > req.amount_to_send);
        balance -= req.amount_to_send;
        transfer_coins(color_of_coins, req.recipient_address, req.amount_to_send);
    }
}

struct SendFundsRequest {
    amount_to_send: u64,
    recipient_address: b256,
}
```
