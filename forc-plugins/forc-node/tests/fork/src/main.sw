contract;

storage {
    counter: u64 = 0,
}

struct Adder {
    vals: (u64, u64),
}

abi Fork {
    #[storage(read)]
    fn get_count() -> u64;
    
    #[storage(read, write)]
    fn increment_count(adder: Adder);
}

impl Fork for Contract {
    #[storage(read)]
    fn get_count() -> u64 {
        storage.counter.read()
    }

    #[storage(read, write)]
    fn increment_count(adder: Adder) {
        let counter = storage.counter.read();
        let new_counter = counter + adder.vals.0 + adder.vals.1;
        storage.counter.write(new_counter);
    }
}