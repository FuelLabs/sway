// TODO: Replace `assert(x == y)` back with `assert_eq(x, y)` once `assert_eq` no longer
//       causes data-section explosion. See also: https://github.com/FuelLabs/sway/issues/7612
contract;

use test_types::*;

use std::hash::{Hash, sha256};
use std::storage::storage_map::*;

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[cfg(experimental_dynamic_storage = false)]
#[storage(read, write)]
fn assert_remove_remove_existed_impl<K, V>(map: StorageKey<StorageMap<K, V>>, key: K)
where
    K: Hash,
    V: Eq + TestInstance + AbiEncode,
{
    let removed = map.remove(key);
    assert(removed);
    assert(map.get(key).try_read().is_none());
    let removed_again = map.remove(key);
    assert(!removed_again);
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[cfg(experimental_dynamic_storage = true)]
#[storage(read, write)]
fn assert_remove_remove_existed_impl<K, V>(map: StorageKey<StorageMap<K, V>>, key: K)
where
    K: Hash,
    V: Eq + TestInstance + AbiEncode,
{
    let removed = map.remove_existed(key);
    assert(removed);
    assert(map.get(key).try_read().is_none());
    let removed_again = map.remove_existed(key);
    assert(!removed_again);

    map.insert(key, V::typical_value());
    map.remove(key);
    assert(map.get(key).try_read().is_none());
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read, write)]
fn assert_insert_try_insert_get_remove_impl<K, V>(slot_id_preimage: u64, key1: K, key2: K, key3: K)
where
    K: Hash,
    V: Eq + TestInstance + AbiEncode,
{
    assert_non_zero_sized_type::<V>();

    let map: StorageKey<StorageMap<K, V>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage));

    assert(map.get(key1).try_read().is_none());
    assert(map.get(key2).try_read().is_none());

    map.insert(key1, V::typical_value());
    assert(map.get(key1).read() == V::typical_value());
    assert(map.get(key1).try_read() == Some(V::typical_value()));

    assert(map.get(key2).try_read().is_none());

    assert_remove_remove_existed_impl(map, key1);

    let try_insert_result = map.try_insert(key3, V::typical_value());
    assert(match try_insert_result {
        Result::Ok(v) => v == V::typical_value(),
        Result::Err(_) => false,
    });
    assert(map.get(key3).read() == V::typical_value());

    let try_insert_again_result = map.try_insert(key3, V::default());
    assert(match try_insert_again_result {
        Result::Ok(_) => false,
        Result::Err(StorageMapError::OccupiedError(v)) => v == V::typical_value(),
    });
    // The existing value must not have been overwritten.
    assert(map.get(key3).read() == V::typical_value());
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored as a value
    // in a `StorageMap<K, V>`. If the `V` is zero-sized it must be another
    // nested storage type, e.g., `StorageMap<u64, StorageVec<u64>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_insert_try_insert_get_remove() {
        // Key: u64, Value: various types
        assert_insert_try_insert_get_remove_impl::<u64, bool>(1, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, u8>(2, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, u16>(3, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, u32>(4, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, u64>(5, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, u256>(6, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, b256>(7, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, raw_slice>(8, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, str>(9, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, str[2]>(10, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, str[5]>(11, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, str[6]>(12, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, str[8]>(13, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, str[12]>(14, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, str[13]>(15, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, [u64; 2]>(16, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayU8Len2>(17, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayU8Len5>(18, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayU8Len6>(19, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayU8Len8>(20, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayU8Len12>(21, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayU8Len13>(22, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayU64Len3>(23, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayNestedArrayU8Len2Len3>(24, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, ArrayStructBLen2>(25, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, RawPtrNewtype>(26, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, StructA>(27, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, StructB>(28, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, EnumSingleU8>(29, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, EnumSingleU64>(30, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, EnumSingleBool>(31, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, EnumMultiUnits>(32, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, EnumMultiOneByte>(33, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, EnumU8AndU64>(34, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, EnumQuadSlotSize>(35, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, EnumLargerThanQuadSlot>(36, 1, 2, 3);
        assert_insert_try_insert_get_remove_impl::<u64, (u8, u32)>(37, 1, 2, 3);

        // Key: various types (non-u64), Value: same type as the key
        assert_insert_try_insert_get_remove_impl::<bool, bool>(38, false, true, false);
        assert_insert_try_insert_get_remove_impl::<u8, u8>(39, 1u8, 2u8, 3u8);
        assert_insert_try_insert_get_remove_impl::<u16, u16>(40, 1u16, 2u16, 3u16);
        assert_insert_try_insert_get_remove_impl::<u32, u32>(41, 1u32, 2u32, 3u32);
        assert_insert_try_insert_get_remove_impl::<u256, u256>(42, 1u256, 2u256, 3u256);
        assert_insert_try_insert_get_remove_impl::<b256, b256>(
            43,
            0x0000000000000000000000000000000000000000000000000000000000000001u256.as_b256(),
            0x0000000000000000000000000000000000000000000000000000000000000002u256.as_b256(),
            0x0000000000000000000000000000000000000000000000000000000000000003u256.as_b256(),
        );
        assert_insert_try_insert_get_remove_impl::<str[2], str[2]>(
            44,
            __to_str_array("k1"),
            __to_str_array("k2"),
            __to_str_array("k3"),
        );
        assert_insert_try_insert_get_remove_impl::<str[5], str[5]>(
            45,
            __to_str_array("key01"),
            __to_str_array("key02"),
            __to_str_array("key03"),
        );
        assert_insert_try_insert_get_remove_impl::<str[6], str[6]>(
            46,
            __to_str_array("key001"),
            __to_str_array("key002"),
            __to_str_array("key003"),
        );
        assert_insert_try_insert_get_remove_impl::<str[8], str[8]>(
            47,
            __to_str_array("keykey01"),
            __to_str_array("keykey02"),
            __to_str_array("keykey03"),
        );
        assert_insert_try_insert_get_remove_impl::<str[12], str[12]>(
            48,
            __to_str_array("keykey000001"),
            __to_str_array("keykey000002"),
            __to_str_array("keykey000003"),
        );
        assert_insert_try_insert_get_remove_impl::<str[13], str[13]>(
            49,
            __to_str_array("keykey0000001"),
            __to_str_array("keykey0000002"),
            __to_str_array("keykey0000003"),
        );
        assert_insert_try_insert_get_remove_impl::<[u64; 2], [u64; 2]>(50, [1u64; 2], [2u64; 2], [3u64; 2]);
        assert_insert_try_insert_get_remove_impl::<ArrayU8Len2, ArrayU8Len2>(51, [1u8; 2], [2u8; 2], [3u8; 2]);
        assert_insert_try_insert_get_remove_impl::<ArrayU8Len5, ArrayU8Len5>(52, [1u8; 5], [2u8; 5], [3u8; 5]);
        assert_insert_try_insert_get_remove_impl::<ArrayU8Len6, ArrayU8Len6>(53, [1u8; 6], [2u8; 6], [3u8; 6]);
        assert_insert_try_insert_get_remove_impl::<ArrayU8Len8, ArrayU8Len8>(54, [1u8; 8], [2u8; 8], [3u8; 8]);
        assert_insert_try_insert_get_remove_impl::<ArrayU8Len12, ArrayU8Len12>(55, [1u8; 12], [2u8; 12], [3u8; 12]);
        assert_insert_try_insert_get_remove_impl::<ArrayU8Len13, ArrayU8Len13>(56, [1u8; 13], [2u8; 13], [3u8; 13]);
        assert_insert_try_insert_get_remove_impl::<ArrayU64Len3, ArrayU64Len3>(57, [1u64; 3], [2u64; 3], [3u64; 3]);
        assert_insert_try_insert_get_remove_impl::<ArrayNestedArrayU8Len2Len3, ArrayNestedArrayU8Len2Len3>(
            58,
            [[1u8; 2]; 3],
            [[2u8; 2]; 3],
            [[3u8; 2]; 3],
        );
        assert_insert_try_insert_get_remove_impl::<RawPtrNewtype, RawPtrNewtype>(
            59,
            RawPtrNewtype::default(),
            RawPtrNewtype::typical_value(),
            RawPtrNewtype::default(),
        );
        assert_insert_try_insert_get_remove_impl::<StructA, StructA>(
            60,
            StructA::default(),
            StructA::typical_value(),
            StructA::default(),
        );
        assert_insert_try_insert_get_remove_impl::<EnumSingleU8, EnumSingleU8>(
            61,
            EnumSingleU8::A(10u8),
            EnumSingleU8::A(20u8),
            EnumSingleU8::A(30u8),
        );
        assert_insert_try_insert_get_remove_impl::<EnumSingleU64, EnumSingleU64>(
            62,
            EnumSingleU64::A(10u64),
            EnumSingleU64::A(20u64),
            EnumSingleU64::A(30u64),
        );
        assert_insert_try_insert_get_remove_impl::<EnumSingleBool, EnumSingleBool>(
            63,
            EnumSingleBool::A(false),
            EnumSingleBool::A(true),
            EnumSingleBool::A(false),
        );
        assert_insert_try_insert_get_remove_impl::<EnumMultiUnits, EnumMultiUnits>(
            64,
            EnumMultiUnits::A,
            EnumMultiUnits::B,
            EnumMultiUnits::C,
        );
        assert_insert_try_insert_get_remove_impl::<EnumMultiOneByte, EnumMultiOneByte>(
            65,
            EnumMultiOneByte::A(false),
            EnumMultiOneByte::B(1u8),
            EnumMultiOneByte::C,
        );
        assert_insert_try_insert_get_remove_impl::<EnumU8AndU64, EnumU8AndU64>(
            66,
            EnumU8AndU64::A(1u8),
            EnumU8AndU64::B(1u64),
            EnumU8AndU64::A(2u8),
        );
        assert_insert_try_insert_get_remove_impl::<EnumQuadSlotSize, EnumQuadSlotSize>(
            67,
            EnumQuadSlotSize::A(1u8),
            EnumQuadSlotSize::B((1u64, 2u64, 3u64)),
            EnumQuadSlotSize::A(2u8),
        );
        assert_insert_try_insert_get_remove_impl::<EnumLargerThanQuadSlot, EnumLargerThanQuadSlot>(
            68,
            EnumLargerThanQuadSlot::A(1u8),
            EnumLargerThanQuadSlot::B((1u64, 2u64, 3u64, 4u64, 5u64)),
            EnumLargerThanQuadSlot::A(2u8),
        );
        assert_insert_try_insert_get_remove_impl::<(u8, u32), (u8, u32)>(
            69,
            (1u8, 1u32),
            (2u8, 2u32),
            (3u8, 3u32),
        );
    }
}

#[test]
fn insert_try_insert_get_remove() {
    let caller = abi(StorageMapContractTestsAbi, CONTRACT_ID);
    caller.assert_insert_try_insert_get_remove();
}
