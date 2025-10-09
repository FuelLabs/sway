library;

// Reexported items from items_1.sw. All reexports shadowed by private local definitions in lib_1.sw.
use ::lib_1::*;
 // Reexported items from items_2_1.sw and items_2_2.sw. All reexports from items_2_1.sw are
 // shadowed by items imported and not reexported from items_2_2.sw
use ::lib_2::*;
// Reexported items from items_3_1.sw. All reexports shadowed by private local
// definitions in lib_3.sw.
use ::lib_3::Items3_Struct;
use ::lib_3::Items3_Enum;
use ::lib_3::Items3_Variants;
use ::lib_3::Items3_Variants::G;
use ::lib_3::Items3_Variants::H;
use ::lib_3::ITEMS_3_FUNCTION_RES;
use ::lib_3::items_3_function;
use ::lib_3::Items3Trait;

// Reexported items from items_4_1.sw and items_4_2.sw. All reexports from items_4_1.sw are
// shadowed by items imported and not reexported from items_4_2.sw
use ::lib_4::Items4_Struct;
use ::lib_4::Items4_Enum;
use ::lib_4::Items4_Variants;

// This ought to be possible, but Items4_Variants is interpreted as a module rather than as an enum.
use ::lib_4::Items4_Variants::K;
use ::lib_4::Items4_Variants::L;
// Using the variant names directly works, but that's not how the test should work.
// Uncomment the previous two imports and remove these next two when the problem has been resolved.
//use ::lib_4::K;
//use ::lib_4::L;

use ::lib_4::ITEMS_4_FUNCTION_RES;
use ::lib_4::items_4_function;
use ::lib_4::Items4Trait;
// Items4_Variants2 defined in items_4_4.sw, variants reexported by lib_4.sw
use ::items_4_4::Items4_Variants2;
use ::lib_4::M;
use ::lib_4::N;




// Helper types

struct TestStruct1 {
    Z: u64,
}

struct TestStruct2 {
    W: bool,
}


// lib_1 tests

fn project_items_1_struct(input: Items1_Struct) -> u64 {
    input.a
}

fn project_items_1_enum(input: Items1_Enum) -> u64 {
    match input {
        Items1_Enum::A(val) => val,
        Items1_Enum::B(val) => val + 1000,
    }
}

fn call_items_1_function() -> u64 {
    items_1_function()
}

impl Items1Trait<TestStruct2> for TestStruct1 {
    fn items_1_trait_function(self, x: TestStruct2) -> u64 {
        if x.W {
            self.Z
        }
        else {
            0
        }
    }
}


// lib_2 tests

fn project_items_2_struct(input: Items2_Struct) -> u64 {
    input.b
}

fn project_items_2_enum(input: Items2_Enum) -> u64 {
    match input {
        Items2_Enum::C(val) => val,
        Items2_Enum::D(val) => val + 1000,
    }
}

fn call_items_2_function() -> u64 {
    items_2_function()
}

impl Items2Trait<TestStruct2> for TestStruct1 {
    fn items_2_trait_function(self, x: TestStruct2) -> u64 {
        if x.W {
            0
        }
        else {
            self.Z
        }
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
        Items3_Variants::G(val) => val,
        Items3_Variants::H(val) => val + 1000,
    }
}

fn call_items_3_function() -> u64 {
    items_3_function()
}

impl Items3Trait<TestStruct2> for TestStruct1 {
    fn items_3_trait_function(self, x: TestStruct2) -> u64 {
        if x.W {
            self.Z
        }
        else {
            0
        }
    }
}

// lib_4 tests

fn project_items_4_struct(input: Items4_Struct) -> u64 {
    input.d
}

fn project_items_4_enum(input: Items4_Enum) -> u64 {
    match input {
        Items4_Enum::I(val) => val,
        Items4_Enum::J(val) => val + 1000,
    }
}

fn project_items_4_variants(input: Items4_Variants) -> u64 {
    match input {
        K(val) => val,
        L(val) => val + 1000,
    }
}

fn call_items_4_function() -> u64 {
    items_4_function()
}

impl Items4Trait<TestStruct2> for TestStruct1 {
    fn items_4_trait_function(self, x: TestStruct2) -> u64 {
        if x.W {
            0
        }
        else {
            self.Z
        }
    }
}

fn project_items_4_variants2(input: Items4_Variants2) -> u64 {
    match input {
        M(val) => val,
        N(val) => val + 1000,
    }
}



pub fn run_all_tests() -> u64 {
    let items_1_struct = Items1_Struct { a: 123 };
    let items_1_struct_res = project_items_1_struct(items_1_struct);
    poke(items_1_struct_res == 123);

    let items_1_enum = Items1_Enum::A(432);
    let items_1_enum_res = project_items_1_enum(items_1_enum);
    poke(items_1_enum_res == 432);

    let items_1_function_res = call_items_1_function();
    poke(items_1_function_res == ITEMS_1_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_1_trait_teststruct_1_res = teststruct_1.items_1_trait_function(teststruct_2);
    poke(items_1_trait_teststruct_1_res == 64);


    let items_2_struct = Items2_Struct { b: 879 };
    let items_2_struct_res = project_items_2_struct(items_2_struct);
    poke(items_2_struct_res == 879);

    let items_2_enum = Items2_Enum::C(246);
    let items_2_enum_res = project_items_2_enum(items_2_enum);
    poke(items_2_enum_res == 246);

    let items_2_function_res = call_items_2_function();
    poke(items_2_function_res == ITEMS_2_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 128 };
    let teststruct_2 = TestStruct2 { W : false };
    let items_2_trait_teststruct_1_res = teststruct_1.items_2_trait_function(teststruct_2);
    poke(items_2_trait_teststruct_1_res == 128);


    let items_3_struct = Items3_Struct { c: 123 };
    let items_3_struct_res = project_items_3_struct(items_3_struct);
    poke(items_3_struct_res == 123);

    let items_3_enum = Items3_Enum::E(432);
    let items_3_enum_res = project_items_3_enum(items_3_enum);
    poke(items_3_enum_res == 432);

    let items_3_variants = G(432);
    let items_3_variants_res = project_items_3_variants(items_3_variants);
    poke(items_3_variants_res == 432);

    let items_3_function_res = call_items_3_function();
    poke(items_3_function_res == ITEMS_3_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 64 };
    let teststruct_2 = TestStruct2 { W : true };
    let items_3_trait_teststruct_1_res = teststruct_1.items_3_trait_function(teststruct_2);
    poke(items_3_trait_teststruct_1_res == 64);

    
    let items_4_struct = Items4_Struct { d: 879 };
    let items_4_struct_res = project_items_4_struct(items_4_struct);
    poke(items_4_struct_res == 879);

    let items_4_enum = Items4_Enum::I(446);
    let items_4_enum_res = project_items_4_enum(items_4_enum);
    poke(items_4_enum_res == 446);

    let items_4_variants = K(446);
    let items_4_variants_res = project_items_4_variants(items_4_variants);
    poke(items_4_variants_res == 446);

    let items_4_function_res = call_items_4_function();
    poke(items_4_function_res == ITEMS_4_FUNCTION_RES);

    let teststruct_1 = TestStruct1 { Z : 148 };
    let teststruct_2 = TestStruct2 { W : false };
    let items_4_trait_teststruct_1_res = teststruct_1.items_4_trait_function(teststruct_2);
    poke(items_4_trait_teststruct_1_res == 148);

    let items_4_variants2 = M(446);
    let items_4_variants2_res = project_items_4_variants2(items_4_variants2);
    poke(items_4_variants2_res == 446);

    42
}

fn poke(b: bool) { }