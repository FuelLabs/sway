contract;

const B256_ZERO: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const B256_ONE: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const B256_TWO: b256 = 0x0000000000000000000000000000000000000000000000000000000000000002;

impl Contract {
    // BEGIN: __state_load_word

    // Empty slots can be read at any offset, and should always return 0 and not be marked as set.
    #[cfg(experimental_dynamic_storage = true)]
    #[storage(read)]
    fn state_load_word_empty_slots() {
        let res = __state_load_word(B256_ZERO, 0);
        assert_eq(res, 0);

        let res = __state_load_word(B256_ZERO, 42);
        assert_eq(res, 0);
    }

    #[cfg(experimental_dynamic_storage = false)]
    #[storage(read)]
    fn state_load_word_empty_slots() {
        let res = __state_load_word(B256_ZERO);
        assert_eq(res, 0);
    }

    #[cfg(experimental_dynamic_storage = true)]
    #[storage(read, write)]
    fn state_load_word_occupied_slots_valid_offset_quod() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let res = __state_load_word(B256_ZERO, 0);
        assert_eq(res, 42);

        let res = __state_load_word(B256_ZERO, 1);
        assert_eq(res, 43);

        let res = __state_load_word(B256_ZERO, 2);
        assert_eq(res, 44);

        let res = __state_load_word(B256_ZERO, 3);
        assert_eq(res, 45);
    }

    #[cfg(experimental_dynamic_storage = false)]
    #[storage(read, write)]
    fn state_load_word_occupied_slots_valid_offset_quod() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let res = __state_load_word(B256_ZERO);
        assert_eq(res, 42);
    }

    #[cfg(experimental_dynamic_storage = true)]
    #[storage(read, write)]
    fn state_load_word_occupied_slots_valid_offset_dynamic() {
        let slots_data = [42u64, 43u64];
        let _ = __state_store_slot(B256_ZERO, __addr_of(slots_data), 2 * 8);

        let res = __state_load_word(B256_ZERO, 0);
        assert_eq(res, 42);

        let res = __state_load_word(B256_ZERO, 1);
        assert_eq(res, 43);
    }

    #[cfg(experimental_dynamic_storage = false)]
    #[storage(read, write)]
    fn state_load_word_occupied_slots_valid_offset_dynamic() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_slot(B256_ZERO, __addr_of(slots_data), 2 * 8);

        let res = __state_load_word(B256_ZERO);
        assert_eq(res, 42);
    }

    // Reading out of slot bounds must revert.
    #[cfg(experimental_dynamic_storage = true)]
    #[storage(read, write)]
    fn state_load_word_occupied_slots_offset_out_of_bounds_quod() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let res = __state_load_word(B256_ZERO, 4);
        poke(res);
    }

    #[cfg(experimental_dynamic_storage = true)]
    #[storage(read, write)]
    fn state_load_word_occupied_slots_offset_out_of_bounds_dynamic() {
        let slots_data = [42u64, 43u64];
        let _ = __state_store_slot(B256_ZERO, __addr_of(slots_data), 2 * 8);

        let res = __state_load_word(B256_ZERO, 2);
        poke(res);
    }

    // END: __state_load_word

    // BEGIN: __state_clear

    #[storage(write)]
    fn state_clear_empty_slots() {
        let res = __state_clear(B256_ZERO, 1);
        assert_eq(res, false);
    }

    // If `slots` argument is zero, no slots are cleared, and the return value
    // is always true.
    #[storage(write)]
    fn state_clear_slots_arg_set_to_zero() {
        let res = __state_clear(B256_ZERO, 0);
        assert_eq(res, true);

        let slots_data = [42u64; 4];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let res = __state_clear(B256_ZERO, 0);
        assert_eq(res, true);

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);
    }

    #[storage(write)]
    fn state_clear_occupied_slots() {
        let slots_data = [42u64; 12]; // Three slots of 4 words each.
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 3);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let res = __state_clear(B256_ZERO, 2);
        assert_eq(res, true);

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_ONE);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_TWO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let res = __state_clear(B256_ZERO, 3);
        assert_eq(res, false); // False because the first two slots were already cleared.

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_ONE);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_TWO);
        assert_eq(is_set, 0);
        assert_eq(res, 0);
    }

    // END: __state_clear

    // BEGIN: __state_clear_slots

    #[storage(write)]
    fn state_clear_slots_empty_slots() {
        let res = __state_clear_slots(B256_ZERO, 1);
        assert_eq(res, ());
    }

    // If `slots` argument is zero, no slots are cleared.
    #[storage(write)]
    fn state_clear_slots_slots_arg_set_to_zero() {
        let res = __state_clear_slots(B256_ZERO, 0);
        assert_eq(res, ());

        let slots_data = [42u64; 4];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let res = __state_clear_slots(B256_ZERO, 0);
        assert_eq(res, ());

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);
    }

    #[storage(write)]
    fn state_clear_slots_occupied_slots() {
        let slots_data = [42u64; 12]; // Three slots of 4 words each.
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 3);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let res = __state_clear_slots(B256_ZERO, 2);
        assert_eq(res, ());

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_ONE);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_TWO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let res = __state_clear_slots(B256_ZERO, 3);
        assert_eq(res, ());

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_ONE);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_TWO);
        assert_eq(is_set, 0);
        assert_eq(res, 0);
    }

    // END: __state_clear_slots

    // BEGIN: __state_store_quad

    // Writing zero quads does not write anything into an empty slot.
    #[storage(read, write)]
    fn state_store_quad_zero_quads_in_empty_slot() {
        let val: [u8; 0] = [];
        let _ = __state_store_quad(B256_ZERO, __addr_of(val), 0);

        let res = [42u64; 1];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i0;
            err
        };
        assert_eq(is_err, 1); // Slot does not exists.
        assert_eq(res, [42u64]); // Memory is not overwritten.

        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ZERO, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 1); // The slot is not set.
        assert_eq(len, 0); // The length of the non-set slot is zero.
    }

    // Writing zero quads does not write anything into an empty slot.
    #[storage(read, write)]
    fn state_store_quad_zero_quads_in_occupied_slot() {
        // Occupy slot.
        let slots_data = [42u64; 4];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val: [u8; 0] = [];
        let _ = __state_store_quad(B256_ZERO, __addr_of(val), 0);

        let res = [0u64; 4];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i32;
            err
        };
        assert_eq(is_err, 0); // Slot exists.
        assert_eq(res, [42u64, 42u64, 42u64, 42u64]); // Slot is not overwritten.

        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ZERO, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 0); // The slot is still set.
        assert_eq(len, 32); // The length of the original slot.
    }

    // END: __state_store_quad

    // BEGIN: __state_store_slot

    #[storage(read, write)]
    fn state_store_slot_zero_size_data() {
        let val: [u8; 0] = [];
        __state_store_slot(B256_ZERO, __addr_of(val), 0);

        let res = [42u64; 1];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i0;
            err
        };
        assert_eq(is_err, 0); // Slot exists.
        assert_eq(res, [42u64]); // Memory is not overwritten.

        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ZERO, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 0); // The slot is set, but empty.
        assert_eq(len, 0); // The length of the slot is zero, although it is set.
    }

    #[storage(read, write)]
    fn state_store_slot_one_word() {
        let val = [42u64];
        __state_store_slot(B256_ZERO, __addr_of(val), 1 * 8);

        let res = [0u64; 1];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i8;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64]);
    }

    #[storage(read, write)]
    fn state_store_slot_two_words() {
        let val = [42u64, 43u64];
        __state_store_slot(B256_ZERO, __addr_of(val), 2 * 8);

        let res = [0u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i16;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64]);
    }

    #[storage(read, write)]
    fn state_store_slot_quod() {
        let val = [42u64, 43u64, 44u64, 45u64];
        __state_store_slot(B256_ZERO, __addr_of(val), 4 * 8);

        let res = [0u64; 4];
        let is_ok = __state_load_quad(B256_ZERO, __addr_of(res), 1);
        assert_eq(is_ok, true);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64]);
    }

    #[storage(read, write)]
    fn state_store_slot_two_quods() {
        let val = [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64];
        __state_store_slot(B256_ZERO, __addr_of(val), 8 * 8);

        let res = [0u64; 8];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0, len: 8 * 8) {
            srdd res slot offset len;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64]);
    }

    // Store with a runtime (non-constant) length to ensure the SWRD (register) path is used.
    #[storage(read, write)]
    fn state_store_slot_runtime_len() {
        let val = [42u64, 43u64];
        let len = get_runtime_len(2 * 8);
        __state_store_slot(B256_ZERO, __addr_of(val), len);

        let res = [0u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i16;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64]);
    }

    // Store slots of different lengths.
    #[storage(read, write)]
    fn state_store_slot_overwrites() {
        let val = [42u64, 43u64];
        let len = get_runtime_len(2 * 8);
        __state_store_slot(B256_ZERO, __addr_of(val), len);
        assert_eq(len, __state_preload(B256_ZERO));

        let res = [0u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i16;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64]);

        // Overwrite.
        let val = [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64];
        __state_store_slot(B256_ZERO, __addr_of(val), 8 * 8);
        assert_eq(8 * 8, __state_preload(B256_ZERO));

        let res = [0u64; 8];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0, len: 8 * 8) {
            srdd res slot offset len;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64]);

        // Truncate to zero.
        let val: [u64;0] = [];
        __state_store_slot(B256_ZERO, __addr_of(val), 0);
        assert_eq(0, __state_preload(B256_ZERO));
    }

    // END: __state_store_slot

    // BEGIN: __state_load_slot

    // Read from an empty slot. Should return false (slot was not previously set).
    #[storage(read)]
    fn state_load_slot_empty_slot() {
        let dest = [42u64; 2];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(dest), 0, 2 * 8);
        assert_eq(was_set, false);
        assert_eq(dest, [42u64; 2]);
    }

    #[storage(read)]
    fn state_load_slot_empty_slot_zero_len() {
        let was_set = __state_load_slot(B256_ZERO, __addr_of(()), 0, 0);
        assert_eq(was_set, false);
    }

    #[storage(read, write)]
    fn state_load_slot_occupied_slot() {
        let slots_data = (42u64, 43u64, 44u64, 45u64);
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let dest = [0u64; 2];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(dest), 0, 2 * 8);
        assert_eq(was_set, true);
        assert_eq(dest, [42u64, 43u64]);

        let was_set = __state_load_slot(B256_ZERO, __addr_of(dest), 1 * 8, 2 * 8);
        assert_eq(was_set, true);
        assert_eq(dest, [43u64, 44u64]);

        let was_set = __state_load_slot(B256_ZERO, __addr_of(dest), 2 * 8, 2 * 8);
        assert_eq(was_set, true);
        assert_eq(dest, [44u64, 45u64]);
    }

    #[storage(read, write)]
    fn state_load_large_slot() {
        let val = [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64];
        __state_store_slot(B256_ZERO, __addr_of(val), 8 * 8);

        let dest = [0u64; 8];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(dest), 0, 8 * 8);
        assert_eq(was_set, true);
        assert_eq(dest, [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64]);
    }

    // Read with a runtime (non-constant) length to ensure the SRDD (register) path is used.
    #[storage(read, write)]
    fn state_load_slot_runtime_len() {
        let val = [42u64, 43u64];
        __state_store_slot(B256_ZERO, __addr_of(val), 2 * 8);

        let dest = [0u64; 2];
        let len = get_runtime_len(2 * 8);
        let was_set = __state_load_slot(B256_ZERO, __addr_of(dest), 0, len);
        assert_eq(was_set, true);
        assert_eq(dest, [42u64, 43u64]);
    }

    #[storage(read, write)]
    fn state_load_slot_out_of_bounds() {
        let val = [42u64, 43u64];
        __state_store_slot(B256_ZERO, __addr_of(val), 2 * 8);

        let dest = [0u64; 8];
        let _ = __state_load_slot(B256_ZERO, __addr_of(dest), 2 * 8 + 1, 1 * 8);
    }

    // END: __state_load_slot

    // BEGIN: __state_update_slot

    // Append to an empty slot (offset = u64::max).
    #[storage(read, write)]
    fn state_update_slot_empty_append() {
        let val = [42u64];
        __state_update_slot(B256_ZERO, __addr_of(val), u64::max(), 1 * 8);

        let res = [0u64; 1];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(res), 0, 1 * 8);
        assert_eq(was_set, true);
        assert_eq(res, [42u64]);
    }

    // Append to an occupied slot (offset = u64::max).
    #[storage(read, write)]
    fn state_update_slot_occupied_append() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        __state_store_slot(B256_ZERO, __addr_of(slots_data), 4 * 8);

        let val = [46u64];
        __state_update_slot(B256_ZERO, __addr_of(val), u64::max(), 1 * 8);

        let res = [0u64; 5];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(res), 0, 5 * 8);
        assert_eq(was_set, true);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64, 46u64]);
    }

    // Overwrite a portion of an occupied slot at a specific offset.
    #[storage(read, write)]
    fn state_update_slot_overwrite() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        __state_store_slot(B256_ZERO, __addr_of(slots_data), 4 * 8);

        let val = [34u64];
        __state_update_slot(B256_ZERO, __addr_of(val), 1 * 8, 1 * 8);

        let res = [0u64; 4];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(res), 0, 4 * 8);
        assert_eq(was_set, true);
        assert_eq(res, [42u64, 34u64, 44u64, 45u64]);
    }

    // Overwrite and extend past the current end of a slot.
    #[storage(read, write)]
    fn state_update_slot_overwrite_and_extend() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        __state_store_slot(B256_ZERO, __addr_of(slots_data), 4 * 8);

        let val = [11u64, 12u64, 13u64];
        __state_update_slot(B256_ZERO, __addr_of(val), 2 * 8, 3 * 8);

        let res = [0u64; 5];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(res), 0, 5 * 8);
        assert_eq(was_set, true);
        assert_eq(res, [42u64, 43u64, 11u64, 12u64, 13u64]);
    }

    // Update with a runtime (non-constant) length to ensure the SUPD (register) path is used.
    #[storage(read, write)]
    fn state_update_slot_runtime_len() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        __state_store_slot(B256_ZERO, __addr_of(slots_data), 4 * 8);

        let val = [34u64];
        let len = get_runtime_len(1 * 8);
        __state_update_slot(B256_ZERO, __addr_of(val), 1 * 8, len);

        let res = [0u64; 4];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(res), 0, 4 * 8);
        assert_eq(was_set, true);
        assert_eq(res, [42u64, 34u64, 44u64, 45u64]);
    }

    // The index equal to slot length (one after the last byte) has the
    // append semantics, same as passing `u64::max()`.
    #[storage(read, write)]
    fn state_update_slot_offset_equal_slot_length() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        __state_store_slot(B256_ZERO, __addr_of(slots_data), 4 * 8);

        let val = [46u64];
        __state_update_slot(B256_ZERO, __addr_of(val), 32, 1 * 8);

        let res = [0u64; 5];
        let was_set = __state_load_slot(B256_ZERO, __addr_of(res), 0, 5 * 8);
        assert_eq(was_set, true);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64, 46u64]);
    }

    #[storage(write)]
    fn state_update_slot_update_out_of_bounds() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        __state_store_slot(B256_ZERO, __addr_of(slots_data), 4 * 8);

        let val = [46u64];
        __state_update_slot(B256_ZERO, __addr_of(val), 32 + 1, 1 * 8);
    }

    // END: __state_update_slot

    // BEGIN: __state_preload

    #[storage(read)]
    fn state_preload_empty_slot() {
        let len = __state_preload(B256_ZERO);
        assert_eq(len, 0);
    }

    #[storage(read, write)]
    fn state_preload_occupied_slot_zero_length() {
        let val = [42u64];
        asm(slot: B256_ZERO, src: __addr_of(val), len: 0) {
            swrd slot src len;
        };

        let len = __state_preload(B256_ZERO);
        assert_eq(len, 0);
    }

    #[storage(read, write)]
    fn state_preload_occupied_slot_quad() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let len = __state_preload(B256_ZERO);
        assert_eq(len, 4 * 8);
    }

    #[storage(read, write)]
    fn state_preload_occupied_slot_dynamic() {
        let val = [11u64, 12u64, 13u64];
        asm(slot: B256_ZERO, src: __addr_of(val), len: 3 * 8) {
            swrd slot src len;
        };

        let len = __state_preload(B256_ZERO);
        assert_eq(len, 3 * 8);
    }

    // Preloading different slots must return independent lengths.
    #[storage(read, write)]
    fn state_preload_different_slots() {
        let val_a = [1u64, 2u64];
        asm(slot: B256_ZERO, src: __addr_of(val_a), len: 2 * 8) {
            swrd slot src len;
        };

        let val_b = [3u64, 4u64, 5u64, 6u64, 7u64];
        asm(slot: B256_ONE, src: __addr_of(val_b), len: 5 * 8) {
            swrd slot src len;
        };

        let len = __state_preload(B256_ZERO);
        assert_eq(len, 2 * 8);

        let len = __state_preload(B256_ONE);
        assert_eq(len, 5 * 8);
    }

    // END: __state_preload
}

#[inline(never)]
fn get_runtime_len(len: u64) -> u64 {
    len
}

// TODO-DCA: Fix false DCA warning for this function as a part of https://github.com/FuelLabs/sway/issues/5921.
#[allow(dead_code)]
#[inline(never)]
fn poke<T>(_t: T) { }

// TODO-DCA: Fix false DCA warning for this function as a part of https://github.com/FuelLabs/sway/issues/5921.
#[allow(dead_code)]
#[storage(read)]
fn read_first_word_in_quod(slot: b256) -> (u64, u64) {
    let is_set_res = (0u64, 0u64);
    asm(slot: slot, is_set, res, is_set_res: is_set_res) {
        srw res is_set slot i0;
        sw is_set_res is_set i0;
        sw is_set_res res i1;
        is_set_res: (u64, u64)
    }
}

#[test]
fn test_state_load_word_empty_slots() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_word_empty_slots();
}

#[test]
fn test_state_load_word_occupied_slots_valid_offset_quod() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_word_occupied_slots_valid_offset_quod();
}

#[test]
fn test_state_load_word_occupied_slots_valid_offset_dynamic() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_word_occupied_slots_valid_offset_dynamic();
}

#[test(should_revert)]
#[cfg(experimental_dynamic_storage = true)]
fn test_state_load_word_occupied_slots_offset_out_of_bounds_quod() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_word_occupied_slots_offset_out_of_bounds_quod();
}

#[test(should_revert)]
#[cfg(experimental_dynamic_storage = true)]
fn test_state_load_word_occupied_slots_offset_out_of_bounds_dynamic() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_word_occupied_slots_offset_out_of_bounds_dynamic();
}

#[test]
fn test_state_clear_empty_slots() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_clear_empty_slots();
}

#[test]
fn test_state_clear_slots_arg_set_to_zero() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_clear_slots_arg_set_to_zero();
}

#[test]
fn test_state_clear_occupied_slots() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_clear_occupied_slots();
}

#[test]
fn test_state_clear_slots_empty_slots() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_clear_slots_empty_slots();
}

#[test]
fn test_state_clear_slots_slots_arg_set_to_zero() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_clear_slots_slots_arg_set_to_zero();
}

#[test]
fn test_state_clear_slots_occupied_slots() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_clear_slots_occupied_slots();
}

#[test]
fn test_state_store_quad_zero_quads_in_empty_slot() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_quad_zero_quads_in_empty_slot();
}

#[test]
fn test_state_store_quad_zero_quads_in_occupied_slot() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_quad_zero_quads_in_occupied_slot();
}

#[test]
fn test_state_store_slot_zero_size_data() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_slot_zero_size_data();
}

#[test]
fn test_state_store_slot_one_word() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_slot_one_word();
}

#[test]
fn test_state_store_slot_two_words() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_slot_two_words();
}

#[test]
fn test_state_store_slot_quod() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_slot_quod();
}

#[test]
fn test_state_store_slot_two_quods() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_slot_two_quods();
}

#[test]
fn test_state_store_slot_runtime_len() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_slot_runtime_len();
}

#[test]
fn test_state_store_slots_overwrites() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_store_slot_overwrites();
}

#[test]
fn test_state_load_slot_empty_slot() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_slot_empty_slot();
}

#[test]
fn test_state_load_slot_empty_slot_zero_len() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_slot_empty_slot_zero_len();
}

#[test]
fn test_state_load_slot_occupied_slot() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_slot_occupied_slot();
}

#[test]
fn test_state_load_large_slot() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_large_slot();
}

#[test]
fn test_state_load_slot_runtime_len() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_slot_runtime_len();
}

#[test(should_revert)]
fn test_state_load_slot_out_of_bounds() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_load_slot_out_of_bounds();
}

#[test]
fn test_state_update_slot_empty_append() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_update_slot_empty_append();
}

#[test]
fn test_state_update_slot_occupied_append() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_update_slot_occupied_append();
}

#[test]
fn test_state_update_slot_overwrite() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_update_slot_overwrite();
}

#[test]
fn test_state_update_slot_overwrite_and_extend() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_update_slot_overwrite_and_extend();
}

#[test]
fn test_state_update_slot_runtime_len() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_update_slot_runtime_len();
}

#[test(should_revert)]
fn test_state_update_slot_update_out_of_bounds() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_update_slot_update_out_of_bounds();
}

#[test]
fn test_state_update_slot_offset_equal_slot_length() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_update_slot_offset_equal_slot_length();
}

#[test]
fn test_state_preload_empty_slot() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_preload_empty_slot();
}

#[test]
fn test_state_preload_occupied_slot_zero_length() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_preload_occupied_slot_zero_length();
}

#[test]
fn test_state_preload_occupied_slot_quad() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_preload_occupied_slot_quad();
}

#[test]
fn test_state_preload_occupied_slot_dynamic() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_preload_occupied_slot_dynamic();
}

#[test]
fn test_state_preload_different_slots() {
    let caller = abi(StorageIntrinsicsAbi, CONTRACT_ID);
    caller.state_preload_different_slots();
}
