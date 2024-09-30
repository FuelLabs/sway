contract;

use interface::Wallet;

impl Wallet for Contract {
    #[storage(read, write)]
    fn receive_funds() {
        // function implementation
    }

    #[storage(read, write)]
    fn send_funds(amount_to_send: u64, recipient: Identity) {
        // function implementation
    }
}
