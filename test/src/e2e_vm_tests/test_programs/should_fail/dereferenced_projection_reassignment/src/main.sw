script;

struct MyStruct {
    a: u64
}

fn main() -> u32 {
    let mut a = [1u64, 2u64];
    let mut p = &mut a;
    (*p)[1] = 1u8;

    let mut a = (1u64, 2u64);
    let mut p = &mut a;
    (*p).0 = 0u8;

    let mut a = MyStruct { a: 1 };
    let mut p = &mut a;
    (*p).a = 2u8;

    let mut a = [1u64, 2u64];
    let mut p = &mut a;
    let mut p2 = &mut p;
    (**p2)[1] = 1u8;

    let mut a = (1u64, 2u64);
    let mut p = &mut a;
    let mut p2 = &mut p;
    (**p2).0 = 0u8;

    let mut a = MyStruct { a: 1 };
    let mut p = &mut a;
    let mut p2 = &mut p;
    (**p2).a = 2u8;

    0
}
