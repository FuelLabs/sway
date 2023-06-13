script;

abi MyContract {
    #[storage(read)]
    fn get_value() -> u64;
}

fn main() -> u64 {
    let contract_id = 0x4fcd4b153b087f7e0080314533d24cf48b40dccce20acacf60786de0c230884c;
    let caller = abi(MyContract, contract_id);
    caller.get_value()
}