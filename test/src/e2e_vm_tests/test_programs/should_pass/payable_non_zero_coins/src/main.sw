script;

dep wallet_abi;

use std::constants::ZERO_B256;
use wallet_abi::Wallet;

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(Wallet, contract_address);
    // `coins:` is missing (its default value is zero) and hence this is not an error
    // even that `send_funds` is not payable
    caller.receive_funds {
        gas: 10000,
        coins: 42,
        asset_id: ZERO_B256,
    }();
}
