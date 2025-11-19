contract;

use storage_enum_abi::*;

storage {
    // The "value" represents the expected value in the slot.
    s_u8_a: SingleU8 = SingleU8::A(0xCD),           // "value": "0000000000000000 00000000000000cd 0000000000000000 0000000000000000"
    s_u64_a: SingleU64 = SingleU64::A(0xAB),        // "value": "0000000000000000 00000000000000ab 0000000000000000 0000000000000000"
    s_bool_a: SingleBool = SingleBool::A(true),     // "value": "0000000000000000 0000000000000001 0000000000000000 0000000000000000"
    m_units_c: MultiUnits = MultiUnits::C,          // "value": "0000000000000002 0000000000000000 0000000000000000 0000000000000000"
    m_ob_a: MultiOneByte = MultiOneByte::A(true),   // "value": "0000000000000000 0000000000000001 0000000000000000 0000000000000000"
    m_ob_b: MultiOneByte = MultiOneByte::B(0xCD),   // "value": "0000000000000001 00000000000000cd 0000000000000000 0000000000000000"
    m_ob_c: MultiOneByte = MultiOneByte::C,         // "value": "0000000000000002 0000000000000000 0000000000000000 0000000000000000"
    u8_u64_a: U8AndU64 = U8AndU64::A(0xAA),         // "value": "0000000000000000 00000000000000aa 0000000000000000 0000000000000000"
    u8_u64_b: U8AndU64 = U8AndU64::B(0xBB00),       // "value": "0000000000000001 000000000000bb00 0000000000000000 0000000000000000"
    slot_s_a: SlotSize = SlotSize::A(0xEE),         // "value": "0000000000000000 0000000000000000 0000000000000000 00000000000000ee"
    slot_s_b: SlotSize = SlotSize::B((0xAA00, 0xBB00, 0xCC00)),  // "value": "0000000000000001 000000000000aa00 000000000000bb00 000000000000cc00"
    // We expect the FF to be in the second slot on the end of the second word.
    lt_slot_a: LargerThanSlot = LargerThanSlot::A(0xFF),         // "value": "0000000000000000 0000000000000000 0000000000000000 0000000000000000" "0000000000000000 00000000000000ff 0000000000000000 0000000000000000"
    // We expect the last two u64s to be at the beginning of the second slot.
    lt_slot_b: LargerThanSlot = LargerThanSlot::B((0xAB00, 0xBC00, 0xCD00, 0xDE00, 0xEF00)),  // "value": "0000000000000001 000000000000ab00 000000000000bc00 000000000000cd00" "000000000000de00 000000000000ef00 0000000000000000 0000000000000000"
}

impl StorageEnum for Contract {
    #[storage(read, write)]
    fn read_write_enums() -> u64 {
        // Single u8.
        let _ = check_s_u8_a(0xCD);
        storage.s_u8_a.write(SingleU8::A(123));
        let _ = check_s_u8_a(123);
        storage.s_u8_a.write(SingleU8::A(171));
        let _ = check_s_u8_a(171);

        // Single u64.
        let _ = check_s_u64_a(0xAB);
        storage.s_u64_a.write(SingleU64::A(123456));
        let _ = check_s_u64_a(123456);
        storage.s_u64_a.write(SingleU64::A(171));
        let _ = check_s_u64_a(171);

        // Single bool.
        let _ = check_s_bool_a(true);
        storage.s_bool_a.write(SingleBool::A(false));
        let _ = check_s_bool_a(false);
        storage.s_bool_a.write(SingleBool::A(true));
        let _ = check_s_bool_a(true);

        // Multi units.
        let _ = check_m_units_c(MultiUnits::C);
        storage.m_units_c.write(MultiUnits::A);
        let _ = check_m_units_c(MultiUnits::A);
        storage.m_units_c.write(MultiUnits::B);
        let _ = check_m_units_c(MultiUnits::B);
        storage.m_units_c.write(MultiUnits::C);
        let _ = check_m_units_c(MultiUnits::C);

        // Multi one byte.
        let _ = check_m_ob(true, 0xCD);
        storage.m_ob_a.write(MultiOneByte::A(false));
        storage.m_ob_b.write(MultiOneByte::B(123));
        storage.m_ob_c.write(MultiOneByte::C);
        let _ = check_m_ob(false, 123);
        storage.m_ob_a.write(MultiOneByte::B(234));
        storage.m_ob_b.write(MultiOneByte::A(true));
        storage.m_ob_c.write(MultiOneByte::C);
        let _ = check_m_ob(true, 234);
        storage.m_ob_a.write(MultiOneByte::A(true));
        storage.m_ob_b.write(MultiOneByte::B(0xCD));
        storage.m_ob_c.write(MultiOneByte::C);
        let _ = check_m_ob(true, 0xCD);

        // u8 and u64.
        let _ = check_u8_u64(0xAA, 0xBB00);
        storage.u8_u64_a.write(U8AndU64::A(123));
        storage.u8_u64_b.write(U8AndU64::B(123456));
        let _ = check_u8_u64(123, 123456);
        storage.u8_u64_a.write(U8AndU64::B(1234567));
        storage.u8_u64_b.write(U8AndU64::A(231));
        let _ = check_u8_u64(231, 1234567);
        storage.u8_u64_a.write(U8AndU64::A(0xAA));
        storage.u8_u64_b.write(U8AndU64::B(0xBB00));
        let _ = check_u8_u64(0xAA, 0xBB00);

        // Slot size.
        let _ = check_slot_s(0xEE, (0xAA00, 0xBB00, 0xCC00));
        storage.slot_s_a.write(SlotSize::A(123));
        storage.slot_s_b.write(SlotSize::B((123456, 1234567, 12345678)));
        let _ = check_slot_s(123, (123456, 1234567, 12345678));
        storage.slot_s_a.write(SlotSize::B((612345, 7123456, 81234567)));
        storage.slot_s_b.write(SlotSize::A(231));
        let _ = check_slot_s(231, (612345, 7123456, 81234567));
        storage.slot_s_a.write(SlotSize::A(0xEE));
        storage.slot_s_b.write(SlotSize::B((0xAA00, 0xBB00, 0xCC00)));
        let _ = check_slot_s(0xEE, (0xAA00, 0xBB00, 0xCC00));

        // Larger than slot size.
        let _ = check_lt_slot(0xFF, (0xAB00, 0xBC00, 0xCD00, 0xDE00, 0xEF00));
        storage.lt_slot_a.write(LargerThanSlot::A(123));
        storage.lt_slot_b.write(LargerThanSlot::B((123456, 1234567, 12345678, 123456789, 1234567890)));
        let _ = check_lt_slot(123, (123456, 1234567, 12345678, 123456789, 1234567890));
        storage.lt_slot_a.write(LargerThanSlot::B((612345, 723456, 8234567, 912345678, 123456789)));
        storage.lt_slot_b.write(LargerThanSlot::A(231));
        let _ = check_lt_slot(231, (612345, 723456, 8234567, 912345678, 123456789));
        storage.lt_slot_a.write(LargerThanSlot::A(0xFF));
        storage.lt_slot_b.write(LargerThanSlot::B((0xAB00, 0xBC00, 0xCD00, 0xDE00, 0xEF00)));
        let _ = check_lt_slot(0xFF, (0xAB00, 0xBC00, 0xCD00, 0xDE00, 0xEF00));

        171
    }
}

#[storage(read)]
fn check_s_u8_a(expected: u8) -> u8 {
    let s = storage.s_u8_a.read();
    match s {
        SingleU8::A(i) => {
            assert(i == expected);
            return i;
        },
    }
}

#[storage(read)]
fn check_s_u64_a(expected: u64) -> u64 {
    let s = storage.s_u64_a.read();
    match s {
        SingleU64::A(i) => {
            assert(i == expected);
            return i;
        },
    }
}

#[storage(read)]
fn check_s_bool_a(expected: bool) -> u64 {
    let s = storage.s_bool_a.read();
    match s {
        SingleBool::A(i) => {
            assert(i == expected);
            return 171;
        },
    }
}

#[storage(read)]
fn check_m_units_c(expected: MultiUnits) -> u64 {
    let s = storage.m_units_c.read();
    match (s, expected) {
        (MultiUnits::A, MultiUnits::A) => {
            assert(true);
            return 171;
        },
        (MultiUnits::B, MultiUnits::B) => {
            assert(true);
            return 171;
        },
        (MultiUnits::C, MultiUnits::C) => {
            assert(true);
            return 171;
        },
        _ => {
            assert(false);
            return 9999;
        }
    }
}

#[storage(read)]
fn check_m_ob(expected_a: bool, expected_b: u8) -> u64 {
    let a = storage.m_ob_a.read();
    let b = storage.m_ob_b.read();
    let c = storage.m_ob_c.read();
    match (a, b, c) {
        (MultiOneByte::A(a), MultiOneByte::B(b), MultiOneByte::C) => {
            assert(a == expected_a);
            assert(b == expected_b);
            return 171;
        },
        (MultiOneByte::B(b), MultiOneByte::A(a), MultiOneByte::C) => {
            assert(a == expected_a);
            assert(b == expected_b);
            return 171;
        },
        _ => {
            assert(false);
            return 9999;
        }
    }
}

#[storage(read)]
fn check_u8_u64(expected_u8: u8, expected_u64: u64) -> u64 {
    let a = storage.u8_u64_a.read();
    let b = storage.u8_u64_b.read();
    match (a, b) {
        (U8AndU64::A(a), U8AndU64::B(b)) => {
            assert(a == expected_u8);
            assert(b == expected_u64);
            return 171;
        },
        (U8AndU64::B(b), U8AndU64::A(a)) => {
            assert(a == expected_u8);
            assert(b == expected_u64);
            return 171;
        },
        _ => {
            assert(false);
            return 9999;
        }
    }
}

#[storage(read)]
fn check_slot_s(expected_u8: u8, expected_tuple: (u64, u64, u64)) -> u64 {
    let a = storage.slot_s_a.read();
    let b = storage.slot_s_b.read();
    match (a, b) {
        (SlotSize::A(a), SlotSize::B(b)) => {
            assert(a == expected_u8);
            assert(b.0 == expected_tuple.0 && b.1 == expected_tuple.1 && b.2 == expected_tuple.2);
            return 171;
        },
        (SlotSize::B(b), SlotSize::A(a)) => {
            assert(a == expected_u8);
            assert(b.0 == expected_tuple.0 && b.1 == expected_tuple.1 && b.2 == expected_tuple.2);
            return 171;
        },
        _ => {
            assert(false);
            return 9999;
        }
    }
}

#[storage(read)]
fn check_lt_slot(expected_u8: u8, expected_tuple: (u64, u64, u64, u64, u64)) -> u64 {
    let a = storage.lt_slot_a.read();
    let b = storage.lt_slot_b.read();
    match (a, b) {
        (LargerThanSlot::A(a), LargerThanSlot::B(b)) => {
            assert(a == expected_u8);
            assert(b.0 == expected_tuple.0 && b.1 == expected_tuple.1 && b.2 == expected_tuple.2 && b.3 == expected_tuple.3 && b.4 == expected_tuple.4);
            return 171;
        },
        (LargerThanSlot::B(b), LargerThanSlot::A(a)) => {
            assert(a == expected_u8);
            assert(b.0 == expected_tuple.0 && b.1 == expected_tuple.1 && b.2 == expected_tuple.2 && b.3 == expected_tuple.3 && b.4 == expected_tuple.4);
            return 171;
        },
        _ => {
            assert(false);
            return 9999;
        }
    }
}

#[test]
fn collect_storage_enum_contract_gas_usages() {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let _ = caller.read_write_enums();
}
