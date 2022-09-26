script;

use interface::Wallet;

fn main(amount_to_send: u64, asset_id: b256, recipient: Identity, wallet_id: b256) -> bool {
    let caller = abi(Wallet, wallet_id);

    caller.send_funds {
        gas: 10000,
        coins: 0,
        asset_id
    }(amount_to_send, recipient);

    true
}
