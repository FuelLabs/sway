contract;

const KEY = 0x0000000000000000000000000000000000000000000000000000000000000000;

abi Incrementor {
    #[storage(write)]
    fn initialize(initial_value: u64) -> u64;

    #[storage(read, write)]
    fn increment(initial_value: u64) -> u64;

    #[storage(read)]
    fn get() -> u64;
}

impl Incrementor for Contract {
    #[storage(write)]
    fn initialize(initial_value: u64) -> u64 {
        asm(key: KEY, v: initial_value) {
            sww key v;
        }
        initial_value
    }

    #[storage(read, write)]
    fn increment(increment_by: u64) -> u64 {
        let new_val = asm(key: KEY, i: increment_by, res) {
            srw res key;
            add res res i;
            sww key res;
            res: u64
        };
        new_val
    }

    #[storage(read)]
    fn get() -> u64 {
        asm(key: KEY, res) {
            srw key res;
            res: u64
        }
    }
}

// check: fn initialize<557ac400>(initial_value $MD: u64) -> u64, $MD, $(write_md=$MD) {
// check: fn increment<e543c666>(increment_by $MD: u64) -> u64, $MD, $(readwrite_md=$MD) {
// check: fn get<75b70457>() -> u64, $MD, $(read_md=$MD) {

// unordered: $write_md = storage write
// unordered: $readwrite_md = storage readwrite
// unordered: $read_md = storage read
