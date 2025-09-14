library;

// Reexported items from items_1.sw. All reexports aliased by lib_1
use ::lib_1::Items1_Struct;
use ::lib_1::Items1_Enum;
use ::lib_1::Items1_X;
use ::lib_1::Items1_Y;
use ::lib_1::ITEMS_1_FUNCTION_RES;
use ::lib_1::items_1_function;
use ::lib_1::Items1Trait;

use ::items_1::Items1_Variants;

// Reexported items from items_2.sw. All reexports aliased by lib_2
use ::lib_2::Alias2_Struct;
use ::lib_2::Alias2_Enum;
use ::lib_2::Alias2_Z;
use ::lib_2::Alias2_W;
use ::lib_2::ALIAS_2_FUNCTION_RES;
use ::lib_2::alias_2_function;
use ::lib_2::Alias2Trait;

use ::items_2::Items2_Variants;

// Reexported items from items_3.sw. All reexports aliased by lib_3
use ::lib_3::*;

use ::items_3::Items3_Variants;

// Reexported items from items_4.sw. All reexports aliased by lib_4_1 and realiased by lib_4_2
use ::lib_4_2::Alias4_Struct;
use ::lib_4_2::Alias4_Enum;
use ::lib_4_2::Alias4_S;
use ::lib_4_2::Alias4_T;
use ::lib_4_2::ALIAS_4_FUNCTION_RES;
use ::lib_4_2::alias_4_function;
use ::lib_4_2::Alias4Trait;

// Reexported trait from items_5.sw. Aliased both by lib_5_1 and by lib_5_2
use ::lib_5_1::*;
use ::lib_5_2::*;


// Helper types

struct TestStruct1 {
    Z: u64,
}

struct TestStruct2 {
    W: bool,
}


// lib_2 tests

fn project_items_2_struct(input: Items2_Struct) -> u64 {
    input.a
}

fn project_items_2_enum(input: Items2_Enum) -> u64 {
    match input {
        Items2_Enum::A(val) => val,
        Items2_Enum::B(val) => val + 1000,
    }
}

fn project_items_2_variants(input: Items2_Variants) -> u64 {
    match input {
        Items2_X(val) => val,
        Items2_Y(val) => val + 1000,
    }
}

fn call_items_2_function() -> u64 {
    items_2_function()
}

impl Items2Trait<TestStruct2> for TestStruct1 {
    fn items_2_trait_function(self, x: TestStruct2) -> bool {
        self.Z == 64 && x.W
    }
}


// lib_3 tests

fn project_items_3_struct(input: Items3_Struct) -> u64 {
    input.c
}

fn project_items_3_enum(input: Items3_Enum) -> u64 {
    match input {
        Items3_Enum::E(val) => val,
        Items3_Enum::F(val) => val + 1000,
    }
}

fn project_items_3_variants(input: Items3_Variants) -> u64 {
    match input {
        Items3_U(val) => val,
        Items3_V(val) => val + 1000,
    }
}

fn call_items_3_function() -> u64 {
    items_3_function()
}

impl Items3Trait<TestStruct2> for TestStruct1 {
    fn items_3_trait_function(self, x: TestStruct2) -> bool {
        self.Z == 64 && x.W
    }
}

// lib_5 tests

// Alias5Trait and AltAlias5Trait refer to the same trait, but no error is reported for multiple
// impls of same trait for same type.
impl Alias5Trait<TestStruct2> for TestStruct1 {
    fn items_5_trait_function(self, x: TestStruct2) -> bool {
        self.Z == 64 && x.W
    }
}

impl AltAlias5Trait<TestStruct2> for TestStruct1 {
    fn items_5_trait_function(self, x: TestStruct2) -> bool {
        self.Z == 64 && x.W
    }
}

pub fn run_all_tests() -> u64 {
    let items_2_struct = Items2_Struct { b: 123 };
    let items_2_struct_res = project_items_2_struct(items_2_struct);
    poke(items_2_struct_res == 123);

    let items_2_enum = Items2_Enum::C(432);
    let items_2_enum_res = project_items_2_enum(items_2_enum);
    poke(items_2_enum_res == 432);

    let items_2_variants = Z(680);
    let items_2_variants_res = project_items_2_variants(items_2_variants);
    poke(items_2_variants_res == 680);

    let items_2_function_res = call_items_2_function();
    poke(items_2_function_res == ITEMS_2_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_2_trait_teststruct_1_res = teststruct_1.items_2_trait_function(teststruct_2);
    poke(items_2_trait_teststruct_1_res);


    let items_3_struct = Items3_Struct { c: 123 };
    let items_3_struct_res = project_items_3_struct(items_3_struct);
    poke(items_3_struct_res == 123);

    let items_3_enum = Items3_Enum::E(432);
    let items_3_enum_res = project_items_3_enum(items_3_enum);
    poke(items_3_enum_res == 432);

    let items_3_variants = U(680);
    let items_3_variants_res = project_items_3_variants(items_3_variants);
    poke(items_3_variants_res == 680);

    let items_3_function_res = call_items_3_function();
    poke(items_3_function_res == ITEMS_3_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_3_trait_teststruct_1_res = teststruct_1.items_3_trait_function(teststruct_2);
    poke(items_3_trait_teststruct_1_res);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_5_trait_teststruct_1_res = teststruct_1.items_5_trait_function(teststruct_2);
    poke(items_5_trait_teststruct_1_res);

    42
}

fn poke(b: bool) { }
