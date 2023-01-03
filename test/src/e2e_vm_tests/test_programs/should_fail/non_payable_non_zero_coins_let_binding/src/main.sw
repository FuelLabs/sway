script;

dep wallet_abi;

use std::constants::ZERO_B256;
use wallet_abi::Wallet;

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(Wallet, contract_address);
    let amount_to_send = 200;
    let recipient_address = Address::from(0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b);
    // calling a non-payable method with a non-zero amount `coins`
    // this must result in an error
    // we use a chain of let-bindings that ends in a non-zero literal
    let fortytwo = 42;
    let coins = fortytwo;
    caller.send_funds {
        gas: 10000,
        coins: coins,
        asset_id: ZERO_B256,
    }(amount_to_send, recipient_address);
}
