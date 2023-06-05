contract;

use increment_abi::Incrementor;

storage {
  value: u64 = 0,
}

impl Incrementor for Contract {
    #[storage(read, write)]
    fn increment(increment_by: u64) -> u64 {
        let new_val = storage.value.read() + increment_by;
        storage.value.write(new_val);
        new_val
    }

    #[storage(read)]
    fn get() -> u64 {
        storage.value.read()
    }
}
