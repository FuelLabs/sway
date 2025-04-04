contract;

abi MyOwnedContract {
    fn receive(field_1: u64) -> bool;
}

const OWNER: Address = Address::from(0x9ae5b658754e096e4d681c548daf46354495a437cc61492599e33fc64dcdc30c);

impl MyOwnedContract for Contract {
    fn receive(field_1: u64) -> bool {
        let sender = msg_sender().unwrap();
        if let Identity::Address(addr) = sender {
            assert(addr == OWNER);
        } else {
            revert(0);
        }

        true
    }
}
