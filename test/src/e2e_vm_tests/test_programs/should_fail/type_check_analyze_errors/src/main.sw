script;

fn main() {
    let _a = 0x100;
    Vec::<u8>::new().push(_a);

    // u16
    let _a = 0x10000;
    Vec::<u16>::new().push(_a);

    // u32
    let _a = 0x100000000;
    Vec::<u32>::new().push(_a);

    // Array
    let a = [1, 2, "hello"];

    // Array - different numerics
    let a = [1, 2u8, 3u16, 4u32, 5u64];

    // Array - unspecified generic structs
    let a = [None, Some(1), Some(1u8)];
    let _b: Option<u16> = a[1];

    // Wrong cast
    let a = [8, 256u16, 8u8];
    let b: u32 = a[2];
}

fn insufficient_type_check(arg: u64) -> [u32;2] {
    let res = [1u32, arg];
    res
}
