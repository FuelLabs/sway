// TODO: Replace `assert(x == y)` back with `assert_eq(x, y)` once `assert_eq` no longer
//       causes data-section explosion. See also: https://github.com/FuelLabs/sway/issues/7612
contract;

use std::bytes::Bytes;
use std::hash::sha256;
use std::storage::storage_bytes::*;

// In `experimental_dynamic_storage = false` mode, `clear()` returns `bool`.
// Note: calling `clear()` when no bytes are stored also returns `true` because
// `__state_clear` with `slots = 0` always returns `true` (see compiler intrinsic docs).
// The meaningful assertion is therefore that `clear()` returns `true` after storing bytes,
// confirming the content slots were actually set.
#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[cfg(experimental_dynamic_storage = false)]
#[storage(read, write)]
fn assert_clear_clear_existed_impl(storage_bytes: StorageKey<StorageBytes>) {
    let cleared = storage_bytes.clear();
    assert(cleared);
    assert(storage_bytes.read_slice().is_none());
    assert(storage_bytes.len() == 0);
}

// In `experimental_dynamic_storage = true` mode, `clear()` is void and `clear_existed()`
// returns `bool`. `clear_existed()` reliably returns `false` when the slot is already empty.
#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[cfg(experimental_dynamic_storage = true)]
#[storage(read, write)]
fn assert_clear_clear_existed_impl(storage_bytes: StorageKey<StorageBytes>) {
    let existed = storage_bytes.clear_existed();
    assert(existed);
    assert(storage_bytes.read_slice().is_none());
    assert(storage_bytes.len() == 0);

    let existed_again = storage_bytes.clear_existed();
    assert(!existed_again);

    // Also test the void `clear()`.
    let mut new_bytes = Bytes::new();
    new_bytes.push(1u8);
    storage_bytes.write_slice(new_bytes);
    storage_bytes.clear();
    assert(storage_bytes.read_slice().is_none());
    assert(storage_bytes.len() == 0);
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read, write)]
fn assert_write_read_slice_len_clear_clear_existed_impl(
    slot_id_preimage: u64,
    bytes: Bytes,
    overwrite_bytes: Bytes,
) {
    let storage_bytes: StorageKey<StorageBytes> = StorageKey::new( sha256(slot_id_preimage), 0, sha256(slot_id_preimage));

    // Initially empty.
    assert(storage_bytes.read_slice().is_none());
    assert(storage_bytes.len() == 0);

    // Write and read back.
    storage_bytes.write_slice(bytes);
    assert(storage_bytes.read_slice().unwrap() == bytes);
    assert(storage_bytes.len() == bytes.len());

    // Overwrite with different bytes and read back.
    storage_bytes.write_slice(overwrite_bytes);
    assert(storage_bytes.read_slice().unwrap() == overwrite_bytes);
    assert(storage_bytes.len() == overwrite_bytes.len());

    // Test clear (and clear_existed in dynamic mode).
    assert_clear_clear_existed_impl(storage_bytes);
}

impl Contract {
    #[storage(read, write)]
    fn assert_write_read_slice_len_clear_clear_existed() {
        // 1 byte.
        let mut bytes_1 = Bytes::new();
        bytes_1.push(42u8);
        let mut overwrite_bytes_1 = Bytes::new();
        overwrite_bytes_1.push(99u8);
        assert_write_read_slice_len_clear_clear_existed_impl(1, bytes_1, overwrite_bytes_1);

        // 3 bytes (less than one legacy quad slot).
        let mut bytes_3 = Bytes::new();
        bytes_3.push(1u8);
        bytes_3.push(2u8);
        bytes_3.push(3u8);
        let mut overwrite_bytes_3 = Bytes::new();
        overwrite_bytes_3.push(4u8);
        overwrite_bytes_3.push(5u8);
        overwrite_bytes_3.push(6u8);
        overwrite_bytes_3.push(7u8);
        assert_write_read_slice_len_clear_clear_existed_impl(2, bytes_3, overwrite_bytes_3);

        // 32 bytes (exactly one full legacy quad slot).
        let bytes_32 = Bytes::from(0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
        let overwrite_bytes_32 = Bytes::from(0x201f1e1d1c1b1a191817161514131211100f0e0d0c0b0a090807060504030201);
        assert_write_read_slice_len_clear_clear_existed_impl(3, bytes_32, overwrite_bytes_32);

        // 35 bytes (crosses a legacy quad slot boundary), overwritten with 2 bytes (shorter).
        let mut bytes_35 = Bytes::from(0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
        bytes_35.push(33u8);
        bytes_35.push(34u8);
        bytes_35.push(35u8);
        let mut overwrite_bytes_35 = Bytes::new();
        overwrite_bytes_35.push(11u8);
        overwrite_bytes_35.push(22u8);
        assert_write_read_slice_len_clear_clear_existed_impl(4, bytes_35, overwrite_bytes_35);
    }
}

#[test]
fn write_read_slice_len_clear_clear_existed() {
    let caller = abi(StorageBytesAbi, CONTRACT_ID);
    caller.assert_write_read_slice_len_clear_clear_existed();
}
