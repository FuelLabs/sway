contract;

impl Contract {
    // Empty slots can be read at any offset, and should always return 0 and not be marked as set.
    #[storage(read, write)]
    fn srw_empty_slots() {
        let slot = b256::zero();

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = asm(slot: slot, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i0;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = asm(slot: slot, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i42;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 0);
        assert_eq(res, 0);
    }

    // Occupied slots should be readable at the correct offsets, and marked as set.
    #[storage(read, write)]
    fn srw_occupied_slots_valid_offset() {
        let slot = b256::zero();

        let slots_data = [42u64, 43u64, 44u64, 45u64];
        __state_store_quad(slot, __addr_of(slots_data), 1);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = asm(slot: slot, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i0;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let (is_set, res) = asm(slot: slot, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i1;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 43);

        let (is_set, res) = asm(slot: slot, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i2;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 44);

        let (is_set, res) = asm(slot: slot, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i3;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 45);
    }

    // Reading out of slot bounds must revert.
    #[storage(read, write)]
    fn srw_occupied_slots_offset_out_of_bounds() {
        let slot = b256::zero();

        let slots_data = [42u64, 43u64, 44u64, 45u64];
        __state_store_quad(slot, __addr_of(slots_data), 1);

        let is_set_res = (0u64, 0u64);

        asm(slot: slot, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i4;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
    }
}

#[inline(never)]
fn poke<T>(_t: T) {}

#[test]
fn test_srw_empty_slots() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srw_empty_slots();
}

#[test]
fn test_srw_occupied_slots_valid_offsets() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srw_occupied_slots_valid_offset();
}

#[test(should_revert)]
fn test_srw_occupied_slots_offset_out_of_bounds() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srw_occupied_slots_offset_out_of_bounds();
}
