script;
use std::assert::*;

fn main() -> u64 {
    1337
}

struct SA<T1> {
    #[allow(dead_code)]
    a: T1
}

struct SB<T1, T2> {
    #[allow(dead_code)]
    a: T1,
    #[allow(dead_code)]
    b: T2
}

enum EA<T1> {
    #[allow(dead_code)]
    A: T1,
}

enum EB<T1, T2> {
    #[allow(dead_code)]
    A: T1,
    #[allow(dead_code)]
    B: T2,
}

#[test]
fn test_is_memcopy() {
    assert(__encode_memcopy::<()>());

    assert(__encode_memcopy::<bool>());

    assert(__encode_memcopy::<u8>());
    assert(!__encode_memcopy::<u16>());
    assert(!__encode_memcopy::<u32>());
    assert(__encode_memcopy::<u64>());
    assert(__encode_memcopy::<u256>());
    assert(__encode_memcopy::<b256>());

    assert(!__encode_memcopy::<(u8,)>());
    assert(!__encode_memcopy::<(u16,)>());
    assert(!__encode_memcopy::<(u32,)>());
    assert(__encode_memcopy::<(u64,)>());

    assert(!__encode_memcopy::<(u8, u8)>());
    assert(!__encode_memcopy::<(u16, u64)>());
    assert(!__encode_memcopy::<(u64, u16)>());
    assert(!__encode_memcopy::<(u32, u64)>());
    assert(!__encode_memcopy::<(u64, u32)>());
    assert(__encode_memcopy::<(u64, u64)>());

    assert(!__encode_memcopy::<SA<u8>>());
    assert(!__encode_memcopy::<SA<u16>>());
    assert(!__encode_memcopy::<SA<u32>>());
    assert(__encode_memcopy::<SA<u64>>());

    assert(!__encode_memcopy::<SB<u8, u8>>());
    assert(!__encode_memcopy::<SB<u16, u16>>());
    assert(!__encode_memcopy::<SB<u32, u32>>());
    assert(__encode_memcopy::<SB<u64, u64>>());

    assert(!__encode_memcopy::<EA<u8>>());
    assert(!__encode_memcopy::<EA<u16>>());
    assert(!__encode_memcopy::<EA<u32>>());
    assert(__encode_memcopy::<EA<u64>>());

    assert(!__encode_memcopy::<EB<u8, u8>>());
    assert(!__encode_memcopy::<EB<u16, u16>>());
    assert(!__encode_memcopy::<EB<u32, u32>>());
    assert(__encode_memcopy::<EB<u64, u64>>());

    assert(__encode_memcopy::<str[0]>());
    assert(__encode_memcopy::<(u64, u64,u64, u64,u64, u64,u64, u64,u64, u64,)>());
    assert(__encode_memcopy::<str[1]>());
    assert(__encode_memcopy::<str[8]>());
    assert(!__encode_memcopy::<SA<str[1]>>());
    assert(__encode_memcopy::<SA<str[8]>>());

    assert(__encode_memcopy::<[u8; 0]>());
    assert(!__encode_memcopy::<[u8; 1]>());
    assert(__encode_memcopy::<[u8; 8]>());
    assert(!__encode_memcopy::<SA<[u8; 1]>>());
    assert(__encode_memcopy::<SA<[u8; 8]>>());

    assert(!__encode_memcopy::<raw_ptr>());
    assert(!__encode_memcopy::<raw_slice>());
    assert(!__encode_memcopy::<str>());
    assert(!__encode_memcopy::<&__slice[u8]>());
    assert(!__encode_memcopy::<&[u8]>());
}
