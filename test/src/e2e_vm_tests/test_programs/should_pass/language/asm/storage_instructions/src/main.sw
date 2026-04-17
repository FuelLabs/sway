contract;

const B256_ZERO: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const B256_ONE: b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const B256_TWO: b256 = 0x0000000000000000000000000000000000000000000000000000000000000002;

impl Contract {
    // BEGIN: SRW

    // Empty slots can be read at any offset, and should always return 0 and not be marked as set.
    #[storage(read)]
    fn srw_empty_slots() {
        let is_set_res = (0u64, 0u64);

        let (is_set, res) = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i0;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
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
    fn srw_occupied_slots_valid_offset_quod() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i0;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let (is_set, res) = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i1;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 43);

        let (is_set, res) = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i2;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 44);

        let (is_set, res) = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i3;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 45);
    }

    // Occupied slots should be readable at the correct offsets, and marked as set.
    #[storage(read, write)]
    fn srw_occupied_slots_valid_offset_dynamic() {
        let slots_data = [42u64, 43u64];
        let _ = __state_store_slot(B256_ZERO, __addr_of(slots_data), 2 * 8);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i0;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        let (is_set, res) = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i1;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
        assert_eq(is_set, 1);
        assert_eq(res, 43);
    }

    // Reading out of slot bounds must revert.
    #[storage(read, write)]
    fn srw_occupied_slots_offset_out_of_bounds_quod() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let is_set_res = (0u64, 0u64);

        let _ = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i4;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
    }

    // Reading out of slot bounds must revert.
    #[storage(read, write)]
    fn srw_occupied_slots_offset_out_of_bounds_dynamic() {
        let slots_data = [42u64, 43u64];
        let _ = __state_store_slot(B256_ZERO, __addr_of(slots_data), 2 * 8);

        let is_set_res = (0u64, 0u64);

        let _ = asm(slot: B256_ZERO, is_set, res, is_set_res: is_set_res) {
            srw res is_set slot i2;
            sw is_set_res is_set i0;
            sw is_set_res res i1;
            is_set_res: (u64, u64)
        };
    }

    // END: SRW

    // BEGIN: SCLR

    #[storage(write)]
    fn sclr_empty_slots() {
        asm(slot: B256_ZERO, num_of_slots: 2) {
            sclr slot num_of_slots;
        };
    }

    // If `slots` argument is zero, no slots are cleared.
    #[storage(write)]
    fn sclr_clear_slots_arg_set_to_zero() {
        let slots_data = [42u64; 4];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        asm(slot: B256_ZERO, num_of_slots: 0) {
            sclr slot num_of_slots;
        };

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);
    }

    #[storage(read, write)]
    fn sclr_occupied_slots() {
        let slots_data = [42u64; 12]; // Three slots of 4 words each.
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 3);

        let is_set_res = (0u64, 0u64);

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        asm(slot: B256_ZERO, num_of_slots: 2) {
            sclr slot num_of_slots;
        };

        let (is_set, res) = read_first_word_in_quod(B256_ZERO);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_ONE);
        assert_eq(is_set, 0);
        assert_eq(res, 0);

        let (is_set, res) = read_first_word_in_quod(B256_TWO);
        assert_eq(is_set, 1);
        assert_eq(res, 42);

        asm(slot: B256_ZERO, num_of_slots: 3) {
            sclr slot num_of_slots;
        };

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

    // END: SCLR

    // BEGIN: SRDD

    #[storage(read)]
    fn srdd_empty_slot() {
        let dest = [42u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 0, len: dest.len()) {
            srdd res slot index len;
            err
        };
        assert_eq(is_err, 1);
        assert_eq(dest, [42u64; 2]);
    }

    #[storage(read)]
    fn srdd_empty_slot_zero_len() {
        let is_err = asm(slot: B256_ZERO, res: __addr_of(()), index: 0, len: 0) {
            srdd res slot index len;
            err
        };
        assert_eq(is_err, 1);
    }

    #[storage(read, write)]
    fn srdd_occupied_slots() {
        let slots_data = (42u64, 43u64, 44u64, 45u64);
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let dest = [0u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 0, len: dest.len() * 8) {
            srdd res slot index len;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(dest, [42u64, 43u64]);

        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 1 * 8, len: dest.len() * 8) {
            srdd res slot index len;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(dest, [43u64, 44u64]);

        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 2 * 8, len: dest.len() * 8) {
            srdd res slot index len;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(dest, [44u64, 45u64]);
    }

    #[storage(read, write)]
    fn srdd_occupied_slots_out_of_bounds() {
        let slots_data = (42u64, 43u64, 44u64, 45u64);
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let dest = [0u64; 4];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 0, len: dest.len() * 9) {
            srdd res slot index len;
            err
        };
    }

    // END: SRDD

    // BEGIN: SRDI

    #[storage(read)]
    fn srdi_empty_slot() {
        let dest = [42u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 0) {
            srdi res slot index i16;
            err
        };
        assert_eq(is_err, 1);
        assert_eq(dest, [42u64; 2]);
    }

    #[storage(read)]
    fn srdi_empty_slot_zero_len() {
        let is_err = asm(slot: B256_ZERO, res: __addr_of(()), index: 0) {
            srdi res slot index i0;
            err
        };
        assert_eq(is_err, 1);
    }

    #[storage(read, write)]
    fn srdi_occupied_slots() {
        let slots_data = (42u64, 43u64, 44u64, 45u64);
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let dest = [0u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 0) {
            srdi res slot index i16;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(dest, [42u64, 43u64]);

        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 1 * 8) {
            srdi res slot index i16;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(dest, [43u64, 44u64]);

        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 2 * 8) {
            srdi res slot index i16;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(dest, [44u64, 45u64]);
    }

    #[storage(read, write)]
    fn srdi_occupied_slots_out_of_bounds() {
        let slots_data = (42u64, 43u64, 44u64, 45u64);
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let dest = [0u64; 4];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(dest), index: 0) {
            srdi res slot index i36;
            err
        };
    }

    // END: SRDI

    // BEGIN: SWRD

    #[storage(read, write)]
    fn swrd() {
        let val = [42u64];
        asm(slot: B256_ZERO, src: __addr_of(val), len: 1 * 8) {
            swrd slot src len;
        };

        let res = [0u64; 1];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i8;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64]);

        let val = [42u64, 43u64];
        asm(slot: B256_ZERO, src: __addr_of(val), len: 2 * 8) {
            swrd slot src len;
        };

        let res = [0u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i16;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64]);

        let val = [42u64, 43u64, 44u64, 45u64];
        asm(slot: B256_ZERO, src: __addr_of(val), len: 4 * 8) {
            swrd slot src len;
        };

        let res = [0u64; 4];
        let is_ok = __state_load_quad(B256_ZERO, __addr_of(res), 1);
        assert_eq(is_ok, true);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64]);

        let val = [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64];
        asm(slot: B256_ZERO, src: __addr_of(val), len: 8 * 8) {
            swrd slot src len;
        };

        let res = [0u64; 8];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0, len: 8 * 8) {
            srdd res slot offset len;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64]);
    }

    // END: SWRD

    // BEGIN: SWRI

    #[storage(read, write)]
    fn swri() {
        let val = [42u64];
        asm(slot: B256_ZERO, src: __addr_of(val)) {
            swri slot src i8;
        };

        let res = [0u64; 1];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i8;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64]);

        let val = [42u64, 43u64];
        asm(slot: B256_ZERO, src: __addr_of(val)) {
            swri slot src i16;
        };

        let res = [0u64; 2];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i16;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64]);

        let val = [42u64, 43u64, 44u64, 45u64];
        asm(slot: B256_ZERO, src: __addr_of(val)) {
            swri slot src i32;
        };

        let res = [0u64; 4];
        let is_ok = __state_load_quad(B256_ZERO, __addr_of(res), 1);
        assert_eq(is_ok, true);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64]);

        let val = [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64];
        asm(slot: B256_ZERO, src: __addr_of(val)) {
            swri slot src i64;
        };

        let res = [0u64; 8];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0, len: 8 * 8) {
            srdd res slot offset len;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64, 46u64, 47u64, 48u64, 49u64]);
    }

    // END: SWRI

    // BEGIN: SUPD

    #[storage(read, write)]
    fn supd_empty_slots_append() {
        let val = [42u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: u64::max(), len: 1 * 8) {
            supd slot src offset len;
        };

        let res = [0u64; 1];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i8;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64]);
    }

    #[storage(read, write)]
    fn supd_occupied_slots_append() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val = [46u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: u64::max(), len: 1 * 8) {
            supd slot src offset len;
        };

        let res = [0u64; 5];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i40;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64, 46u64]);
    }

    #[storage(read, write)]
    fn supd_offset_out_of_bounds() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val = [46u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: 33, len: 1 * 8) {
            supd slot src offset len;
        };
    }

    #[storage(read, write)]
    fn supd_occupied_slots_overwrite() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val = [34u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: 8, len: 1 * 8) {
            supd slot src offset len;
        };

        let res = [0u64; 4];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i32;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 34u64, 44u64, 45u64]);
    }

    #[storage(read, write)]
    fn supd_occupied_slots_overwrite_and_extend() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val = [11u64, 12u64, 13u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: 16, len: 3 * 8) {
            supd slot src offset len;
        };

        let res = [0u64; 5];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i40;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64, 11u64, 12u64, 13u64]);
    }

    // END: SUPD


    // BEGIN: SUPI

    #[storage(read, write)]
    fn supi_empty_slots_append() {
        let val = [42u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: u64::max()) {
            supi slot src offset i8;
        };

        let res = [0u64; 1];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i8;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64]);
    }

    #[storage(read, write)]
    fn supi_occupied_slots_append() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val = [46u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: u64::max()) {
            supi slot src offset i8;
        };

        let res = [0u64; 5];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i40;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64, 44u64, 45u64, 46u64]);
    }

    #[storage(read, write)]
    fn supi_offset_out_of_bounds() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val = [46u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: 33) {
            supi slot src offset i8;
        };
    }

    #[storage(read, write)]
    fn supi_occupied_slots_overwrite() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val = [34u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: 8) {
            supi slot src offset i8;
        };

        let res = [0u64; 4];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i32;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 34u64, 44u64, 45u64]);
    }

    #[storage(read, write)]
    fn supi_occupied_slots_overwrite_and_extend() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let val = [11u64, 12u64, 13u64];
        asm(slot: B256_ZERO, src: __addr_of(val), offset: 16) {
            supi slot src offset i24;
        };

        let res = [0u64; 5];
        let is_err = asm(slot: B256_ZERO, res: __addr_of(res), offset: 0) {
            srdi res slot offset i40;
            err
        };
        assert_eq(is_err, 0);
        assert_eq(res, [42u64, 43u64, 11u64, 12u64, 13u64]);
    }

    // END: SUPI

    // BEGIN: SPLD

    #[storage(read)]
    fn spld_empty_slot() {
        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ZERO, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 1);
        assert_eq(len, 0);
    }

    #[storage(read, write)]
    fn spld_occupied_slot_zero_length() {
        let val = [42u64];
        asm(slot: B256_ZERO, src: __addr_of(val), len: 0) {
            swrd slot src len;
        };

        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ZERO, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 0);
        assert_eq(len, 0);
    }

    #[storage(read, write)]
    fn spld_occupied_slot_quad() {
        let slots_data = [42u64, 43u64, 44u64, 45u64];
        let _ = __state_store_quad(B256_ZERO, __addr_of(slots_data), 1);

        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ZERO, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 0);
        assert_eq(len, 4 * 8);
    }

    #[storage(read, write)]
    fn spld_occupied_slot_dynamic() {
        let val = [11u64, 12u64, 13u64];
        asm(slot: B256_ZERO, src: __addr_of(val), len: 3 * 8) {
            swrd slot src len;
        };

        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ZERO, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 0);
        assert_eq(len, 3 * 8);
    }

    // Preloading different slots must return independent lengths.
    #[storage(write)]
    fn spld_different_slots() {
        let val_a = [1u64, 2u64];
        asm(slot: B256_ZERO, src: __addr_of(val_a), len: 2 * 8) {
            swrd slot src len;
        };

        let val_b = [3u64, 4u64, 5u64, 6u64, 7u64];
        asm(slot: B256_ONE, src: __addr_of(val_b), len: 5 * 8) {
            swrd slot src len;
        };

        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ZERO, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 0);
        assert_eq(len, 2 * 8);

        let is_not_set_len = (0u64, 0u64);
        let (is_not_set, len) = asm(slot: B256_ONE, len, is_not_set_len: is_not_set_len) {
            spld len slot;
            sw is_not_set_len err i0;
            sw is_not_set_len len i1;
            is_not_set_len: (u64, u64)
        };

        assert_eq(is_not_set, 0);
        assert_eq(len, 5 * 8);
    }

    // END: SPLD
}

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
fn test_srw_empty_slots() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srw_empty_slots();
}

#[test]
fn test_srw_occupied_slots_valid_offsets_quod() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srw_occupied_slots_valid_offset_quod();
}

#[test]
fn test_srw_occupied_slots_valid_offsets_dynamic() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srw_occupied_slots_valid_offset_dynamic();
}

#[test(should_revert)]
fn test_srw_occupied_slots_offset_out_of_bounds_quod() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srw_occupied_slots_offset_out_of_bounds_quod();
}

#[test(should_revert)]
fn test_srw_occupied_slots_offset_out_of_bounds_dynamic() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srw_occupied_slots_offset_out_of_bounds_dynamic();
}

#[test]
fn test_sclr_empty_slots() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.sclr_empty_slots();
}

#[test]
fn test_sclr_clear_slots_arg_set_to_zero() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.sclr_clear_slots_arg_set_to_zero();
}

#[test]
fn test_sclr_occupied_slots() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.sclr_occupied_slots();
}

#[test]
fn test_srdd_empty_slot() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srdd_empty_slot();
}

#[test]
fn test_srdd_empty_slot_zero_len() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srdd_empty_slot_zero_len();
}

#[test]
fn test_srdd_occupied_slots() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srdd_occupied_slots();
}

#[test(should_revert)]
fn test_srdd_occupied_slots_out_of_bounds() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srdd_occupied_slots_out_of_bounds();
}

#[test]
fn test_srdi_empty_slot() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srdi_empty_slot();
}

#[test]
fn test_srdi_empty_slot_zero_len() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srdi_empty_slot_zero_len();
}

#[test]
fn test_srdi_occupied_slots() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srdi_occupied_slots();
}

#[test(should_revert)]
fn test_srdi_occupied_slots_out_of_bounds() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.srdi_occupied_slots_out_of_bounds();
}

#[test]
fn test_swrd() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.swrd();
}

#[test]
fn test_swri() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.swri();
}

#[test]
fn test_supd_empty_slots_append() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supd_empty_slots_append();
}

#[test]
fn test_supd_occupied_slots_append() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supd_occupied_slots_append();
}

#[test(should_revert)]
fn test_supd_offset_out_of_bounds() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supd_offset_out_of_bounds();
}

#[test]
fn test_supd_occupied_slots_overwrite() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supd_occupied_slots_overwrite();
}

#[test]
fn test_supd_occupied_slots_overwrite_and_extend() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supd_occupied_slots_overwrite_and_extend();
}

#[test]
fn test_supi_empty_slots_append() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supi_empty_slots_append();
}

#[test]
fn test_supi_occupied_slots_append() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supi_occupied_slots_append();
}

#[test(should_revert)]
fn test_supi_offset_out_of_bounds() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supi_offset_out_of_bounds();
}

#[test]
fn test_supi_occupied_slots_overwrite() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supi_occupied_slots_overwrite();
}

#[test]
fn test_supi_occupied_slots_overwrite_and_extend() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.supi_occupied_slots_overwrite_and_extend();
}

#[test]
fn test_spld_empty_slot() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.spld_empty_slot();
}

#[test]
fn test_spld_occupied_slot_zero_length() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.spld_occupied_slot_zero_length();
}

#[test]
fn test_spld_occupied_slot_quad() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.spld_occupied_slot_quad();
}

#[test]
fn test_spld_occupied_slot_dynamic() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.spld_occupied_slot_dynamic();
}

#[test]
fn test_spld_different_slots() {
    let caller = abi(StorageInstructionsAbi, CONTRACT_ID);
    caller.spld_different_slots();
}
