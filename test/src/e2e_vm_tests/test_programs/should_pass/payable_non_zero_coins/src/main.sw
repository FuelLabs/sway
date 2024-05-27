script;

mod wallet_abi;

use wallet_abi::Wallet;

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(Wallet, contract_address);
    // `receive_funds` is payable, this should pass
    caller.receive_funds {
        gas: 10000,
        coins: 42,
        asset_id: b256::zero(),
    }();
}
