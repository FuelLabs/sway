script;

use interface::Wallet;
use std::constants::ZERO_B256;

fn main() {
    let amount_to_send = 200;
    let recipient = Identity::Address(~Address::from(OWNER_ADDRESS));
    let caller = abi(Wallet, 0x6ee4526ea417b207cc31bf66843a9134ad303bf403ce0293f09d968462dea1fc);

    caller.send_funds {
        gas: 10000,
        coins: 0,
        asset_id: ZERO_B256,
    }(amount_to_send, recipient);
}
