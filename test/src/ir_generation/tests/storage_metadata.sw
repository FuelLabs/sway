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
            srw res is_set key i0;
            add res res i;
            sww key is_set res;
            res: u64
        };
        new_val
    }

    #[storage(read)]
    fn get() -> u64 {
        asm(key: KEY, is_set, res) {
            srw res is_set key i0;
            res: u64
        }
    }
}

// Each function should have a span and a storage attribute.  It gets a little dicey assuming which
// one is which, but should at least be deterministic for any particular version of the compiler.

// check: fn get<75b70457>() -> u64, $(get_md=$MD) {
// check: fn increment<e543c666>(increment_by $MD: u64) -> u64, $(increment_md=$MD) {
// check: fn initialize<557ac400>(initial_value $MD: u64) -> u64, $(init_md=$MD) {

// unordered: $(write_md=$MD) = purity "writes"
// unordered: $(write_fn_name_md=$MD) = fn_name_span $MD 359 369

// unordered: $(readwrite_md=$MD) = purity "readswrites"
// unordered: $(readwrite_fn_name_md=$MD) = fn_name_span $MD 553 562

// unordered: $(read_md=$MD) = purity "reads"
// unordered: $(read_fn_name_md=$MD) = fn_name_span $MD 836 839

// The span idx is first, then the storage attribute, then the function name attribute.

// unordered: $init_md = ($MD $write_md $write_fn_name_md)
// unordered: $increment_md = ($MD $readwrite_md $readwrite_fn_name_md)
// unordered: $get_md = ($MD $read_md $read_fn_name_md)