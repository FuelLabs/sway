library;

#[test]
fn storage_key_slot() {
    let key_1 = StorageKey::<u64>::new(b256::min(), u64::zero(), b256::zero());
    assert(key_1.slot() == b256::min());

    let key_2 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        u64::zero(),
        b256::zero(),
    );
    assert(
        key_2
            .slot() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let key_3 = StorageKey::<u64>::new(b256::max(), u64::zero(), b256::zero());
    assert(key_3.slot() == b256::max());
}

#[test]
fn storage_key_offset() {
    let key_1 = StorageKey::<u64>::new(b256::zero(), u64::min(), b256::zero());
    assert(key_1.offset() == u64::min());

    let key_2 = StorageKey::<u64>::new(b256::zero(), 1, b256::zero());
    assert(key_2.offset() == 1);

    let key_3 = StorageKey::<u64>::new(b256::zero(), u64::max(), b256::zero());
    assert(key_3.offset() == u64::max());
}

#[test]
fn storage_key_field_id() {
    let key_1 = StorageKey::<u64>::new(b256::zero(), u64::zero(), b256::min());
    assert(key_1.field_id() == b256::min());

    let key_2 = StorageKey::<u64>::new(
        b256::zero(),
        u64::zero(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        key_2
            .field_id() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let key_3 = StorageKey::<u64>::new(b256::zero(), u64::zero(), b256::max());
    assert(key_3.field_id() == b256::max());
}

#[test]
fn storage_key_new() {
    let key = StorageKey::<u64>::new(b256::min(), u64::min(), b256::min());
    assert(key.slot() == b256::min());
    assert(key.offset() == u64::min());
    assert(key.field_id() == b256::min());

    let key = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        1,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        key
            .slot() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(key.offset() == 1);
    assert(
        key
            .field_id() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );

    let key = StorageKey::<u64>::new(b256::max(), u64::max(), b256::max());
    assert(key.slot() == b256::max());
    assert(key.offset() == u64::max());
    assert(key.field_id() == b256::max());
}

#[test]
fn storage_key_zero() {
    let storage_key = StorageKey::<u64>::zero();
    assert(
        storage_key
            .slot() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(storage_key.offset() == 0);
    assert(
        storage_key
            .field_id() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn storage_key_is_zero() {
    let zero_storage_key = StorageKey::<u64>::zero();
    assert(zero_storage_key.is_zero());

    let storage_key_2 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(!storage_key_2.is_zero());

    let storage_key_3 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000000,
        1,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(!storage_key_3.is_zero());

    let storage_key_4 = StorageKey::<u64>::new(
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(!storage_key_4.is_zero());

    let storage_key_5 = StorageKey::<u64>::new(b256::max(), u64::max(), b256::max());
    assert(!storage_key_5.is_zero());
}
