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

    #[storage(read, write)]
    fn increment_or_not(initial_value: Option<u64>) -> u64 {
        let current_val = storage.value.read();
        match initial_value {
            Some(increment_by) => {
                let new_val = current_val + increment_by;
                storage.value.write(new_val);
                new_val
            }
            None => {
                current_val
            }
        }
    }

    #[storage(read)]
    fn get() -> u64 {
        storage.value.read()
    }
}

#[fallback]
fn fallback() -> u64 {
    444444444
}

#[test]
fn collect_incrementor_contract_gas_usages() {
    let caller = abi(Incrementor, CONTRACT_ID);
    let _ = caller.get();
    let _ = caller.increment(0);
}
