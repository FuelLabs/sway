library;

use std::alloc::*;

#[test]
fn alloc_alloc() {
    // Alloc zero
    let zero_ptr = alloc::<u64>(0);
    assert(!zero_ptr.is_null());

    // Can alloc u8
    let u8_ptr = alloc::<u8>(1);
    assert(u8_ptr.read::<u8>() == 0u8);
    u8_ptr.write(u8::max());
    assert(u8_ptr.read::<u8>() == u8::max());

    // Can alloc u64
    let u64_ptr = alloc::<u64>(1);
    assert(u64_ptr.read::<u64>() == 0u64);
    u64_ptr.write(u64::max());
    assert(u64_ptr.read::<u64>() == u64::max());

    // Can alloc b256
    let b256_ptr = alloc::<b256>(1);
    assert(b256_ptr.read::<b256>() == b256::zero());
    b256_ptr.write(b256::max());
    assert(b256_ptr.read::<b256>() == b256::max());

    // Can alloc struct
    let address_ptr = alloc::<Address>(1);
    assert(address_ptr.read::<Address>() == Address::zero());
    address_ptr.write(Address::from(b256::max()));
    assert(address_ptr.read::<Address>() == Address::from(b256::max()));

    // Can alloc multiple
    let count = 1000;
    let multiple_u64 = alloc::<u64>(count);
    let mut iter = 0;
    while iter < count {
        assert(multiple_u64.add::<u64>(iter).read::<u64>() == 0u64);
        iter += 1;
    }

    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // Can alloc array
    // let array_ptr = alloc::<[u64; 1]>(2);
    // assert(array_ptr.read::<u64>() == 0u64);
    // assert(array_ptr.add::<u64>(1).read::<u64>() == 0u64);
    // array_ptr.write(u64::max());
    // array_ptr.add::<u64>(1).write(u64::max());
    // assert(array_ptr.read::<u64>() == u64::max());
    // assert(array_ptr.add::<u64>(1).read::<u64>() == u64::max());
}

#[test(should_revert)]
fn revert_alloc_alloc_does_not_exceed_bounds() {
    let u64_ptr = alloc::<u64>(1);
    assert(u64_ptr.read::<u64>() == 0u64);

    let out_of_bounds = u64_ptr.add::<u64>(1).read::<u64>();
    log(out_of_bounds);
}

#[test]
fn alloc_realloc() {
    // realloc from zero
    let zero_ptr = alloc::<u64>(0);
    let realloc_zero_ptr = realloc::<u64>(zero_ptr, 0, 1);
    assert(realloc_zero_ptr.read::<u64>() == u64::zero());

    // Can realloc u8
    let u8_ptr = alloc::<u8>(1);
    u8_ptr.write(u8::max());
    let realloc_u8_ptr = realloc::<u8>(u8_ptr, 1, 2);
    assert(realloc_u8_ptr.read::<u8>() == u8::max());
    assert(realloc_u8_ptr.add::<u8>(1).read::<u8>() == u8::zero());

    // Can alloc u64
    let u64_ptr = alloc::<u64>(1);
    u64_ptr.write(u64::max());
    let realloc_u64_ptr = realloc::<u64>(u64_ptr, 1, 2);
    assert(realloc_u64_ptr.read::<u64>() == u64::max());
    assert(realloc_u64_ptr.add::<u64>(1).read::<u64>() == u64::zero());

    // Can alloc b256
    let b256_ptr = alloc::<b256>(1);
    b256_ptr.write(b256::max());
    let realloc_b256_ptr = realloc::<b256>(b256_ptr, 1, 2);
    assert(realloc_b256_ptr.read::<b256>() == b256::max());
    assert(realloc_b256_ptr.add::<b256>(1).read::<b256>() == b256::zero());

    // Can alloc struct
    let address_ptr = alloc::<Address>(1);
    address_ptr.write(Address::from(b256::max()));
    let realloc_address_ptr = realloc::<Address>(address_ptr, 1, 2);
    assert(realloc_address_ptr.read::<Address>() == Address::from(b256::max()));
    assert(realloc_address_ptr.add::<Address>(1).read::<Address>() == Address::zero());

    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // Can realloc array
    // let array_ptr = alloc::<[u64; 1]>(2);
    // array_ptr.write(u64::max());
    // array_ptr.add::<u64>(1).write(u64::max());
    // let realloc_array_ptr = realloc::<[u64; 1]>(array_ptr, 2, 3);
    // assert(realloc_array_ptr.read::<u64>() == u64::max());
    // assert(realloc_array_ptr.add::<u64>(1).read::<u64>() == u64::max());
    // assert(realloc_array_ptr.add::<u64>(2).read::<u64>() == u64::zero());

    // Can alloc multiple
    let count = 100;
    let recount = 1000;

    let multiple_u64 = alloc::<u64>(count);
    let mut iter = 0;
    while iter < count {
        multiple_u64.add::<u64>(iter).write(u64::max());
        iter += 1;
    }

    let realloc_multiple_u64 = realloc::<u64>(multiple_u64, count, recount);
    let mut iter2 = 0;
    while iter2 < count {
        assert(realloc_multiple_u64.add::<u64>(iter2).read::<u64>() == u64::max());
        iter2 += 1;
    }
    let mut iter3 = count;
    while iter3 < recount {
        assert(realloc_multiple_u64.add::<u64>(iter3).read::<u64>() == 0u64);
        iter3 += 1;
    }

    // Edge cases

    // Realloc to same size
    let same_u64_ptr = alloc::<u64>(2);
    same_u64_ptr.write(u64::max());
    let same_realloc_u64_ptr = realloc::<u64>(same_u64_ptr, 2, 2);
    assert(same_realloc_u64_ptr.read::<u64>() == u64::max());
    assert(same_realloc_u64_ptr.add::<u64>(1).read::<u64>() == u64::zero());
    assert(same_realloc_u64_ptr == same_u64_ptr);

    // Realloc to less size
    let less_u64_ptr = alloc::<u64>(2);
    less_u64_ptr.write(u64::max());
    let less_realloc_u64_ptr = realloc::<u64>(less_u64_ptr, 2, 1);
    assert(less_realloc_u64_ptr.read::<u64>() == u64::max());
    assert(less_realloc_u64_ptr.add::<u64>(1).read::<u64>() == u64::zero());
    assert(less_realloc_u64_ptr == less_u64_ptr);

    // Realloc excludes values when count is less then total allocated
    let exclude_u64_ptr = alloc::<u64>(2);
    exclude_u64_ptr.write(u64::max());
    exclude_u64_ptr.add::<u64>(1).write(u64::max());
    let exclude_realloc_u64_ptr = realloc::<u64>(exclude_u64_ptr, 1, 2);
    assert(exclude_realloc_u64_ptr.read::<u64>() == u64::max());
    assert(exclude_realloc_u64_ptr.add::<u64>(1).read::<u64>() == u64::zero());
}

#[test(should_revert)]
fn revert_alloc_realloc_when_realloc_unallocated_memory() {
    let u64_ptr = alloc::<u64>(1);
    u64_ptr.write(u64::max());

    let _realloc_b256_ptr = realloc::<u64>(u64_ptr, 2, 3);
}

#[test]
fn alloc_alloc_bytes() {
    // Alloc zero
    let zero_ptr = alloc_bytes(0);
    assert(!zero_ptr.is_null());

    // Can alloc u8
    let u8_ptr = alloc_bytes(1);
    assert(u8_ptr.read::<u8>() == 0u8);
    u8_ptr.write(u8::max());
    assert(u8_ptr.read::<u8>() == u8::max());

    // Can alloc u64
    let u64_ptr = alloc_bytes(__size_of::<u64>());
    assert(u64_ptr.read::<u64>() == 0u64);
    u64_ptr.write(u64::max());
    assert(u64_ptr.read::<u64>() == u64::max());

    // Can alloc b256
    let b256_ptr = alloc_bytes(__size_of::<b256>());
    assert(b256_ptr.read::<b256>() == b256::zero());
    b256_ptr.write(b256::max());
    assert(b256_ptr.read::<b256>() == b256::max());

    // Can alloc struct
    let address_ptr = alloc_bytes(__size_of_val::<Address>(Address::zero()));
    assert(address_ptr.read::<Address>() == Address::zero());
    address_ptr.write(Address::from(b256::max()));
    assert(address_ptr.read::<Address>() == Address::from(b256::max()));

    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved 
    // Can alloc array
    // let array_ptr = alloc_bytes(__size_of::<[u64; 1]>() * 2);
    // assert(array_ptr.read::<u64>() == 0u64);
    // assert(array_ptr.add::<u64>(1).read::<u64>() == 0u64);
    // array_ptr.write(u64::max());
    // array_ptr.add::<u64>(1).write(u64::max());
    // assert(array_ptr.read::<u64>() == u64::max());
    // assert(array_ptr.add::<u64>(1).read::<u64>() == u64::max());

    // Can alloc multiple
    let count = 1000;
    let multiple_u64 = alloc_bytes(__size_of::<u64>() * count);
    let mut iter = 0;
    while iter < count {
        assert(multiple_u64.add::<u64>(iter).read::<u64>() == 0u64);
        iter += 1;
    }
}

#[test(should_revert)]
fn revert_alloc_alloc_bytes_does_not_exceed_bounds() {
    let u64_ptr = alloc_bytes(__size_of::<u64>());
    assert(u64_ptr.read::<u64>() == 0u64);

    let out_of_bounds = u64_ptr.add::<u64>(1).read::<u64>();
    log(out_of_bounds);
}

#[test]
fn alloc_realloc_bytes() {
    // realloc from zero
    let zero_ptr = alloc_bytes(0);
    let realloc_zero_ptr = realloc_bytes(zero_ptr, 0, 1);
    assert(realloc_zero_ptr.read::<u8>() == u8::zero());

    // Can realloc u8
    let u8_ptr = alloc_bytes(1);
    u8_ptr.write(u8::max());
    let realloc_u8_ptr = realloc_bytes(u8_ptr, 1, 2);
    assert(realloc_u8_ptr.read::<u8>() == u8::max());
    assert(realloc_u8_ptr.add::<u8>(1).read::<u8>() == u8::zero());

    // Can alloc u64
    let u64_ptr = alloc_bytes(__size_of::<u64>());
    u64_ptr.write(u64::max());
    let realloc_u64_ptr = realloc_bytes(u64_ptr, __size_of::<u64>(), __size_of::<u64>() * 2);
    assert(realloc_u64_ptr.read::<u64>() == u64::max());
    assert(realloc_u64_ptr.add::<u64>(1).read::<u64>() == u64::zero());

    // Can alloc b256
    let b256_ptr = alloc_bytes(__size_of::<b256>());
    b256_ptr.write(b256::max());
    let realloc_b256_ptr = realloc_bytes(b256_ptr, __size_of::<b256>(), __size_of::<b256>() * 2);
    assert(realloc_b256_ptr.read::<b256>() == b256::max());
    assert(realloc_b256_ptr.add::<b256>(1).read::<b256>() == b256::zero());

    // Can alloc struct
    let address_ptr = alloc_bytes(__size_of_val::<Address>(Address::zero()));
    address_ptr.write(Address::from(b256::max()));
    let realloc_address_ptr = realloc_bytes(
        address_ptr,
        __size_of_val::<Address>(Address::zero()),
        __size_of_val::<Address>(Address::zero()) * 2,
    );
    assert(realloc_address_ptr.read::<Address>() == Address::from(b256::max()));
    assert(realloc_address_ptr.add::<Address>(1).read::<Address>() == Address::zero());

    // TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6086 is resolved
    // Can realloc array
    // let array_ptr = alloc_bytes(__size_of::<[u64; 1]>() * 2);
    // array_ptr.write(u64::max());
    // array_ptr.add::<u64>(1).write(u64::max());
    // let realloc_array_ptr = realloc_bytes(array_ptr, __size_of::<[u64; 1]>() * 2, __size_of::<[u64; 1]>() * 3);
    // assert(realloc_array_ptr.read::<u64>() == u64::max());
    // assert(realloc_array_ptr.add::<u64>(1).read::<u64>() == u64::max());
    // assert(realloc_array_ptr.add::<u64>(2).read::<u64>() == u64::zero());

    // Can alloc multiple
    let count = 100;
    let recount = 1000;

    let multiple_u64 = alloc_bytes(__size_of::<u64>() * count);
    let mut iter = 0;
    while iter < count {
        multiple_u64.add::<u64>(iter).write(u64::max());
        iter += 1;
    }

    let realloc_multiple_u64 = realloc_bytes(
        multiple_u64,
        __size_of::<u64>() * count,
        __size_of::<u64>() * recount,
    );
    let mut iter2 = 0;
    while iter2 < count {
        assert(realloc_multiple_u64.add::<u64>(iter2).read::<u64>() == u64::max());
        iter2 += 1;
    }
    let mut iter3 = count;
    while iter3 < recount {
        assert(realloc_multiple_u64.add::<u64>(iter3).read::<u64>() == 0u64);
        iter3 += 1;
    }

    // Edge cases

    // Realloc to same size
    let same_u64_ptr = alloc_bytes(__size_of::<u64>() * 2);
    same_u64_ptr.write(u64::max());
    let same_realloc_u64_ptr = realloc_bytes(same_u64_ptr, __size_of::<u64>() * 2, __size_of::<u64>() * 2);
    assert(same_realloc_u64_ptr.read::<u64>() == u64::max());
    assert(same_realloc_u64_ptr.add::<u64>(1).read::<u64>() == u64::zero());
    assert(same_realloc_u64_ptr == same_u64_ptr);

    // Realloc to less size
    let less_u64_ptr = alloc_bytes(__size_of::<u64>() * 2);
    less_u64_ptr.write(u64::max());
    let less_realloc_u64_ptr = realloc_bytes(less_u64_ptr, __size_of::<u64>() * 2, __size_of::<u64>() * 1);
    assert(less_realloc_u64_ptr.read::<u64>() == u64::max());
    assert(less_realloc_u64_ptr.add::<u64>(1).read::<u64>() == u64::zero());
    assert(less_realloc_u64_ptr == less_u64_ptr);

    // Realloc excludes values when count is less then total allocated
    let exclude_u64_ptr = alloc_bytes(__size_of::<u64>() * 2);
    exclude_u64_ptr.write(u64::max());
    exclude_u64_ptr.add::<u64>(1).write(u64::max());
    let exclude_realloc_u64_ptr = realloc_bytes(exclude_u64_ptr, __size_of::<u64>(), __size_of::<u64>() * 2);
    assert(exclude_realloc_u64_ptr.read::<u64>() == u64::max());
    assert(exclude_realloc_u64_ptr.add::<u64>(1).read::<u64>() == u64::zero());
}

#[test(should_revert)]
fn revert_alloc_realloc_bytes_when_realloc_unallocated_memory() {
    let u64_ptr = alloc_bytes(__size_of::<u64>());
    u64_ptr.write(u64::max());

    let _realloc_b256_ptr = realloc_bytes(u64_ptr, __size_of::<u64>() * 2, __size_of::<u64>() * 3);
}
