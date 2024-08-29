script;

mod wallet_abi;

use wallet_abi::Wallet;

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(Wallet, contract_address);
    let amount_to_send = 200;
    let recipient_address = Address::from(0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b);
    // calling a non-payable method with a non-zero amount `coins`
    // this must result in an error
    caller.send_funds {
        gas: 10000,
        coins: 42,
        asset_id: b256::zero(),
    }(amount_to_send, recipient_address);
}
