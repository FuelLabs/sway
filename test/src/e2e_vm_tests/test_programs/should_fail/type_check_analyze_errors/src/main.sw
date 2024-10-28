script;

fn main() {
    // 0x100 does not fit into a u8
    let _a = 0x100;
    Vec::<u8>::new().push(_a);

    // 0x10000 does not fit into a u16
    let _a = 0x10000;
    Vec::<u16>::new().push(_a);

    // 0x100000000 does not fit into a u32
    let _a = 0x100000000;
    Vec::<u32>::new().push(_a);

    
}

fn insufficient_type_check(arg: u64) -> [u32;2] {
    let res = [1u32, arg];
    res
}
