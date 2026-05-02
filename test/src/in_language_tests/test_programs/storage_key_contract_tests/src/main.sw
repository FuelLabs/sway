// TODO: Replace `assert(x == y)` back with `assert_eq(x, y)` once `assert_eq` no longer
//       causes data-section explosion. See also: https://github.com/FuelLabs/sway/issues/7612
contract;

use test_types::*;

use std::hash::{Hash, sha256};

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[cfg(experimental_dynamic_storage = false)]
#[storage(read, write)]
fn assert_clear_clear_existed_impl<T>(key: StorageKey<T>)
where
    T: Eq + TestInstance + AbiEncode,
{
    let existed = key.clear();
    assert(existed);
    assert(key.try_read().is_none());
    let existed_again = key.clear();
    assert(!existed_again);
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[cfg(experimental_dynamic_storage = true)]
#[storage(read, write)]
fn assert_clear_clear_existed_impl<T>(key: StorageKey<T>)
where
    T: Eq + TestInstance + AbiEncode,
{
    let existed = key.clear_existed();
    assert(existed);
    assert(key.try_read().is_none());

    let existed_again = key.clear_existed();
    assert(!existed_again);

    key.write(T::typical_value());
    key.clear();
    assert(key.try_read().is_none());
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read, write)]
fn assert_write_read_try_read_clear_clear_existed_impl<T>(slot_id_preimage: u64)
where
    T: Eq + TestInstance + AbiEncode,
{
    assert_non_zero_sized_type::<T>();

    let key: StorageKey<T> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage));

    assert(key.try_read().is_none());

    key.write(T::typical_value());
    assert(key.read() == T::typical_value());
    assert(key.try_read() == Some(T::typical_value()));

    assert_clear_clear_existed_impl(key);
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_write_read_try_read_clear_clear_existed() {
        assert_write_read_try_read_clear_clear_existed_impl::<bool>(1);
        assert_write_read_try_read_clear_clear_existed_impl::<u8>(2);
        assert_write_read_try_read_clear_clear_existed_impl::<u16>(3);
        assert_write_read_try_read_clear_clear_existed_impl::<u32>(4);
        assert_write_read_try_read_clear_clear_existed_impl::<u64>(5);
        assert_write_read_try_read_clear_clear_existed_impl::<u256>(6);
        assert_write_read_try_read_clear_clear_existed_impl::<b256>(7);
        assert_write_read_try_read_clear_clear_existed_impl::<raw_slice>(8);
        assert_write_read_try_read_clear_clear_existed_impl::<str>(9);
        assert_write_read_try_read_clear_clear_existed_impl::<str[2]>(10);
        assert_write_read_try_read_clear_clear_existed_impl::<str[5]>(11);
        assert_write_read_try_read_clear_clear_existed_impl::<str[6]>(12);
        assert_write_read_try_read_clear_clear_existed_impl::<str[8]>(13);
        assert_write_read_try_read_clear_clear_existed_impl::<str[12]>(14);
        assert_write_read_try_read_clear_clear_existed_impl::<str[13]>(15);
        assert_write_read_try_read_clear_clear_existed_impl::<[u64; 2]>(16);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayU8Len2>(17);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayU8Len5>(18);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayU8Len6>(19);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayU8Len8>(20);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayU8Len12>(21);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayU8Len13>(22);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayU64Len3>(23);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_write_read_try_read_clear_clear_existed_impl::<ArrayStructBLen2>(25);
        assert_write_read_try_read_clear_clear_existed_impl::<RawPtrNewtype>(26);
        assert_write_read_try_read_clear_clear_existed_impl::<StructA>(27);
        assert_write_read_try_read_clear_clear_existed_impl::<StructB>(28);
        assert_write_read_try_read_clear_clear_existed_impl::<EnumSingleU8>(29);
        assert_write_read_try_read_clear_clear_existed_impl::<EnumSingleU64>(30);
        assert_write_read_try_read_clear_clear_existed_impl::<EnumSingleBool>(31);
        assert_write_read_try_read_clear_clear_existed_impl::<EnumMultiUnits>(32);
        assert_write_read_try_read_clear_clear_existed_impl::<EnumMultiOneByte>(33);
        assert_write_read_try_read_clear_clear_existed_impl::<EnumU8AndU64>(34);
        assert_write_read_try_read_clear_clear_existed_impl::<EnumQuadSlotSize>(35);
        assert_write_read_try_read_clear_clear_existed_impl::<EnumLargerThanQuadSlot>(36);
        assert_write_read_try_read_clear_clear_existed_impl::<(u8, u32)>(37);
    }
}

#[test]
fn write_read_try_read_clear_clear_existed() {
    let caller = abi(StorageKeyContractTestsAbi, CONTRACT_ID);
    caller.assert_write_read_try_read_clear_clear_existed();
}
