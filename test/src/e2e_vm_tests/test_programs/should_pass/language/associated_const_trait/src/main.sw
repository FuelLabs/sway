library;

mod traits;

use traits::*;

struct Struct { }

impl Struct {
    const ID: u32 = 3;

    fn foo() -> u32 {
        Self::ID
    }
}

impl ConstantId for Struct {
    const ID: u32 = 5;
}

impl OtherConstantId for Struct {
    const ID: u32 = 7;
}

impl GenericConstantId<u64> for Struct {
    const ID: u64 = 9;
}

impl GenericConstantId<bool> for Struct {
    const ID: bool = true;
}

impl GenericConstantIdWithDefault<u64> for Struct {
    const ID: u64 = 11;
}

impl GenericConstantIdWithDefault<bool> for Struct {
    const ID: bool = true;
}

enum Enum { }

impl Enum {
    const ID: u32 = 3;

    // TODO: Uncomment this function once https://github.com/FuelLabs/sway/issues/6344 is fixed.
    // fn foo() -> u32 {
    //     Self::ID
    // }
}

impl ConstantId for Enum {
    const ID: u32 = 5;
}

impl OtherConstantId for Enum {
    const ID: u32 = 7;
}

impl GenericConstantId<u64> for Enum {
    const ID: u64 = 9;
}

impl GenericConstantId<bool> for Enum {
    const ID: bool = true;
}

impl GenericConstantIdWithDefault<u64> for Enum {
    const ID: u64 = 11;
}

impl GenericConstantIdWithDefault<bool> for Enum {
    const ID: bool = true;
}

#[test]
fn test_for_structs() {
    // TODO: Uncomment this assert once https://github.com/FuelLabs/sway/issues/6345 is fixed.
    // assert_eq(3, Struct::ID);
    assert_eq(5, <Struct as ConstantId>::ID);
    assert_eq(7, <Struct as OtherConstantId>::ID);

    // TODO: Uncomment these asserts once https://github.com/FuelLabs/sway/issues/6346 is fixed.
    // assert_eq(9, <Struct as GenericConstantId::<u64>>::ID);
    // assert_eq(true, <Struct as GenericConstantId::<bool>>::ID);
    // assert_eq(11, <Struct as GenericConstantIdWithDefault::<u64>>::ID);
    // assert_eq(true, <Struct as GenericConstantIdWithDefault::<bool>>::ID);

    assert_eq(3, Struct::foo());
}


// TODO: Uncomment this test once https://github.com/FuelLabs/sway/issues/6344 is fixed.
// #[test]
// fn test_for_enums() {
//     assert_eq(3, Enum::ID);
//     assert_eq(5, <Enum as ConstantId>::ID);
//     assert_eq(7, <Enum as OtherConstantId>::ID);

//     assert_eq(9, <Enum as GenericConstantId::<u64>>::ID);
//     assert_eq(true, <Enum as GenericConstantId::<bool>>::ID);
//     assert_eq(11, <Enum as GenericConstantIdWithDefault::<u64>>::ID);
//     assert_eq(true, <Enum as GenericConstantIdWithDefault::<bool>>::ID);

//     assert_eq(3, Enum::foo());
// }
