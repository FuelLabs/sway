contract;

abi ImpurityTest {
    #[storage(read, write)]
    fn impure_func() -> bool;
}

impl ImpurityTest for Contract {
    #[storage(read, write)]
    fn impure_func() -> bool {
        let _a = can_also_read_and_write();
        true
    }
}

#[storage(read)]
fn can_read_only() -> u64 {
    100
}

#[storage(read)]
fn can_also_read_only() -> u64 {
    can_read_only()
}

#[storage(write)]
fn can_write_only() -> u64 {
    101
}

#[storage(write)]
fn can_also_write_only() -> u64 {
    can_write_only()
}

#[storage(read, write)]
fn can_read_and_write() -> u64 {
    let _a = can_also_read_only();
    let _b = can_also_write_only();
    102
}

#[storage(read)]
#[storage(write)]
fn can_also_read_and_write() -> u64 {
    can_read_and_write()
}
