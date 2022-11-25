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
        asm(key: KEY, is_set, v: initial_value) {
            sww key is_set v;
        }
        initial_value
    }

    #[storage(read, write)]
    fn increment(increment_by: u64) -> u64 {
        let new_val = asm(key: KEY, is_set, i: increment_by, res) {
            srw res is_set key;
            add res res i;
            sww key is_set res;
            res: u64
        };
        new_val
    }

    #[storage(read)]
    fn get() -> u64 {
        asm(key: KEY, is_set, res) {
            srw res is_set key;
            res: u64
        }
    }
}

// Each function should have a span and a storage attribute.  It gets a little dicey assuming which
// one is which, but should at least be deterministic for any particular version of the compiler.

// check: fn get<75b70457>() -> u64, $(get_md=$MD) {
// check: fn increment<e543c666>(increment_by $MD: u64) -> u64, $(increment_md=$MD) {
// check: fn initialize<557ac400>(initial_value $MD: u64) -> u64, $(init_md=$MD) {

// unordered: $(write_md=$MD) = storage "writes"
// unordered: $(readwrite_md=$MD) = storage "readswrites"
// unordered: $(read_md=$MD) = storage "reads"

// The span idx is first, then the storage attribute.

// unordered: $init_md = ($MD $write_md)
// unordered: $increment_md = ($MD $readwrite_md)
// unordered: $get_md = ($MD $read_md)
