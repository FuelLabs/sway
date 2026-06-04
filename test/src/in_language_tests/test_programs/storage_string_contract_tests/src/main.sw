// TODO: Replace `assert(x == y)` back with `assert_eq(x, y)` once `assert_eq` no longer
//       causes data-section explosion. See also: https://github.com/FuelLabs/sway/issues/7612
contract;

use std::hash::sha256;
use std::storage::storage_string::*;
use std::string::String;

// In `experimental_dynamic_storage = false` mode, `clear()` returns `bool`.
// Note: calling `clear()` when no string is stored also returns `true` because
// `__state_clear` with `slots = 0` always returns `true` (see compiler intrinsic docs).
// The meaningful assertion is therefore that `clear()` returns `true` after storing a string,
// confirming the content slots were actually set.
#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[cfg(experimental_dynamic_storage = false)]
#[storage(read, write)]
fn assert_clear_clear_existed_impl(storage_string: StorageKey<StorageString>) {
    let cleared = storage_string.clear();
    assert(cleared);
    assert(storage_string.read_slice().is_none());
    assert(storage_string.len() == 0);
}

// In `experimental_dynamic_storage = true` mode, `clear()` is void and `clear_existed()`
// returns `bool`. `clear_existed()` reliably returns `false` when the slot is already empty.
#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[cfg(experimental_dynamic_storage = true)]
#[storage(read, write)]
fn assert_clear_clear_existed_impl(storage_string: StorageKey<StorageString>) {
    let existed = storage_string.clear_existed();
    assert(existed);
    assert(storage_string.read_slice().is_none());
    assert(storage_string.len() == 0);

    let existed_again = storage_string.clear_existed();
    assert(!existed_again);

    // Also test the void `clear()`.
    storage_string.write_slice(String::from_ascii_str("x"));
    storage_string.clear();
    assert(storage_string.read_slice().is_none());
    assert(storage_string.len() == 0);
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read, write)]
fn assert_write_read_slice_len_clear_clear_existed_impl(
    slot_id_preimage: u64,
    string: String,
    overwrite_string: String,
) {
    let storage_string: StorageKey<StorageString> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage));

    // Initially empty.
    assert(storage_string.read_slice().is_none());
    assert(storage_string.len() == 0);

    // Write and read back.
    storage_string.write_slice(string);
    assert(storage_string.read_slice().unwrap() == string);
    assert(storage_string.len() == string.len());

    // Overwrite with a different string and read back.
    storage_string.write_slice(overwrite_string);
    assert(storage_string.read_slice().unwrap() == overwrite_string);
    assert(storage_string.len() == overwrite_string.len());

    // Test clear (and clear_existed in dynamic mode).
    assert_clear_clear_existed_impl(storage_string);
}

impl Contract {
    #[storage(read, write)]
    fn assert_write_read_slice_len_clear_clear_existed() {
        // 1 character (1 byte in UTF-8).
        assert_write_read_slice_len_clear_clear_existed_impl(
            1,
            String::from_ascii_str("x"),
            String::from_ascii_str("y"),
        );

        // 3 characters (less than one legacy quad slot).
        assert_write_read_slice_len_clear_clear_existed_impl(
            2,
            String::from_ascii_str("foo"),
            String::from_ascii_str("bar"),
        );

        // 22 characters (less than one legacy quad slot).
        assert_write_read_slice_len_clear_clear_existed_impl(
            3,
            String::from_ascii_str("Fuel is blazingly fast"),
            String::from_ascii_str("Sway is blazingly fast"),
        );

        // 32 characters (exactly one full legacy quad slot).
        assert_write_read_slice_len_clear_clear_existed_impl(
            4,
            String::from_ascii_str("abcdefghijklmnopqrstuvwxyz012345"),
            String::from_ascii_str("543210zyxwvutsrqponmlkjihgfedcba"),
        );

        // 35 characters (crosses a legacy quad slot boundary), overwritten with a shorter string.
        assert_write_read_slice_len_clear_clear_existed_impl(
            5,
            String::from_ascii_str("abcdefghijklmnopqrstuvwxyz012345678"),
            String::from_ascii_str("hi"),
        );
    }
}

#[test]
fn write_read_slice_len_clear_clear_existed() {
    let caller = abi(StorageStringContractTestsAbi, CONTRACT_ID);
    caller.assert_write_read_slice_len_clear_clear_existed();
}
