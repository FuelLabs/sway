script;

use std::{alloc::alloc_bytes, bytes::Bytes};

pub trait MyFrom<T> {
    fn my_from(b: T) -> Self;
    fn my_into(self) -> T;
}

impl MyFrom<b256> for Bytes {
    fn my_from(b: b256) -> Self {
        // Artificially create bytes with capacity and len
        let new_ptr = alloc_bytes(32);

        // Copy bytes from contract_id into the buffer of the target bytes
        __addr_of(b).copy_bytes_to(new_ptr, 32);

        Bytes::from(raw_slice::from_parts::<u8>(new_ptr, 32))
    }

    fn my_into(self) -> b256 {
        let mut value = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let ptr = __addr_of(value);
        self.ptr().copy_to::<b256>(ptr, 1);

        value
    }
}

impl MyFrom<u64> for Bytes {
    fn my_from(_b: u64) -> Self {
        let mut bytes = Self::with_capacity(32);
        
        bytes
    }

    fn my_into(self) -> u64 {
        42
    }
}


fn implicit_return_into(bytes: Bytes) -> b256 {
    assert(bytes.len() == 32);
    bytes.my_into()
}

fn explicit_return_into(bytes: Bytes) -> b256 {
    assert(bytes.len() == 32);
    return bytes.my_into();
    // Should not throw:
    //        ^^^^^^^ Multiple applicable items in scope. 
    //        Disambiguate the associated function for candidate #0
    //          <Bytes as MyFrom<b256>>::my_into
    //        Disambiguate the associated function for candidate #1
    //          <Bytes as MyFrom<u64>>::my_into
}

fn main() -> u64 {
    let mut value = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let bytes = Bytes::my_from(value);
    assert(value == implicit_return_into(bytes));
    assert(value == explicit_return_into(bytes));

    1
}
