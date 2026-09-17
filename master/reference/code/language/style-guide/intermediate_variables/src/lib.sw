library;

#[allow(dead_code)]
fn update_state() -> u64 {
    // Used for context in the following function
    42
}

#[allow(dead_code)]
// ANCHOR: contextual_assignment
fn contextual_assignment() {
    let remaining_amount = update_state();
    // code that uses `remaining_amount` instead of directly calling `update_state()`
}
// ANCHOR_END: contextual_assignment

#[allow(dead_code)]
fn update_state_of_vault_v3_storage_contract() -> u64 {
    // Used for context in the following function
    42
}

#[allow(dead_code)]
// ANCHOR: shortened_name
fn shortened_name() {
    let remaining_amount = update_state_of_vault_v3_storage_contract();
    // code that uses `remaining_amount` instead of directly calling `update_state_of_vault_v3_storage_contract()`
}
// ANCHOR_END: shortened_name
