// TODO: The initial reason for having several `storage_vec` tests is
//       reaching the limit in the data section size of a single contract
//       that contained all the tests for `StorageVec<T>` methods.
//       See: https://github.com/FuelLabs/sway/issues/7612
//       But even when the issue is solved, it perhaps still makes sense to have
//       separate test projects for better organization and readability
//       of the tests.
// TODO: Replace `assert(x == y)` back with `assert_eq(x, y)` once `assert_eq` no longer
//       causes data-section explosion. See also: https://github.com/FuelLabs/sway/issues/7612
contract;

use test_types::*;

use std::hash::{Hash, sha256};
use std::storage::storage_vec::*;

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
const NUM_OF_NESTED_VECTORS: u64 = 7;
#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
const NUM_OF_ELEMENTS: u64 = 11;

storage {
    //vec_of_vec: StorageVec<StorageVec<u64>> = StorageVec {},
}

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
#[storage(read, write)]
fn assert_push_and_get_impl<T>(
    slot_id_preimage: u64,
)
where
    T: Eq + TestInstance + AbiEncode,
{
    let vec: StorageKey<StorageVec<StorageVec<T>>> = StorageKey::new(sha256(slot_id_preimage), 0, sha256(slot_id_preimage + 100));

    let mut i = 0;
    while i < NUM_OF_NESTED_VECTORS {
        vec.push(StorageVec {});
        i += 1;
    }

    assert(NUM_OF_NESTED_VECTORS == vec.len());

    let expected_values = T::instances(NUM_OF_ELEMENTS);
    let mut i = 0;
    while i < NUM_OF_NESTED_VECTORS {
        let nested_vec = vec.get(i).unwrap();
            let mut j = 0;
            while j < NUM_OF_ELEMENTS {
                nested_vec.push(expected_values.get(j).unwrap());
                j += 1;
            }
        i += 1;
    }

    let mut i = 0;
    while i < NUM_OF_NESTED_VECTORS {
        let nested_vec = vec.get(i).unwrap();

        let mut j = 0;
        while j < NUM_OF_ELEMENTS {
            assert(expected_values.get(j).unwrap() == nested_vec.get(j).unwrap().read());
            j += 1;
        }

        i += 1;
    }
}

impl Contract {
    // Note that zero-sized types like, e.g., `()`, `[u64;0]`, or `EmptyStruct`,
    // by definition of the storage access semantics, cannot be stored in
    // a `StorageVec<T>`. If the `T` is zero-sized it must be an another
    // nested storage type, e.g., `StorageVec<StorageMap<u64, b256>>`.
    // So, in all the tests below, we don't have zero-sized types.

    #[storage(read, write)]
    fn assert_push_and_get() {
        assert_push_and_get_impl::<bool>(1);
        assert_push_and_get_impl::<u8>(2);
        assert_push_and_get_impl::<u16>(3);
        assert_push_and_get_impl::<u32>(4);
        assert_push_and_get_impl::<u64>(5);
        assert_push_and_get_impl::<u256>(6);
        assert_push_and_get_impl::<b256>(7);
        assert_push_and_get_impl::<raw_slice>(8);
        assert_push_and_get_impl::<str>(9);
        assert_push_and_get_impl::<str[2]>(10);
        assert_push_and_get_impl::<str[5]>(11);
        assert_push_and_get_impl::<str[6]>(12);
        assert_push_and_get_impl::<str[8]>(13);
        assert_push_and_get_impl::<str[12]>(14);
        assert_push_and_get_impl::<str[13]>(15);
        assert_push_and_get_impl::<[u64; 2]>(16);
        assert_push_and_get_impl::<ArrayU8Len2>(17);
        assert_push_and_get_impl::<ArrayU8Len5>(18);
        assert_push_and_get_impl::<ArrayU8Len6>(19);
        assert_push_and_get_impl::<ArrayU8Len8>(20);
        assert_push_and_get_impl::<ArrayU8Len12>(21);
        assert_push_and_get_impl::<ArrayU8Len13>(22);
        assert_push_and_get_impl::<ArrayU64Len3>(23);
        assert_push_and_get_impl::<ArrayNestedArrayU8Len2Len3>(24);
        assert_push_and_get_impl::<ArrayStructBLen2>(25);
        assert_push_and_get_impl::<RawPtrNewtype>(26);
        assert_push_and_get_impl::<StructA>(27);
        assert_push_and_get_impl::<StructB>(28);
        assert_push_and_get_impl::<EnumSingleU8>(29);
        assert_push_and_get_impl::<EnumSingleU64>(30);
        assert_push_and_get_impl::<EnumSingleBool>(31);
        assert_push_and_get_impl::<EnumMultiUnits>(32);
        assert_push_and_get_impl::<EnumMultiOneByte>(33);
        assert_push_and_get_impl::<EnumU8AndU64>(34);
        assert_push_and_get_impl::<EnumQuadSlotSize>(35);
        assert_push_and_get_impl::<EnumLargerThanQuadSlot>(36);
        assert_push_and_get_impl::<(u8, u32)>(37);
    }
}

#[test]
fn push_and_get() {
    let contract_abi = abi(StorageVecOfStorageVecContractTestsAbi, CONTRACT_ID);
    contract_abi.assert_push_and_get();
}
