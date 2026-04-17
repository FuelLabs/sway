library;

#[storage(read, write)]
pub fn check_args() {
    let _ = __state_clear();
    let _ = __state_clear(b256::zero());
    let _ = __state_clear(42u64, 1u64);
    let _ = __state_clear(b256::zero(), 1u32);
    let _ = __state_clear::<b256, Option<bool>>(b256::zero(), 1u64);

    let _ = __state_clear_slots();
    let _ = __state_clear_slots(b256::zero());
    let _ = __state_clear_slots(42u64, 1u64);
    let _ = __state_clear_slots(b256::zero(), 1u32);
    let _ = __state_clear_slots::<b256, Option<bool>>(b256::zero(), 1u64);

    let _ = __state_store_word();
    let _ = __state_store_word(b256::zero());
    let _ = __state_store_word(42u64, 1u64);
    let _ = __state_store_word(b256::zero(), 1u32);
    let _ = __state_store_word::<b256, Option<bool>>(b256::zero(), 1u64);

    check_load_word_args();

    let _ = __state_load_quad();
    let _ = __state_load_quad(b256::zero());
    let _ = __state_load_quad(42u64, __addr_of(0), 1u64);
    let _ = __state_load_quad(b256::zero(), 0u32, 1u64);
    let _ = __state_load_quad(b256::zero(), __addr_of(0), 1u32);
    let _ = __state_load_quad::<b256, Option<bool>>(b256::zero(), __addr_of(0), 1u64);

    let _ = __state_store_quad();
    let _ = __state_store_quad(b256::zero());
    let _ = __state_store_quad(42u64, __addr_of(0), 1u64);
    let _ = __state_store_quad(b256::zero(), 0u32, 1u64);
    let _ = __state_store_quad(b256::zero(), __addr_of(0), 1u32);
    let _ = __state_store_quad::<b256, Option<bool>>(b256::zero(), __addr_of(0), 1u64);

    let _ = __state_load_slot();
    let _ = __state_load_slot(b256::zero());
    let _ = __state_load_slot(42u64, __addr_of(0), 0u64, 1u64);
    let _ = __state_load_slot(b256::zero(), 0u32, 0u64, 1u64);
    let _ = __state_load_slot(b256::zero(), __addr_of(0), 0u32, 1u64);
    let _ = __state_load_slot(b256::zero(), __addr_of(0), 0u64, 1u32);
    let _ = __state_load_slot::<b256, Option<bool>>(b256::zero(), __addr_of(0),  0u64, 1u64);

    let _ = __state_store_slot();
    let _ = __state_store_slot(b256::zero());
    let _ = __state_store_slot(42u64, __addr_of(0), 1u64);
    let _ = __state_store_slot(b256::zero(), 0u32, 1u64);
    let _ = __state_store_slot(b256::zero(), __addr_of(0), 1u32);
    let _ = __state_store_slot::<b256, Option<bool>>(b256::zero(), __addr_of(0), 1u64);

    let _ = __state_update_slot();
    let _ = __state_update_slot(b256::zero());
    let _ = __state_update_slot(42u64, __addr_of(0), 0u64, 1u64);
    let _ = __state_update_slot(b256::zero(), 0u32, 0u64, 1u64);
    let _ = __state_update_slot(b256::zero(), __addr_of(0), 0u32, 1u64);
    let _ = __state_update_slot(b256::zero(), __addr_of(0), 0u64, 1u32);
    let _ = __state_update_slot::<b256, Option<bool>>(b256::zero(), __addr_of(0),  0u64, 1u64);

    let _ = __state_preload();
    let _ = __state_preload(b256::zero(), 1u64);
    let _ = __state_preload::<b256, Option<bool>>(b256::zero());
}

#[storage(read)]
#[cfg(experimental_dynamic_storage = false)]
fn check_load_word_args() {
    let _ = __state_load_word();
    let _ = __state_load_word(b256::zero(), 1u64);
    let _ = __state_load_word::<b256, Option<bool>>(b256::zero());
}

#[storage(read)]
#[cfg(experimental_dynamic_storage = true)]
fn check_load_word_args() {
    let _ = __state_load_word();
    let _ = __state_load_word(b256::zero());
    let _ = __state_load_word(42u64, 1u64);
    let _ = __state_load_word(b256::zero(), 1u32);
    let _ = __state_load_word::<b256, Option<bool>>(b256::zero(), 1u64);
}
