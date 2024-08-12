script;

fn main() {
    // u8
    let _a = 0x100;
    Vec::<u8>::new().push(_a);

    // u16
    let _a = 0x10000;
    Vec::<u16>::new().push(_a);

    // u32
    let _a = 0x100000000;
    Vec::<u32>::new().push(_a);
}