// This test proves that https://github.com/FuelLabs/sway/issues/5581 is fixed.
script;

mod lib;

use ::lib::Struct;
use ::lib::Struct as StructAlias;

struct MyStruct {
    x: u8,
}

struct MyOtherStruct {
    no_x: u8,
}

struct GenericStruct<A, B> {
    a: A,
    b: B,
}

fn main() {
    let _: MyStruct = Struct { x: 123 };

    let _: MyOtherStruct = Struct { x: 123 };

    let s: StructAlias<u8> = Struct { x: 123 };
    let _ = s.x == 123u8; // No error here.

    let s: Struct<u8> = StructAlias { x: 123 };
    let _ = s.x == 123u8; // No error here.

    let s: StructAlias<u8> = Struct { x: 123u64 };
    let _ = s.x == 123u8; // No error here.

    let s: Struct<u8> = StructAlias { x: 123u64 };
    let _ = s.x == 123u8; // No error here.

    let _: Struct<u8> = Struct { x: 123u64 };
    let _ = s.x == 123u8; // No error here.

    let _: Struct<u8> = Struct::<bool> { x: true };
    let _ = s.x == 123u8; // No error here.

    let _: Struct<u8> = Struct::<bool> { x: "not bool" };
    let _ = s.x == 123u8; // No error here.

    // ---------------------------------------------

    let _: MyStruct = Struct { x: Option::Some(123) };

    let _: MyOtherStruct = Struct { x: Option::Some(123) };

    let s: StructAlias<Option<u8>> = Struct { x: Option::Some(123u64) };
    match s.x {
        Some(x) => x == 123u8, // No error here.
        _ => false,
    };

    let s: Struct<Option<u8>> = StructAlias { x: Option::Some(123u64) };
    match s.x {
        Some(x) => x == 123u8, // No error here.
        _ => false,
    };

    let s: Struct<Option<u8>> = Struct { x: Option::Some(123u64) };
    match s.x {
        Some(x) => x == 123u8, // No error here.
        _ => false,
    };

    let s: Struct<Option<u8>> = Struct::<Option<bool>> { x: Option::Some(true) };
    match s.x {
        // TODO: This should not be an error but it is because of this bug:
        //       https://github.com/FuelLabs/sway/issues/5606
        //       Extend the check after the bug is solved.
        Some(x) => x == 123u8, // No error here.
        _ => false,
    };

    let s: Struct<Option<u8>> = Struct::<Option<bool>> { x: Option::Some("not bool") };
    match s.x {
        Some(x) => x == 123u8, // No error here.
        _ => false,
    };

    let s: GenericStruct<_, bool> = GenericStruct::<u8, _> { a: 123u64, b: true };
    let _ = s.a == 123u8; // No error here.
    let _ = s.b == true; // No error here.

    let s: GenericStruct<u8, bool> = GenericStruct::<u8, u32> { a: 123, b: true };
    let _ = s.a == 123u8; // No error here.
    let _ = s.b == true; // No error here.
}
